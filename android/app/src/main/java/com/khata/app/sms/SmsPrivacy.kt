package com.khata.app.sms

import java.util.Locale

/** Runs before persistence, upload or model classification, including inbox scans. */
object SmsPrivacy {
    private val authenticationCode = Regex(
        "(?i)\\b(?:one[ -]?time[ -]?(?:password|passcode|code)|login[ -]?code|verification[ -]?code|authentication[ -]?code)\\b"
    )

    fun containsOtp(body: String): Boolean = body.lowercase(Locale.ROOT).contains("otp") ||
        authenticationCode.containsMatchIn(body)
}
