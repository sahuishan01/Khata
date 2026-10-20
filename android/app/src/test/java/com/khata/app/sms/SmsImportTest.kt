package com.khata.app.sms

import org.junit.Assert.*
import org.junit.Test
import java.time.Instant

class SmsImportTest {
    private val timestamp = Instant.parse("2026-10-20T04:00:00Z").toEpochMilli()
    private val queued = timestamp + 3 * 86_400_000
    private val body = "Rs.500 debited from HDFC account towards HDFC credit card on 18/10/26. Ref 1234567890"

    @Test fun bothSidesOfACardRepaymentAreTransfers() {
        val messages = listOf(
            body,
            "Your payment of INR 500 has been received towards your HDFC credit card on 18/10/26. Ref 2234567890",
            "INR 500 credited towards Axis credit card payment on 18/10/26. Ref 3234567890",
        )
        for (message in messages) {
            val parsed = SmsParser.parse(message, "VM-HDFCBK")!!
            val row = SmsImport.transaction(parsed, message, "VM-HDFCBK", timestamp, queued)
            assertTrue(message, row.isTransfer)
            assertEquals("Transfer", row.category)
            assertEquals("2026-10-18", row.txnDate)
            assertEquals("2026-10-18", row.valueDate)
        }
    }

    @Test fun purchasesRefundsAndFeesRemainEconomicActivity() {
        for (description in listOf(
            "Rs 500 spent using HDFC credit card at AMAZON",
            "Rs 500 credited as refund to your HDFC credit card",
            "Rs 500 credited as cashback to your credit card",
            "Credit card payment fee Rs 500 debited",
            "Credit card payment at AMAZON",
            "Credit card payment failed",
            "Credit card payment due on 01/11/26",
            "Insurance settlement credited Rs 500",
        )) assertFalse(description, CardTransfers.isRepayment(description, "debit"))
    }

    @Test fun rescanningRepairsAnExistingSmsWithoutChangingItsIdentityOrOtherEdits() {
        val parsed = SmsParser.parse(body, "VM-HDFCBK")!!
        val original = SmsImport.transaction(parsed, body, "VM-HDFCBK", timestamp, queued).copy(
            txnDate = "2026-10-20", valueDate = "2026-10-20", isTransfer = false,
            category = "Uncategorized", dirty = false, pendingOp = null, rev = 7, baseRev = 7,
        )
        val repaired = SmsImport.repair(original, parsed, body, timestamp)
        assertEquals(original.id, repaired.id)
        assertEquals(original.clientId, repaired.clientId)
        assertEquals(7L, repaired.baseRev)
        assertEquals(original.updatedAt, repaired.updatedAt)
        assertEquals("2026-10-18", repaired.valueDate)
        assertTrue(repaired.isTransfer)
        assertTrue(repaired.dirty)
        assertEquals("UPDATE", repaired.pendingOp)
        assertEquals(repaired, SmsImport.repair(repaired, parsed, body, timestamp))
        val edited = original.copy(txnDate = "2026-10-16", valueDate = "2026-10-16")
        assertEquals("2026-10-16", SmsImport.repair(edited, parsed, body, timestamp).txnDate)
    }
}
