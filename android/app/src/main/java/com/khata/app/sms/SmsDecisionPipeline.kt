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

    fun active(owner: String): Flow<List<SmsDecisionJob>> = dao.active(owner)
    fun summary(owner: String): Flow<SmsDecisionSummary> = dao.summary(owner)

    internal fun schedule(owner: String, restart: Boolean = false) {
        val request = OneTimeWorkRequestBuilder<SmsDecisionWorker>()
            .setInputData(workDataOf("owner" to owner))
            .setConstraints(Constraints.Builder().setRequiredNetworkType(NetworkType.CONNECTED).build())
            .setBackoffCriteria(BackoffPolicy.EXPONENTIAL, 30, TimeUnit.SECONDS)
            .addTag("sms-decisions").build()
        val policy = if (restart) ExistingWorkPolicy.REPLACE else ExistingWorkPolicy.APPEND_OR_REPLACE
        WorkManager.getInstance(context).enqueueUniqueWork("sms-decisions-$owner", policy, request)
    }

    fun resume() { client.session()?.let { schedule(it.owner, restart=true) } }

    suspend fun enqueue(body: String, sender: String, timestamp: Long, scheduleNow: Boolean = true, legacyTimestamp: Long? = null): Boolean {
        // No OTP is persisted in the decision queue or placed in WorkManager input.
        if (body.length > 16000 || SmsParser.parse(body,sender) == null) return false
        val session = client.session() ?: return false
        val id = smsDigest("${session.owner}|sms-local-v1|${sender.uppercase(Locale.ROOT)}|$timestamp|$body")
        val created = dao.insert(SmsDecisionJob(id,session.owner,sender,body,timestamp,System.currentTimeMillis(),legacyReceivedAt=legacyTimestamp?.takeIf { it != timestamp })) != -1L
        if (created && scheduleNow) schedule(session.owner)
        return created
    }

    suspend fun retryReview() {
        val session = client.session() ?: return
        dao.retryReview(session.owner); schedule(session.owner, restart=true)
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

    /** Recheck old-version pending/review work before any body can leave the device. */
    private suspend fun filterBacklog(owner: String) {
        var after = ""
        while (client.session()?.owner == owner) {
            val page = dao.unfinishedPage(owner,after)
            if (page.isEmpty()) return
            val unrelated = page.filter { it.body == null || SmsParser.parse(it.body,it.sender) == null }
            if (unrelated.isNotEmpty()) database.withTransaction {
                for (job in unrelated) dao.finish(job.id,"filtered","Skipped by the local SMS parser.")
            }
            after = page.last().id
        }
    }

    /** Parse on the device and upload through the existing bulk-sync protocol. */
    suspend fun runPending(owner: String): SmsWorkOutcome = mutex.withLock {
        val session = client.session()?.takeIf { it.owner == owner } ?: return@withLock SmsWorkOutcome.DONE
        dao.resumeLegacyReviews(owner)
        filterBacklog(owner)
        val jobs = dao.pending(owner)
        if (jobs.isEmpty()) return@withLock SmsWorkOutcome.DONE
        var madeProgress = false
        try {
            val snapshot = client.history(session)
            if (client.session()?.owner != owner) return@withLock SmsWorkOutcome.DONE
            val drafts = mutableListOf<Pair<SmsDecisionJob,com.khata.app.data.LocalTransaction>>()
            for (job in jobs) {
                val body = job.body
                val parsed = body?.let { SmsParser.parse(it,job.sender) }
                if (body == null || parsed == null) {
                    dao.finish(job.id,"filtered"); madeProgress=true; continue
                }
                try {
                    val (previous,row) = SmsImport.prepareWithLookup(parsed,body,job.sender,job.receivedAt,job.queuedAt,
                        alternateTimestamp=job.legacyReceivedAt,lookup=snapshot.rows::get)
                    if (row == previous) { dao.finish(job.id,"done"); madeProgress=true }
                    else drafts += job to row
                } catch (_: SmsIdentityConflict) {
                    dao.finish(job.id,"conflict","The older saved SMS transaction needs checking.",true); madeProgress=true
                }
            }
            if (drafts.isNotEmpty()) {
                if (client.session()?.owner != owner) return@withLock SmsWorkOutcome.DONE
                val txnDao = database.transactionDao()
                val before = drafts.associate { (_,row) -> row.clientId to txnDao.getByClientId(row.clientId) }
                val result = client.saveBatch(session,drafts.map { it.second }.distinctBy { it.clientId })
                if (client.session()?.owner != owner) return@withLock SmsWorkOutcome.DONE
                val canonical = client.history(session,snapshot.cursor).rows
                if (client.session()?.owner != owner) return@withLock SmsWorkOutcome.DONE
                val accepted = result.accepted.map { it.clientId }.toSet()
                val conflicts = result.conflicts.map { it.clientId }.toSet()
                database.withTransaction {
                    for ((job,row) in drafts) {
                        if (row.clientId in accepted) {
                            val saved = canonical[row.clientId] ?: throw SmsDecisionFailure(true)
                            val old = before[row.clientId]
                            if (txnDao.getByClientId(row.clientId) == old && old?.dirty != true)
                                txnDao.upsert(saved.copy(id=old?.id ?: saved.id))
                            dao.finish(job.id,"done")
                        } else if (row.clientId in conflicts) {
                            dao.finish(job.id,"conflict","A saved transaction changed. Review it before retrying.",true)
                        } else throw SmsDecisionFailure(true)
                    }
                }
                madeProgress=true
            }
        } catch (e: CancellationException) { throw e }
        catch (e: SmsDecisionFailure) {
            if (!e.retry) {
                val pendingIds = dao.pending(owner).map { it.id }.toSet()
                for (job in jobs) if (job.id in pendingIds)
                    dao.finish(job.id,"conflict",e.message ?: "Review the saved transaction before retrying.",true)
                return@withLock if (dao.pending(owner).isEmpty()) SmsWorkOutcome.DONE else SmsWorkOutcome.MORE
            }
            dao.pendingReason(jobs.first().id,"Waiting for a connection to upload locally parsed transactions.")
            return@withLock if (madeProgress) SmsWorkOutcome.MORE else SmsWorkOutcome.RETRY
        } catch (_: Exception) {
            dao.pendingReason(jobs.first().id,"Waiting for a connection to upload locally parsed transactions.")
            return@withLock if (madeProgress) SmsWorkOutcome.MORE else SmsWorkOutcome.RETRY
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
