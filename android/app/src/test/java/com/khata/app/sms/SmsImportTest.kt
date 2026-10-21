package com.khata.app.sms

import org.junit.Assert.*
import org.junit.Test
import java.time.Instant
import com.google.gson.Gson
import com.khata.app.data.SyncOutboxTest
import kotlinx.coroutines.runBlocking

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

    @Test fun rescanRepairsOldFutureDatesAndServerQueueFallbacks() {
        val text = "INR 500 credited to Canara account on 02/12/26. Ref 123456789012"
        val parsed = SmsParser.parse(text, "AD-CANBNK")!!
        val old = SmsImport.transaction(parsed, text, "AD-CANBNK", timestamp, queued).copy(
            txnDate = "2026-12-02", valueDate = "2026-12-02", smsSource = null,
            rev = 1, baseRev = 1, dirty = false, pendingOp = null)
        val repaired = SmsImport.repair(old, parsed, text, timestamp)
        assertEquals("2026-10-20", repaired.txnDate)
        assertEquals("2026-10-20", repaired.valueDate)
        assertEquals(old.clientId, repaired.clientId)
        val fallback = old.copy(txnDate = "2026-10-23", valueDate = "2026-10-23",
            smsSource = Gson().toJson(SmsSource(parserVersion = 0, dateSource = "queue", parsedDate = "2026-10-23",
                previousTxnDate = "2026-12-02", previousValueDate = "2026-12-02")))
        assertEquals("2026-10-20", SmsImport.repair(fallback, parsed, text, timestamp).valueDate)
        assertEquals(repaired, SmsImport.repair(repaired, parsed, text, timestamp))
    }

    @Test fun newlyRecognizedReferencesRepairTheExistingIdentityInsteadOfCreatingAnotherRow() = runBlocking {
        for (suffix in listOf("UTR: 123456789012", "UPI/123456789012/SHOP")) {
            val text = "INR 500 debited from Kotak A/c XX1234 to SHOP. $suffix"
            val parsed = SmsParser.parse(text, "AD-KOTAKB")!!
            assertEquals("123456789012", parsed.refNo)
            val oldParsed = parsed.copy(refNo = null)
            val old = SmsImport.transaction(oldParsed, text, "AD-KOTAKB", timestamp, queued).copy(
                rev = 1, baseRev = 1, dirty = false, pendingOp = null, smsSource = null)
            val dao = SyncOutboxTest.MemoryDao(listOf(old))
            assertEquals(SmsSaveResult.UPDATED, SmsImport.queue(dao, parsed, text, "AD-KOTAKB", timestamp, queued))
            val row = dao.rows.values.single()
            assertEquals(old.clientId, row.clientId)
            assertEquals("123456789012", row.bankRef)
            assertEquals(SmsSaveResult.UNCHANGED, SmsImport.queue(dao, parsed, text, "AD-KOTAKB", timestamp, queued))
            // An excluded pre-upgrade alert must stay excluded under the new reference parser too.
            dao.upsert(row.copy(deleted = true, dirty = false, pendingOp = null))
            assertEquals(SmsSaveResult.UNCHANGED, SmsImport.queue(dao, parsed, text, "AD-KOTAKB", timestamp, queued))
            assertEquals(1, dao.rows.size)
            assertTrue(dao.rows.values.single().deleted)
        }
    }

    @Test fun onlyAnchoredMessagesCarryAutomaticDuplicateEvidence() {
        val generic = SmsSource.from("INR 500 debited from HDFC account", timestamp, queued)
        assertNull(generic.eventKey)
        val balanced = SmsSource.from("INR 500 debited from HDFC A/c XX1234 to SHOP. Avl bal INR 7500", timestamp, queued)
        assertEquals(balanced.bodyHash, balanced.eventKey)
        assertEquals(64, balanced.eventKey!!.length)
        assertEquals(balanced.bodyHash, SmsSource.from("  INR 500 debited from HDFC A/c XX1234 to SHOP.\nAvl bal INR 7500  ", timestamp, queued).bodyHash)
        assertNull(SmsSource.from("INR 500 debited from HDFC A/c XX1234 to SHOP", timestamp, queued).eventKey)
        assertNotNull(SmsSource.from("INR 500 debited from HDFC A/c XX1234 on 20/10/26 09:30:00", timestamp, queued).eventKey)
    }
}
