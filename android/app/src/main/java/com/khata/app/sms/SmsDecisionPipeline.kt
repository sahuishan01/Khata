package com.khata.app.sms

import android.content.Context
import androidx.room.withTransaction
import androidx.work.*
import com.khata.app.api.toLocal
import com.khata.app.data.KhataDatabase
import dagger.hilt.EntryPoint
import dagger.hilt.InstallIn
import dagger.hilt.android.EntryPointAccessors
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import java.util.Locale
import java.util.concurrent.TimeUnit
import javax.inject.Inject
import javax.inject.Singleton

enum class SmsWorkOutcome { DONE, MORE, RETRY }

@Singleton
class SmsDecisionPipeline @Inject constructor(
    @ApplicationContext private val context: Context,
    private val database: KhataDatabase,
    private val client: SmsDecisionClient,
) {
    private val mutex = Mutex()
    private val dao get() = database.smsDecisionDao()

    fun active(): Flow<List<SmsDecisionJob>> = client.session()?.let { dao.active(it.owner) } ?: flowOf(emptyList())

    internal fun schedule(owner: String) {
        val request = OneTimeWorkRequestBuilder<SmsDecisionWorker>()
            .setInputData(workDataOf("owner" to owner))
            .setConstraints(Constraints.Builder().setRequiredNetworkType(NetworkType.CONNECTED).build())
            .setBackoffCriteria(BackoffPolicy.EXPONENTIAL, 30, TimeUnit.SECONDS)
            .addTag("sms-decisions").build()
        WorkManager.getInstance(context).enqueueUniqueWork("sms-decisions-$owner", ExistingWorkPolicy.APPEND_OR_REPLACE, request)
    }

    fun resume() { client.session()?.let { schedule(it.owner) } }

    suspend fun enqueue(body: String, sender: String, timestamp: Long, scheduleNow: Boolean = true, legacyTimestamp: Long? = null): Boolean {
        // No OTP is persisted in the decision queue or placed in WorkManager input.
        if (SmsPrivacy.containsOtp(body) || body.isBlank() || body.length > 16000) return false
        val session = client.session() ?: return false
        val id = smsDigest("${session.owner}|sms-choice-v1|${sender.uppercase(Locale.ROOT)}|$timestamp|$body")
        val created = dao.insert(SmsDecisionJob(id,session.owner,sender,body,timestamp,System.currentTimeMillis(),legacyReceivedAt=legacyTimestamp?.takeIf { it != timestamp })) != -1L
        if (created && scheduleNow) schedule(session.owner)
        return created
    }

    suspend fun retryReview() {
        val session = client.session() ?: return
        dao.retryReview(session.owner); schedule(session.owner)
    }

    suspend fun ignore(id: String) {
        val session = client.session() ?: return
        if (dao.review(session.owner).any { it.id == id }) dao.finish(id,"ignored")
    }

    suspend fun resolve(id: String, amount: Double, direction: String): String? = mutex.withLock {
        val session = client.session() ?: return@withLock "Sign in before saving."
        val job = dao.review(session.owner).firstOrNull { it.id == id } ?: return@withLock "This SMS was already processed."
        val body = job.body ?: return@withLock "The original SMS is unavailable."
        val parsed = SmsParser.forManualReview(body,job.sender,amount,direction) ?: return@withLock "Enter a valid amount and direction."
        try {
            val (previous,draft) = client.prepare(session,parsed,body,job.sender,job.receivedAt,job.queuedAt,job.legacyReceivedAt)
            if (draft == previous) { dao.finish(id,"done"); return@withLock null }
            val localBefore = database.transactionDao().getByClientId(draft.clientId)
            val source = SmsSource.decode(draft.smsSource) ?: return@withLock "SMS source could not be verified."
            val row = draft.copy(smsSource=com.google.gson.Gson().toJson(source.copy(userEdited=true)))
            val saved = client.save(session,row)
            if (client.session()?.owner != session.owner) return@withLock "Your sign-in changed; reopen this SMS."
            database.withTransaction {
                val txns=database.transactionDao()
                if (txns.getByClientId(row.clientId)==localBefore && localBefore?.dirty != true) txns.upsert(saved.toLocal().copy(id=localBefore?.id ?: saved.id))
                dao.finish(id,"done")
            }
            null
        } catch (e: CancellationException) { throw e }
        catch (_: Exception) { "Could not save. Your SMS remains available for retry." }
    }

    /** Returns true when pending work needs retrying; original SMS remains local on errors. */
    suspend fun runPending(owner: String): SmsWorkOutcome = mutex.withLock {
        val session = client.session()?.takeIf { it.owner == owner } ?: return@withLock SmsWorkOutcome.DONE
        for (job in dao.pending(owner)) {
            if (client.session()?.owner != owner) return@withLock SmsWorkOutcome.DONE
            val body = job.body ?: continue
            if (SmsPrivacy.containsOtp(body)) { dao.finish(job.id,"ignored"); continue }
            try {
                val decision = client.decide(session,body)
                if (client.session()?.owner != owner) return@withLock SmsWorkOutcome.DONE
                if (decision.status == "ignored" && decision.isTransaction == false) {
                    if (SmsParser.parse(body,job.sender) != null)
                        dao.finish(job.id,"review","The transaction decision conflicts with a posting in the SMS.",true)
                    else dao.finish(job.id,"ignored")
                    continue
                }
                if (decision.status != "decision" || decision.isTransaction != true || decision.isDebit == null) {
                    dao.finish(job.id,"review","The model could not decide confidently.",true); continue
                }
                val direction = if (decision.isDebit) "debit" else "credit"
                val parsed = SmsParser.fromDecision(body,job.sender,direction,decision.modelVersion)
                if (parsed == null) {
                    dao.finish(job.id,"review","The amount, direction or completed status needs checking.",true); continue
                }
                val txnDao = database.transactionDao()
                val (previous,row) = client.prepare(session,parsed,body,job.sender,job.receivedAt,job.queuedAt,job.legacyReceivedAt)
                if (row == previous) { dao.finish(job.id,"done"); continue }
                if (client.session()?.owner != owner) return@withLock SmsWorkOutcome.DONE
                val localBefore = txnDao.getByClientId(row.clientId)
                val saved = client.save(session,row)
                if (client.session()?.owner != owner) return@withLock SmsWorkOutcome.DONE
                database.withTransaction {
                    if (txnDao.getByClientId(row.clientId) == localBefore && localBefore?.dirty != true) txnDao.upsert(saved.toLocal().copy(id=localBefore?.id ?: saved.id))
                    dao.finish(job.id,"done")
                }
            } catch (e: CancellationException) { throw e }
            catch (_: SmsIdentityConflict) {
                dao.finish(job.id,"review","The older saved SMS transaction needs checking.",true)
            }
            catch (e: SmsDecisionFailure) {
                if (e.retry) return@withLock SmsWorkOutcome.RETRY
                dao.finish(job.id,"review","Sign in again or review the saved transaction before retrying.",true)
            } catch (_: Exception) { return@withLock SmsWorkOutcome.RETRY }
        }
        if (dao.pending(owner).isNotEmpty()) SmsWorkOutcome.MORE else SmsWorkOutcome.DONE
    }
}

@EntryPoint
@InstallIn(SingletonComponent::class)
interface SmsDecisionEntryPoint { fun smsDecisionPipeline(): SmsDecisionPipeline }

class SmsDecisionWorker(context: Context, parameters: WorkerParameters) : CoroutineWorker(context,parameters) {
    override suspend fun doWork(): Result {
        val owner = inputData.getString("owner") ?: return Result.failure()
        val pipeline = EntryPointAccessors.fromApplication(applicationContext,SmsDecisionEntryPoint::class.java).smsDecisionPipeline()
        return when (pipeline.runPending(owner)) {
            SmsWorkOutcome.RETRY -> Result.retry()
            SmsWorkOutcome.MORE -> { pipeline.schedule(owner); Result.success() }
            SmsWorkOutcome.DONE -> Result.success()
        }
    }
}
