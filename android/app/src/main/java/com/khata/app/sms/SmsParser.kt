package com.khata.app.sms

import java.util.Locale
import java.util.UUID
import java.util.regex.Pattern

data class ParsedSmsTxn(
    val amount: Double,
    val direction: String, // "debit" or "credit"
    val bank: String,
    val payee: String,
    val refNo: String? = null,
    val isTransfer: Boolean = false,
)

object SmsParser {
    /** Shared by live capture and inbox backfill; accepted by the UUID API. */
    fun clientId(parsed: ParsedSmsTxn, body: String, sender: String, timestamp: Long): String {
        val identity = parsed.refNo?.let {
            "${parsed.bank.uppercase(Locale.ROOT)}|${parsed.direction}|${parsed.amount}|${it.uppercase(Locale.ROOT)}"
        } ?: "${sender.uppercase(Locale.ROOT)}|$timestamp|${body.trim()}"
        return UUID.nameUUIDFromBytes(identity.toByteArray(Charsets.UTF_8)).toString()
    }

    // Regex patterns for Indian Bank SMSes (HDFC, ICICI, SBI, Axis, PNB, Kotak, UPI, Paytm, PhonePe, Cred)
    private val DEBIT_PATTERNS = listOf(
        Pattern.compile("(?i)(?:debited|sent|paid|spent|txn of|withdrawn|vpa)\\s*(?:by|for|of|with)?\\s*(?:rs\\.?|inr|₹)\\s*([0-9,]+(?:\\.[0-9]{1,2})?)"),
        Pattern.compile("(?i)(?:rs\\.?|inr|₹)\\s*([0-9,]+(?:\\.[0-9]{1,2})?)\\s*(?:has\\s+been\\s+|is\\s+)?(?:debited|spent|paid|withdrawn|transferred)"),
    )

    private val CREDIT_PATTERNS = listOf(
        Pattern.compile("(?i)payment\\s+(?:of\\s+)?(?:rs\\.?|inr|₹)\\s*([0-9,]+(?:\\.[0-9]{1,2})?).{0,80}?(?:received|credited|acknowledged)"),
        Pattern.compile("(?i)(?:credited|received|deposited|added)\\s*(?:by|for|of|with)?\\s*(?:rs\\.?|inr|₹)\\s*([0-9,]+(?:\\.[0-9]{1,2})?)"),
        Pattern.compile("(?i)(?:rs\\.?|inr|₹)\\s*([0-9,]+(?:\\.[0-9]{1,2})?)\\s*(?:has\\s+been\\s+|is\\s+)?(?:credited|received|deposited)"),
    )

    private val BANK_PATTERN = Pattern.compile("(?i)\\b(HDFC|ICICI|SBI|AXIS|KOTAK|PNB|BOB|CANARA|PAYTM|IDFC|YES|INDUSIND)\\b")
    // \b matters: without it the "to" inside ordinary prose ("...wish to inform
    // you that Rs 514...") matches and the payee becomes "inform you that Rs".
    private val PAYEE_PATTERN = Pattern.compile("(?i)\\b(?:to|at|info|vpa)\\s+([A-Za-z0-9\\s\\.\\@\\-\\_]+?)(?:\\s+on|\\s+ref|\\s+avail|\\s+bal|\\s+bal:|\\.|\\,|$)")

    /** First words that mean the capture is English prose, not a merchant. */
    private val PAYEE_STOP_WORDS = setOf(
        "inform", "you", "your", "yours", "help", "know", "the", "a", "an", "be",
        "is", "are", "was", "has", "have", "had", "this", "that", "these", "those",
        "we", "our", "us", "it", "its", "avail", "get", "view", "check", "make",
        "see", "use", "enjoy", "grow", "earn", "save", "claim", "apply", "opt",
        "continue", "confirm", "verify", "update", "download", "click", "read",
        "learn", "discover", "explore", "ensure", "keep", "stay", "find", "start",
        "date", "and", "or", "for", "with", "from", "all", "any", "more", "new",
    )

