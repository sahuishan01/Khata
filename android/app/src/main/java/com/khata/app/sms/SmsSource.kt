package com.khata.app.sms

import com.google.gson.annotations.SerializedName
import com.google.gson.Gson
import java.security.MessageDigest
import java.time.LocalDate
import java.util.Locale

/** Provenance without retaining SMS text, sender, account numbers or balances. */
data class SmsSource(
    @SerializedName("parser_version") val parserVersion: Int = 1,
    @SerializedName("received_at") val receivedAt: Long? = null,
    @SerializedName("body_hash") val bodyHash: String? = null,
    @SerializedName("event_key") val eventKey: String? = null,
    @SerializedName("date_source") val dateSource: String,
    @SerializedName("parsed_date") val parsedDate: String,
    @SerializedName("previous_txn_date") val previousTxnDate: String? = null,
    @SerializedName("previous_value_date") val previousValueDate: String? = null,
    @SerializedName("payment_role") val paymentRole: String? = null,
    @SerializedName("card_account_key") val cardAccountKey: String? = null,
    @SerializedName("user_edited") val userEdited: Boolean? = null,
    @SerializedName("parsed_payee") val parsedPayee: String? = null,
) {
    companion object {
        /** Older/minified metadata must never prevent the financial row uploading. */
        fun decode(json: String?): SmsSource? {
            val source = json?.let { runCatching { Gson().fromJson(it, SmsSource::class.java) }.getOrNull() }
                ?: return null
            return source.takeIf {
                it.dateSource in listOf("body", "message", "queue") &&
                    runCatching { LocalDate.parse(it.parsedDate) }.isSuccess &&
                    it.parserVersion in 0..1 &&
                    (it.bodyHash == null || it.bodyHash.matches(Regex("[a-fA-F0-9]{64}"))) &&
                    (it.eventKey == null || it.eventKey == it.bodyHash) &&
                    (it.paymentRole == null || it.paymentRole in listOf("card_receipt", "card_confirmation")) &&
                    (it.cardAccountKey == null || it.cardAccountKey.matches(Regex("[a-fA-F0-9]{64}")))
            }
        }

        private val account = Regex("(?i)\\b(?:a/c|ac|account|card)\\s*(?:no\\.?|number)?\\s*[:.-]?\\s*[xX*0-9]{2,}\\d{2,}")
        private val balance = Regex("(?i)\\b(?:avl\\.?|avail(?:able)?|closing|clear)?\\s*bal(?:ance)?\\.?\\s*[:=-]?\\s*(?:rs\\.?|inr|₹)?\\s*[0-9,]+(?:\\.[0-9]{1,2})?")
        private val clock = Regex("(?<![0-9])(?:[01]?[0-9]|2[0-3]):[0-5][0-9](?::[0-5][0-9])?(?![0-9])")

        fun from(body: String, timestamp: Long, queuedAt: Long, direction: String? = null, bank: String? = null): SmsSource {
            val resolved = SmsDates.resolution(body, timestamp, queuedAt)
            val normalized = body.trim().replace(Regex("\\s+"), " ").lowercase(Locale.ROOT)
            val hash = MessageDigest.getInstance("SHA-256").digest(normalized.toByteArray(Charsets.UTF_8))
                .joinToString("") { "%02x".format(it) }
            // An identical amount/payee/day, or even identical generic SMS text, is insufficient.
            // Require an account plus a changing balance or an explicit transaction date/time.
            val anchored = account.containsMatchIn(body) && (balance.containsMatchIn(body) ||
                (resolved.source == "body" && clock.containsMatchIn(body)))
            val cardCredit = direction == "credit" && CardTransfers.isRepayment(body, direction)
            val role = if (cardCredit) {
                if (Regex("(?i)\\b(received|acknowledged)\\b").containsMatchIn(body)) "card_receipt" else "card_confirmation"
            } else null
            val cardToken = if (cardCredit) Regex("(?i)\\bcard\\s*(?:(?:no\\.?|number|ending(?:\\s+(?:in|with))?)\\s*)?[: -]*([xX*0-9]{4,24})(?![0-9])")
                .find(body)?.groupValues?.get(1)?.filter { it.isDigit() }?.takeIf { it.length >= 4 }?.takeLast(4) else null
            val cardKey = cardToken?.let { token -> MessageDigest.getInstance("SHA-256")
                .digest("card|${bank?.uppercase(Locale.ROOT)}|$token".toByteArray(Charsets.UTF_8)).joinToString("") { "%02x".format(it) } }
            return SmsSource(receivedAt = timestamp.takeIf { it > 0 && it <= queuedAt },
                bodyHash = hash, eventKey = hash.takeIf { anchored },
                dateSource = resolved.source, parsedDate = resolved.date, paymentRole = role, cardAccountKey = cardKey)
        }
    }
}
