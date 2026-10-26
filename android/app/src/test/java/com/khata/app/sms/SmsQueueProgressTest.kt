package com.khata.app.sms

import org.junit.Assert.*
import org.junit.Test

class SmsQueueProgressTest {
    @Test fun emptyQueueHasNoProgress() {
        val summary = SmsDecisionSummary()
        assertEquals(0L, summary.totalCount())
        assertEquals(0L, summary.finishedCount())
        assertEquals(0, summary.progressPercent())
        assertEquals(0f, summary.progressFraction(), 0f)
        assertFalse(summary.hasMessages())
    }

    @Test fun pendingAndReviewRemainInTheTotalUntilResolved() {
        val summary = SmsDecisionSummary(pending=2, review=1, accepted=4, ignored=2, filtered=1)
        assertEquals(10L, summary.totalCount())
        assertEquals(7L, summary.finishedCount())
        assertEquals(70, summary.progressPercent())
        assertEquals(.7f, summary.progressFraction(), .0001f)
        assertFalse(summary.isComplete())
        val reviewed = summary.copy(review=0, accepted=5)
        assertEquals(summary.totalCount(), reviewed.totalCount())
        assertEquals(80, reviewed.progressPercent())
    }

    @Test fun oneUnfinishedMessageNeverRoundsUpToOneHundredPercent() {
        for (summary in listOf(
            SmsDecisionSummary(pending=1, accepted=12000),
            SmsDecisionSummary(review=1, accepted=12000),
        )) {
            assertEquals(99, summary.progressPercent())
            assertFalse(summary.isComplete())
        }
    }

    @Test fun acceptedIgnoredAndLocallyFilteredMessagesAllFinishWork() {
        val summary = SmsDecisionSummary(accepted=3, ignored=2, filtered=12000)
        assertEquals(12005L, summary.totalCount())
        assertEquals(summary.totalCount(), summary.finishedCount())
        assertEquals(100, summary.progressPercent())
        assertEquals(1f, summary.progressFraction(), 0f)
        assertTrue(summary.isComplete())
    }
}
