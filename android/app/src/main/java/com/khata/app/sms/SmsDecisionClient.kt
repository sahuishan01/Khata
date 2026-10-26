package com.khata.app.sms

import com.google.gson.Gson
import com.khata.app.api.TokenManager
import com.khata.app.api.TxnRow
import com.khata.app.api.SyncPullResponse
import com.khata.app.api.SyncPushOp
import com.khata.app.api.SyncPushResult
import com.khata.app.api.toLocal
import com.khata.app.data.LocalTransaction
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody.Companion.toRequestBody
import java.security.MessageDigest
import java.util.Base64
import java.util.UUID
import java.util.concurrent.TimeUnit
import javax.inject.Inject
import javax.inject.Singleton

fun smsDigest(value: String): String = MessageDigest.getInstance("SHA-256")
    .digest(value.toByteArray(Charsets.UTF_8)).joinToString("") { "%02x".format(it) }

// No generated toString: a session must never print its bearer token.
class SmsSession(val ownerId: String, val owner: String, val origin: String, val token: String)

fun smsSession(token: String?, origin: String): SmsSession? = runCatching {
    if (token.isNullOrBlank()) return null
    val claims = Gson().fromJson(String(Base64.getUrlDecoder().decode(token.split('.')[1]), Charsets.UTF_8), Map::class.java)
    val id = UUID.fromString(claims["sub"] as String).toString()
    SmsSession(id, smsDigest("${origin.trimEnd('/')}|$id"), origin.trimEnd('/'), token)
}.getOrNull()

data class SmsHistory(val rows: Map<String, LocalTransaction>, val cursor: Long)

class SmsDecisionFailure(val retry: Boolean) : Exception("SMS upload unavailable")

@Singleton
class SmsDecisionClient @Inject constructor(private val tokens: TokenManager) {
    private val gson = Gson()
    // Captured origin/token are used together, independently of mutable global interceptors.
    private val http = OkHttpClient.Builder().connectTimeout(10, TimeUnit.SECONDS)
        .readTimeout(35, TimeUnit.SECONDS).writeTimeout(15, TimeUnit.SECONDS)
        .followRedirects(false).followSslRedirects(false).proxy(java.net.Proxy.NO_PROXY).build()
    fun session(): SmsSession? = smsSession(tokens.getTokenSync(), tokens.getServerUrlSync())

    private suspend fun <T> postOrNull(session: SmsSession, path: String, payload: Any, type: Class<T>): T? = withContext(Dispatchers.IO) {
        val request = Request.Builder().url(session.origin + path)
            .header("Authorization", "Bearer ${session.token}")
            .post(gson.toJson(payload).toRequestBody("application/json".toMediaType())).build()
        http.newCall(request).execute().use { response ->
            if (!response.isSuccessful) throw SmsDecisionFailure(response.code !in listOf(400,401,403,409,422))
            val source = response.body?.source() ?: throw SmsDecisionFailure(true)
            if (source.request(8_000_001)) throw SmsDecisionFailure(false)
            gson.fromJson(source.readUtf8(), type)
        }
    }

    private suspend fun <T> post(session: SmsSession, path: String, payload: Any, type: Class<T>): T =
        postOrNull(session,path,payload,type) ?: throw SmsDecisionFailure(true)

    /** Captured owner/origin/token also scope the normal bulk-sync protocol. */
    suspend fun history(session: SmsSession, since: Long = 0): SmsHistory = withContext(Dispatchers.IO) {
        val rows = linkedMapOf<String,LocalTransaction>()
        var cursor = since
        while (true) {
            val request = Request.Builder().url(session.origin + "/api/sync/pull?since_rev=$cursor&limit=200&cursor_version=2")
                .header("Authorization", "Bearer ${session.token}").build()
            val page = http.newCall(request).execute().use { response ->
                if (!response.isSuccessful) throw SmsDecisionFailure(response.code !in listOf(400,401,403,409,422))
                val source = response.body?.source() ?: throw SmsDecisionFailure(true)
                if (source.request(8_000_001)) throw SmsDecisionFailure(false)
                gson.fromJson(source.readUtf8(), SyncPullResponse::class.java) ?: throw SmsDecisionFailure(true)
            }
            if (page.hasMore && (page.txns.isEmpty() || page.newRev <= cursor)) throw SmsDecisionFailure(true)
            for (row in page.txns) rows[row.clientId ?: row.id] = row.toLocal()
            cursor = page.newRev
            if (!page.hasMore) break
        }
        SmsHistory(rows,cursor)
    }

    suspend fun prepare(session: SmsSession, parsed: ParsedSmsTxn, body: String, sender: String, timestamp: Long, queuedAt: Long, legacyTimestamp: Long? = null): Pair<LocalTransaction?,LocalTransaction> {
        val snapshot = history(session)
        return SmsImport.prepareWithLookup(parsed,body,sender,timestamp,queuedAt,alternateTimestamp=legacyTimestamp,lookup=snapshot.rows::get)
    }

    suspend fun saveBatch(session: SmsSession, rows: List<LocalTransaction>): SyncPushResult =
        post(session,"/api/sync/push",rows.map(::operation),SyncPushResult::class.java)

    suspend fun save(session: SmsSession, row: LocalTransaction): TxnRow =
        post(session,"/api/sms/import",mapOf("owner_id" to session.ownerId,"operation" to operation(row)),TxnRow::class.java)

    private fun operation(row: LocalTransaction): SyncPushOp {
        if (SmsPrivacy.containsOtp(row.notes)) throw SmsDecisionFailure(false)
        return SyncPushOp(clientId=row.clientId,baseRev=row.baseRev,
            description=row.description,amount=row.amount,direction=row.direction,category=row.category,
            txnDate=row.txnDate,valueDate=row.valueDate,bank=row.bank,bankRef=row.bankRef,
            notes=row.notes,isTransfer=row.isTransfer,deleted=row.deleted,smsSource=SmsSource.decode(row.smsSource))
    }
}