    private fun isProse(candidate: String): Boolean {
        if (candidate.contains("@")) return false // a VPA is always a real payee
        val first = candidate.trim().substringBefore(' ')
            .trim { !it.isLetterOrDigit() }
            .lowercase(Locale.ROOT)
        return first in PAYEE_STOP_WORDS
    }
    private val REF_PATTERN = Pattern.compile("(?i)\\b(?:upi\\s*ref(?:erence)?|ref(?:erence)?|rrn|txn\\s*(?:id|no|number)|transaction\\s*(?:id|no|number))\\s*\\.?\\s*(?:(?:no|number|id)\\.?\\s*)?[:#-]?\\s*([A-Za-z0-9][A-Za-z0-9/-]{5,})")
    private val EXTRA_REFERENCE = Regex("(?i)\\b(?:utr(?:\\s*(?:no|number))?\\.?\\s*[:#-]?\\s*([A-Za-z0-9]{10,35})|upi[/:-](?:p2[ap]/)?([0-9]{12})(?![0-9]))")

    // Keep old identities discoverable when a new parser learns a previously missed reference.
    fun legacyReference(body: String): String? {
        val matcher = REF_PATTERN.matcher(body.replace("\n", " "))
        return if (matcher.find()) matcher.group(1)?.takeIf { value -> value.any { it.isDigit() } } else null
    }

    fun parse(body: String, sender: String = ""): ParsedSmsTxn? {
        val cleanBody = body.replace("\n", " ").trim()
        val lowerSender = sender.lowercase(Locale.ROOT)

        // Ignore non-transactional OTP / promo SMSes
        if (cleanBody.lowercase(Locale.ROOT).contains("otp") || cleanBody.lowercase(Locale.ROOT).contains("login code")) {
            return null
        }

        var amount: Double? = null
        var direction: String? = null

        // Check Debit
        for (pattern in DEBIT_PATTERNS) {
            val matcher = pattern.matcher(cleanBody)
            if (matcher.find()) {
                val amtStr = matcher.group(1)?.replace(",", "") ?: continue
                amount = amtStr.toDoubleOrNull()
                if (amount != null && amount > 0) {
                    direction = "debit"
                    break
                }
            }
        }

        // Check Credit if not debit
        if (direction == null) {
            for (pattern in CREDIT_PATTERNS) {
                val matcher = pattern.matcher(cleanBody)
                if (matcher.find()) {
                    val amtStr = matcher.group(1)?.replace(",", "") ?: continue
                    amount = amtStr.toDoubleOrNull()
                    if (amount != null && amount > 0) {
                        direction = "credit"
                        break
                    }
                }
            }
        }

        if (amount == null || direction == null) {
            return null
        }

        // Detect Bank
        var bank = "Bank"
        val bankMatcher = BANK_PATTERN.matcher(cleanBody.uppercase(Locale.ROOT))
        if (bankMatcher.find()) {
            bank = bankMatcher.group(1) ?: "Bank"
        } else if (lowerSender.contains("hdfc")) bank = "HDFC"
        else if (lowerSender.contains("icici")) bank = "ICICI"
        else if (lowerSender.contains("sbi")) bank = "SBI"
        else if (lowerSender.contains("axis")) bank = "AXIS"
        else if (lowerSender.contains("kotak")) bank = "KOTAK"
        else if (lowerSender.contains("canara") || lowerSender.contains("canbnk")) bank = "CANARA"

        // Detect Payee
        var payee = "Transaction"
        val payeeMatcher = PAYEE_PATTERN.matcher(cleanBody)
        if (payeeMatcher.find()) {
            val rawPayee = payeeMatcher.group(1)?.trim() ?: ""
            if (rawPayee.length > 2 &&
                !rawPayee.lowercase(Locale.ROOT).startsWith("ac") &&
                !rawPayee.lowercase(Locale.ROOT).startsWith("a/c") &&
                !isProse(rawPayee)
            ) {
                payee = rawPayee.take(40)
            }
        }

        // Detect Ref Number
        val refNo = legacyReference(cleanBody) ?: EXTRA_REFERENCE.find(cleanBody)?.let { match ->
            match.groupValues.drop(1).firstOrNull { it.isNotEmpty() && it.any(Char::isDigit) }
        }

        return ParsedSmsTxn(
            amount = amount,
            direction = direction,
            bank = bank,
            payee = payee,
            refNo = refNo,
            isTransfer = CardTransfers.isRepayment(cleanBody, direction),
        )
    }
}
