package com.khata.app.sms

import java.util.Locale
import java.util.regex.Pattern

data class ParsedSmsTxn(
    val amount: Double,
    val direction: String, // "debit" or "credit"
    val bank: String,
    val payee: String,
    val refNo: String? = null
)

object SmsParser {
    // Regex patterns for Indian Bank SMSes (HDFC, ICICI, SBI, Axis, PNB, Kotak, UPI, Paytm, PhonePe, Cred)
    private val DEBIT_PATTERNS = listOf(
        Pattern.compile("(?i)(?:debited|sent|paid|spent|txn of|withdrawn|vpa)\\s*(?:by|for|of)?\\s*(?:rs\\.?|inr)\\s*([0-9,]+(?:\\.[0-9]{1,2})?)"),
        Pattern.compile("(?i)(?:rs\\.?|inr)\\s*([0-9,]+(?:\\.[0-9]{1,2})?)\\s*(?:debited|spent|paid|withdrawn|transferred)"),
    )

    private val CREDIT_PATTERNS = listOf(
        Pattern.compile("(?i)(?:credited|received|deposited|added)\\s*(?:by|for|of)?\\s*(?:rs\\.?|inr)\\s*([0-9,]+(?:\\.[0-9]{1,2})?)"),
        Pattern.compile("(?i)(?:rs\\.?|inr)\\s*([0-9,]+(?:\\.[0-9]{1,2})?)\\s*(?:credited|received|deposited)"),
    )

    private val BANK_PATTERN = Pattern.compile("(?i)\\b(HDFC|ICICI|SBI|AXIS|KOTAK|PNB|BOB|CANARA|PAYTM|IDFC|YES|INDUSIND)\\b")
    private val PAYEE_PATTERN = Pattern.compile("(?i)(?:to|at|info|vpa)\\s+([A-Za-z0-9\\s\\.\\@\\-\\_]+?)(?:\\s+on|\\s+ref|\\s+avail|\\s+bal|\\s+bal:|\\.|\\,|$)")
    private val REF_PATTERN = Pattern.compile("(?i)(?:ref|upi ref|txn id|rrn)\\s*:?\\s*([A-Za-z0-9]+)")

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
        else if (lowerSender.contains("axis")) bank = "Axis"

        // Detect Payee
        var payee = "Transaction"
        val payeeMatcher = PAYEE_PATTERN.matcher(cleanBody)
        if (payeeMatcher.find()) {
            val rawPayee = payeeMatcher.group(1)?.trim() ?: ""
            if (rawPayee.length > 2 && !rawPayee.lowercase(Locale.ROOT).startsWith("ac") && !rawPayee.lowercase(Locale.ROOT).startsWith("a/c")) {
                payee = rawPayee.take(40)
            }
        }

        // Detect Ref Number
        var refNo: String? = null
        val refMatcher = REF_PATTERN.matcher(cleanBody)
        if (refMatcher.find()) {
            refNo = refMatcher.group(1)
        }

        return ParsedSmsTxn(
            amount = amount,
            direction = direction,
            bank = bank,
            payee = payee,
            refNo = refNo
        )
    }
}
