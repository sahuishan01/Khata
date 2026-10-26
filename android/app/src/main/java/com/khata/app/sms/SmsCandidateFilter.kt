package com.khata.app.sms

/** A broad monetary gate, not a transaction decision or a bank-template matcher. */
object SmsCandidateFilter {
    private val currency = Regex("(?i)₹|(?<![A-Za-z])(?:INR|RS)\\.?(?![A-Za-z])")
    private val amountAfter = Regex("^[\\s\\p{Zs}]*[:=]?[\\s\\p{Zs}]*[+-]?[\\s\\p{Zs}]*\\d")

    fun isCandidate(body: String): Boolean {
        if (body.isBlank() || body.length > 16000 || SmsPrivacy.containsOtp(body)) return false
        return currency.findAll(body).any { token ->
            amountAfter.containsMatchIn(body.substring(token.range.last + 1)) ||
                body.substring(0, token.range.first).trimEnd().lastOrNull()?.isDigit() == true
        }
    }
}
