package com.khata.app.sms

import com.google.gson.Gson
import com.google.gson.annotations.SerializedName
import com.khata.app.api.TokenManager
import com.khata.app.api.TxnRow
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

data class SmsDecision(
    val status: String,
    @SerializedName("is_transaction") val isTransaction: Boolean?,
    @SerializedName("is_debit") val isDebit: Boolean?,
    @SerializedName("transaction_probability") val transactionProbability: Double?,
    @SerializedName("debit_probability") val debitProbability: Double?,
    @SerializedName("source_hash") val sourceHash: String,
    @SerializedName("model_version") val modelVersion: String,
)

class SmsDecisionFailure(val retry: Boolean) : Exception("SMS decision service unavailable")

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
            if (source.request(256_001)) throw SmsDecisionFailure(false)
            gson.fromJson(source.readUtf8(), type)
        }
    }

    private suspend fun <T> post(session: SmsSession, path: String, payload: Any, type: Class<T>): T =
        postOrNull(session,path,payload,type) ?: throw SmsDecisionFailure(true)

    suspend fun prepare(session: SmsSession, parsed: ParsedSmsTxn, body: String, sender: String, timestamp: Long, queuedAt: Long, legacyTimestamp: Long? = null): Pair<LocalTransaction?,LocalTransaction> {
        val found=mutableMapOf<String,LocalTransaction?>()
        return SmsImport.prepareWithLookup(parsed,body,sender,timestamp,queuedAt,alternateTimestamp=legacyTimestamp) { id ->
            if (runCatching { UUID.fromString(id) }.isFailure) null
            else {
                if (!found.containsKey(id)) found[id]=postOrNull(session,"/api/sms/lookup",mapOf("owner_id" to session.ownerId,"client_id" to id),TxnRow::class.java)?.toLocal()
                found[id]
            }
        }
    }

    suspend fun decide(session: SmsSession, body: String): SmsDecision {
        if (!SmsCandidateFilter.isCandidate(body)) throw SmsDecisionFailure(false)
        return post(session, "/api/sms/decide", mapOf("owner_id" to session.ownerId, "body" to body), SmsDecision::class.java)
            .also {
                if (it.sourceHash != smsDigest(body) || !it.modelVersion.startsWith("laya:")) throw SmsDecisionFailure(false)
            }
    }

    suspend fun save(session: SmsSession, row: LocalTransaction): TxnRow {
        if (SmsPrivacy.containsOtp(row.notes)) throw SmsDecisionFailure(false)
        val operation = mapOf("client_id" to row.clientId, "base_rev" to row.baseRev,
            "description" to row.description, "amount" to row.amount, "direction" to row.direction,
            "category" to row.category, "txn_date" to row.txnDate, "value_date" to row.valueDate,
            "bank" to row.bank, "bank_ref" to row.bankRef, "notes" to row.notes,
            "is_transfer" to row.isTransfer, "deleted" to row.deleted,
            "sms_source" to SmsSource.decode(row.smsSource))
        return post(session, "/api/sms/import", mapOf("owner_id" to session.ownerId, "operation" to operation), TxnRow::class.java)
    }
}
