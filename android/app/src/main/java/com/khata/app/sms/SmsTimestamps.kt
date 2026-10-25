package com.khata.app.sms

/** DATE_SENT is the SMS-centre timestamp used by SmsMessage.timestampMillis. */
object SmsTimestamps {
    fun fromInbox(arrival: Long, sent: Long): Long =
        sent.takeIf { it > 0 && it <= arrival + 300_000 } ?: arrival
}
