package com.khata.app.sms

data class SmsImportProgress(
    val running: Boolean = false,
    val phase: String = "",
    val scanned: Int = 0,
    val total: Int = 0,
    val matched: Int = 0,
    val saved: Int = 0,
    val message: String = "",
    val failed: Boolean = false,
)
