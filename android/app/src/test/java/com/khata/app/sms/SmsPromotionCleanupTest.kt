package com.khata.app.sms

import com.google.gson.Gson
import com.khata.app.data.LocalTransaction
import com.khata.app.data.SyncOutboxTest
import kotlinx.coroutines.runBlocking
import org.junit.Assert.*
import org.junit.Test
import java.time.Instant

class SmsPromotionCleanupTest {
    private val body = "Get Rs.276000 credited in minutes with a Personal Loan on your Kotak Credit Card. " +
        "Tap to apply: https://1.kotak.bank.in/KOTAKB/OFFER T&C apply"
    private val sender = "AD-KOTAKB"
    private val timestamp = Instant.parse("2026-10-24T04:00:00Z").toEpochMilli()
    private val queued = timestamp + 1000
    private val oldParsed = ParsedSmsTxn(276000.0, "credit", "KOTAK", "Transaction")

    private fun imported() = SmsImport.transaction(oldParsed, body, sender, timestamp, queued)
        .copy(rev = 7, baseRev = 7, dirty = false, pendingOp = null)

    @Test fun rescanExcludesOnlyTheProvenUnchangedImportAndRetainsItsIdentityAndFields() = runBlocking {
        val old = imported()
        val dao = SyncOutboxTest.MemoryDao(listOf(old))
        assertTrue(SmsImport.excludePromotion(dao, body, sender, timestamp, queued))
        assertEquals(old.copy(deleted = true, dirty = true, pendingOp = "DELETE"), dao.rows.values.single())
        assertEquals(7L, dao.getOutbox().single().baseRev)
        assertFalse(SmsImport.excludePromotion(dao, body, sender, timestamp, queued))
        assertEquals(1, dao.rows.size)
    }

    @Test fun aFalseImportStillQueuedForItsFirstUploadBecomesATombstone() = runBlocking {
        val old = imported().copy(rev = 0, baseRev = 0, dirty = true, pendingOp = "CREATE")
        val dao = SyncOutboxTest.MemoryDao(listOf(old))
        assertTrue(SmsImport.excludePromotion(dao, body, sender, timestamp, queued))
        assertEquals(old.clientId, dao.getOutbox().single().clientId)
        assertTrue(dao.getOutbox().single().deleted)
        assertEquals(0L, dao.getOutbox().single().baseRev)
    }

    @Test fun editsAndUnverifiableLegacyRowsAreNeverAutomaticallyExcluded() = runBlocking {
        val old = imported()
        val source = SmsSource.decode(old.smsSource)!!
        val protected = listOf(
            old.copy(amount = 1000.0), old.copy(direction = "debit"),
            old.copy(description = "Reviewed loan credit"), old.copy(category = "Loan"),
            old.copy(notes = "Reviewed by user"), old.copy(bank = "OTHER"),
            old.copy(bankRef = "123456789012"), old.copy(balance = 1000.0), old.copy(isTransfer = true),
            old.copy(txnDate = "2026-10-23"), old.copy(valueDate = "2026-10-23"),
            old.copy(dirty = true, pendingOp = "UPDATE"), old.copy(smsSource = null),
            old.copy(smsSource = "{}"), old.copy(deleted = true),
            old.copy(smsSource = Gson().toJson(source.copy(bodyHash = "a".repeat(64)))),
            old.copy(smsSource = Gson().toJson(source.copy(receivedAt = timestamp - 1000))),
        )
        for (row in protected) {
            val dao = SyncOutboxTest.MemoryDao(listOf(row))
            assertFalse(row.toString(), SmsImport.excludePromotion(dao, body, sender, timestamp, queued))
            assertEquals(row, dao.rows.values.single())
        }
    }

    @Test fun aRealCreditWithAnOfferFooterAndOtherMessagesRemainUntouched() = runBlocking {
        val real = "Rs.1500 credited to Kotak A/c XX1234. $body"
        val parsed = SmsParser.parse(real, sender)!!
        val row = SmsImport.transaction(parsed, real, sender, timestamp, queued)
        val dao = SyncOutboxTest.MemoryDao(listOf(row))
        assertFalse(SmsImport.excludePromotion(dao, real, sender, timestamp, queued))
        assertFalse(SmsImport.excludePromotion(dao, body, sender, timestamp, queued))
        assertEquals(row, dao.rows.values.single())
    }

    @Test fun aConcurrentEditInvalidatesTheExclusionSnapshot() = runBlocking {
        val snapshot = imported()
        val edited = snapshot.copy(notes = "Keep this record")
        val dao = SyncOutboxTest.MemoryDao(listOf(edited))
        assertFalse(dao.excludeSmsIfUnchanged(snapshot))
        assertEquals(edited, dao.rows.values.single())
    }
}
