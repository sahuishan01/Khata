package com.khata.app.sms

import org.junit.Assert.*
import org.junit.Test
import java.time.Instant
import java.util.TimeZone

class SmsDatesTest {
    private val received = Instant.parse("2026-10-20T04:00:00Z").toEpochMilli()
    private val queued = Instant.parse("2026-10-23T04:00:00Z").toEpochMilli()

    @Test fun bodyDateWinsOverMessageAndQueueDates() {
        for (date in listOf("18-10-2026", "18/10/26", "18.10.2026", "18-Oct-26", "18 Oct 2026", "18Oct26", "2026-10-18")) {
            assertEquals(date, "2026-10-18", SmsDates.resolve("INR 500 debited from HDFC on $date. Ref 1234567890", received, queued))
        }
    }

    @Test fun messageDateWinsWhenBodyHasNoValidTransactionDate() {
        for (body in listOf("Rs 500 debited", "Rs 500 debited on 31/02/2026", "Payment due on 01/11/2026")) {
            assertEquals("2026-10-20", SmsDates.resolve(body, received, queued))
        }
    }

    @Test fun missingMessageTimestampUsesPersistedQueueDate() {
        for (timestamp in listOf(null, 0L, -1L)) {
            assertEquals("2026-10-23", SmsDates.resolve("Rs 500 debited", timestamp, queued))
            assertEquals("2026-10-18", SmsDates.resolve("Rs 500 debited on 18/10/26", timestamp, queued))
        }
    }

    @Test fun datesUseIndianMidnightEvenOnDevicesInOtherTimezones() {
        val previous = TimeZone.getDefault()
        try {
            TimeZone.setDefault(TimeZone.getTimeZone("America/Los_Angeles"))
            assertEquals("2026-10-20", SmsDates.resolve("Rs 500 debited", Instant.parse("2026-10-19T19:00:00Z").toEpochMilli(), queued))
        } finally { TimeZone.setDefault(previous) }
    }

    @Test fun dueAndStatementDatesDoNotOverrideTheTransactionDate() {
        assertEquals("2026-10-18", SmsDates.resolve("Statement date 01/10/2026. Rs 500 debited on 18/10/26. Payment due on 01/11/2026", received, queued))
        assertEquals("2026-10-20", SmsDates.resolve("Rs 500 debited. Available balance as on 01/11/26", received, queued))
    }
}
