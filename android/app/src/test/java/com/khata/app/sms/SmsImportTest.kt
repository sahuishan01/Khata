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

    @Test fun titleAndNotesAreSwappedWithoutChangingTheExactSms() {
        val message = "  INR 500 debited from HDFC A/c XX1234 to SHOP.\n" + "Bank information. ".repeat(40) + "  "
        val parsed = SmsParser.parse(message, "VM-HDFCBK")!!
        val row = SmsImport.transaction(parsed, message, "VM-HDFCBK", timestamp, queued)
        assertEquals("Auto-captured via SMS (${parsed.bank})", row.description)
        assertEquals(message, row.notes)
        assertTrue(row.notes.length > 500)
        assertEquals(parsed.payee, SmsSource.decode(row.smsSource)!!.parsedPayee)
        assertEquals(message, Gson().fromJson(Gson().toJson(row), com.khata.app.data.LocalTransaction::class.java).notes)
    }

    @Test fun rescanBackfillsProvenGeneratedDescriptionsAndPreservesFullUserEdits() {
        val parsed = SmsParser.parse(body, "VM-HDFCBK")!!
        val fresh = SmsImport.transaction(parsed, body, "VM-HDFCBK", timestamp, queued)
        val legacy = fresh.copy(description = "Credit card bill payment", notes = "Auto-captured via SMS (${parsed.bank})",
            rev = 1, baseRev = 1, dirty = false, pendingOp = null)
        val repaired = SmsImport.repair(legacy, parsed, body, timestamp)
        assertEquals(fresh.description, repaired.description)
        assertEquals(body, repaired.notes)
        val unverified = legacy.copy(smsSource = null)
        val unverifiedRepair = SmsImport.repair(unverified, parsed, body, timestamp)
        assertEquals(unverified.description, unverifiedRepair.description)
        assertEquals(unverifiedRepair, SmsImport.repair(unverifiedRepair, parsed, body, timestamp))
        val edited = legacy.copy(description = "My title", amount = 321.0, direction = "credit", bank = "AXIS",
            bankRef = "", balance = -100.0, category = "Refund", isTransfer = false,
            txnDate = "2026-10-21", valueDate = "2026-10-22",
            smsSource = Gson().toJson(SmsSource.decode(legacy.smsSource)!!.copy(userEdited = true)))
        assertEquals(edited, SmsImport.repair(edited, parsed, body, timestamp))
        assertTrue(SmsSource.decode(edited.smsSource)!!.userEdited == true)
    }

    @Test fun aScanCannotSaveOverAnEditMadeAfterItsLookup() = runBlocking {
        val parsed = SmsParser.parse(body, "VM-HDFCBK")!!
        val original = SmsImport.transaction(parsed, body, "VM-HDFCBK", timestamp, queued)
        val dao = SyncOutboxTest.MemoryDao(listOf(original))
        val edited = original.copy(description = "Just edited", amount = 900.0)
        dao.upsert(edited)
        assertFalse(dao.saveSmsIfUnchanged(original, original.copy(description = "Parsed later")))
        assertEquals(edited, dao.getByClientId(original.clientId))
        val legacyMetadata = """{"parser_version":1,"date_source":"message","parsed_date":"2026-10-20"}"""
        val decoded = SmsSource.decode(legacyMetadata)!!
        assertNull(decoded.userEdited)
        assertFalse(Gson().toJson(decoded).contains("user_edited"))
    }

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

    @Test fun rescanKeepsACustomTitleWhileRepairingCardEvidence() = kotlinx.coroutines.runBlocking {
        val parsed = SmsParser.parse(body, "VM-HDFCBK")!!
        val original = SmsImport.transaction(parsed, body, "VM-HDFCBK", timestamp, queued).copy(
            description = "My October card repayment", isTransfer = false,
            notes = "Auto-captured via SMS (${parsed.bank})",
            category = "Uncategorized", txnDate = "2026-10-20", valueDate = "2026-10-20",
            rev = 9, baseRev = 9, dirty = false, pendingOp = null,
        )
        val dao = com.khata.app.data.SyncOutboxTest.MemoryDao(listOf(original))
        assertEquals(SmsSaveResult.UPDATED, SmsImport.queue(dao, parsed, body, "VM-HDFCBK", timestamp, queued))
        val repaired = dao.rows.values.single()
        assertEquals(original.description, repaired.description)
        assertEquals(original.clientId, repaired.clientId)
        assertEquals(original.amount, repaired.amount, 0.0)
        assertTrue(repaired.isTransfer)
        assertEquals("2026-10-18", repaired.valueDate)
        assertEquals(SmsSaveResult.UNCHANGED, SmsImport.queue(dao, parsed, body, "VM-HDFCBK", timestamp, queued))
    }

    @Test fun rescanSwapsV23FieldsOnceAndPreservesCustomNotesAndTitles() {
        val parsed = SmsParser.parse(body, "VM-HDFCBK")!!
        val current = SmsImport.transaction(parsed, body, "VM-HDFCBK", timestamp, queued)
        val v23 = current.copy(description = body, notes = "Auto-captured via SMS (${parsed.bank})")
        val repaired = SmsImport.repair(v23, parsed, body, timestamp)
        assertEquals(current.description, repaired.description)
        assertEquals(body, repaired.notes)
        assertEquals(v23.clientId, repaired.clientId)
        assertEquals(v23.amount, repaired.amount, 0.0)
        assertEquals(repaired, SmsImport.repair(repaired, parsed, body, timestamp))
        for (edited in listOf(v23.copy(description = "My purchase"),
            v23.copy(notes = "My notes"), current.copy(notes = "My notes"),
            current.copy(description = "My title"), v23.copy(smsSource = null))) {
            val after = SmsImport.repair(edited, parsed, body, timestamp)
            assertEquals(edited.description, after.description)
            assertEquals(edited.notes, after.notes)
        }
    }

    @Test fun rescanRepairsOldFutureDatesAndServerQueueFallbacks() {
        val text = "INR 500 credited to Canara account on 02/12/26. Ref 123456789012"
        val parsed = SmsParser.parse(text, "AD-CANBNK")!!
        val old = SmsImport.transaction(parsed, text, "AD-CANBNK", timestamp, queued).copy(
            txnDate = "2026-12-02", valueDate = "2026-12-02", smsSource = null,
            description = text, notes = "Auto-captured via SMS (${parsed.bank})",
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
                rev = 1, baseRev = 1, dirty = false, pendingOp = null, smsSource = null,
                description = text, notes = "Auto-captured via SMS (${parsed.bank})")
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

    @Test fun aSecondAlertWithoutADateDoesNotReplaceAKnownBodyDate() {
        val parsed = SmsParser.parse(body, "VM-HDFCBK")!!
        val old = SmsImport.transaction(parsed, body, "VM-HDFCBK", timestamp, queued).copy(
            rev = 1, baseRev = 1, dirty = false, pendingOp = null)
        val reminder = "INR 500 debited from HDFC account. Ref 1234567890"
        val repaired = SmsImport.repair(old, SmsParser.parse(reminder, "VM-HDFCBK")!!, reminder, timestamp)
        assertEquals("2026-10-18", repaired.txnDate)
        assertEquals("2026-10-18", repaired.valueDate)
    }

    @Test fun anExistingReferenceIdentityDoesNotHideTheOldUnreferencedAlert() = runBlocking {
        val text = "INR 500 debited from Kotak A/c XX1234 to SHOP. UTR: 123456789012"
        val parsed = SmsParser.parse(text, "AD-KOTAKB")!!
        val old = SmsImport.transaction(parsed.copy(refNo = null), text, "AD-KOTAKB", timestamp, queued)
            .copy(rev = 1, baseRev = 1, smsSource = null, dirty = false, pendingOp = null,
                description = text, notes = "Auto-captured via SMS (${parsed.bank})")
        val canonical = SmsImport.transaction(parsed, text, "VM-KOTAKB", timestamp, queued)
            .copy(rev = 1, baseRev = 1, dirty = false, pendingOp = null)
        val dao = SyncOutboxTest.MemoryDao(listOf(old, canonical))
        SmsImport.queue(dao, parsed, text, "AD-KOTAKB", timestamp, queued)
        assertEquals(2, dao.rows.size)
        assertEquals("123456789012", dao.getByClientId(old.clientId)!!.bankRef)
        assertTrue(dao.getByClientId(old.clientId)!!.dirty)
        assertEquals(canonical, dao.getByClientId(canonical.clientId))
    }
    @Test fun bankCardSettlementAndReceiptShareACardKeyWithoutTreatingCredPurchasesAsRepayments() {
        val receipt = "Payment of INR 7777 received for HDFC Bank Card XX1234 on 15/10/26"
        val confirmation = "Payment of INR 7777 credited to HDFC Bank Card ending 1234 on 15/10/26"
        for (text in listOf(receipt, confirmation)) assertTrue(CardTransfers.isRepayment(text, "credit"))
        val first = SmsSource.from(receipt, timestamp, queued, "credit", "HDFC")
        val later = SmsSource.from(confirmation, timestamp, queued, "credit", "HDFC")
        assertEquals("card_receipt", first.paymentRole)
        assertEquals("card_confirmation", later.paymentRole)
        assertNotNull(first.cardAccountKey)
        assertEquals(first.cardAccountKey, later.cardAccountKey)
        assertNotEquals(first.cardAccountKey, SmsSource.from(confirmation.replace("1234", "9876"), timestamp, queued, "credit", "HDFC").cardAccountKey)
        assertFalse(CardTransfers.isRepayment("INR 7777 debited to CRED Club", "debit"))
        assertFalse(CardTransfers.isRepayment("Payment refund credited to HDFC Bank Card XX1234", "credit"))
        assertFalse(CardTransfers.isRepayment("Payment received for HDFC debit card XX1234", "credit"))
    }

}
