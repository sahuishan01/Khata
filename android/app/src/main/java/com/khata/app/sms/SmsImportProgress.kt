package com.khata.app.sms

data class SmsImportProgress(
    val running: Boolean = false,
    val phase: String = "",
    val scanned: Int = 0,
    val total: Int = 0,
    val matched: Int = 0,
    val saved: Int = 0,
    val updated: Int = 0,
    val alreadySaved: Int = 0,
    val excludedNonTransactions: Int = 0,
    val serverCheckFailed: Boolean = false,
    val message: String = "",
    val failed: Boolean = false,
)
