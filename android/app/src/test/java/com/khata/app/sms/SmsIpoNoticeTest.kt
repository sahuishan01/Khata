package com.khata.app.sms

import com.khata.app.data.SyncOutboxTest
import kotlinx.coroutines.runBlocking
import org.junit.Assert.*
import org.junit.Test
import java.time.Instant

class SmsIpoNoticeTest {
    // Reported wording with neutral application/VPA identifiers.
    private val notice = "Dear ABCXXXXX0G IPO Jio Platforms Limited : App no GROWW123example " +
        "UPI ID investor@okaxis for Rs 107424.00 received. Please check for UPI-Mandate " +
        "collect request for blocking of funds- NSEIL"
    private val sender = "AD-NSEIL"

    @Test fun anIpoApplicationReceivedNoticeIsNotACredit() {
        for (address in listOf("", sender, "VM-NSEIL-P")) {
            assertNull(address, SmsParser.parse(notice, address))
            assertNull(address, SmsParser.parse(notice.uppercase(), address))
        }
    }

    @Test fun applicationAcknowledgementsAreNotMoneyReceived() {
        for (body in listOf(
            notice.replace("App no ", "Application number ").replace("Rs 107424.00", "Rs. 1,07,424.00"),
            notice.replace("App no ", "App no. "),
            notice.replace("UPI ID", "\nUPI ID"),
            "Your IPO application for INR 107424.00 received. Approve the mandate in your UPI app.",
            "Application no. EXAMPLE123 for Rs 107424.00 received by the exchange.",
            "IPO bid number EXAMPLE123 for Rs 107424.00 received.",
            "Application received for Rs 107424.00. Please approve the UPI request.",
        )) assertNull(body, SmsParser.parse(body, sender))
    }

    @Test fun fundBlockingIsNotAPostedCredit() {
        for (body in listOf(
            "UPI: Rs 107424.00 received for blocking of funds for IPO.",
            "Rs 107424.00 received; funds will be blocked on approval of this mandate.",
            "Funds blocked: Rs 107424.00 received for IPO application EXAMPLE123.",
        )) assertNull(body, SmsParser.parse(body, sender))
    }

    @Test fun completedIpoPaymentsAndRefundsRemainTransactions() {
        for ((body, direction) in listOf(
            "IPO application no. EXAMPLE123: Rs 107424.00 has been debited from Kotak A/c XX1234." to "debit",
            "Rs 107424.00 debited from Kotak A/c XX1234 for IPO application EXAMPLE123." to "debit",
            "IPO application EXAMPLE123: Rs 107424.00 credited to Kotak A/c XX1234 as refund." to "credit",
            "Refund received for Rs 107424.00 for IPO application EXAMPLE123." to "credit",
            "Rs 107424.00 received in Kotak A/c XX1234 as an IPO refund." to "credit",
        )) {
            val parsed = SmsParser.parse(body, sender)!!
            assertEquals(body, 107424.0, parsed.amount, 0.0)
            assertEquals(body, direction, parsed.direction)
        }
    }

    @Test fun aMobileApplicationFooterDoesNotHideARealReceipt() {
        for (footer in listOf("Download the Kotak mobile application.",
            "Use the Kotak application for more details.")) {
            val body = "Rs 500.00 received. $footer"
            val parsed = SmsParser.parse(body, "AD-KOTAKB")!!
            assertEquals(500.0, parsed.amount, 0.0)
            assertEquals("credit", parsed.direction)
        }
    }

    @Test fun aSeparateRealPostingKeepsItsOwnAmount() {
        val credit = "Rs 500.00 credited to Kotak A/c XX1234. Ref 123456789012."
        val receipt = "Received Rs 500.00 in Kotak A/c XX1234."
        for (body in listOf("$notice. $credit", "$credit $notice", "$notice\n$credit",
            "$notice; $credit", "$notice. $receipt", "$receipt $notice")) {
            val parsed = SmsParser.parse(body, sender)!!
            assertEquals(body, 500.0, parsed.amount, 0.0)
            assertEquals(body, "credit", parsed.direction)
        }
    }

    @Test fun anUnchangedFalseIpoCreditCanBeExcludedOnRescan() = runBlocking {
        val timestamp = Instant.parse("2026-10-24T04:00:00Z").toEpochMilli()
        val previous = SmsParser.previouslyImportedNonTransaction(notice, sender)!!
        assertEquals(107424.0, previous.amount, 0.0)
        assertEquals("credit", previous.direction)
        val old = SmsImport.transaction(previous, notice, sender, timestamp, timestamp + 1000)
            .copy(rev = 7, baseRev = 7, dirty = false, pendingOp = null)
        val dao = SyncOutboxTest.MemoryDao(listOf(old))
        assertTrue(SmsImport.excludeNonTransaction(dao, notice, sender, timestamp, timestamp + 2000))
        assertEquals(old.copy(deleted = true, dirty = true, pendingOp = "DELETE"), dao.rows.values.single())
        assertFalse(SmsImport.excludeNonTransaction(dao, notice, sender, timestamp, timestamp + 2000))
    }

    @Test fun editedOrUnverifiableIpoEntriesArePreserved() = runBlocking {
        val timestamp = Instant.parse("2026-10-24T04:00:00Z").toEpochMilli()
        val previous = SmsParser.previouslyImportedNonTransaction(notice, sender)!!
        val old = SmsImport.transaction(previous, notice, sender, timestamp, timestamp + 1000)
            .copy(rev = 7, baseRev = 7, dirty = false, pendingOp = null)
        for (row in listOf(old.copy(description = "Reviewed IPO"), old.copy(smsSource = null))) {
            val dao = SyncOutboxTest.MemoryDao(listOf(row))
            assertFalse(SmsImport.excludeNonTransaction(dao, notice, sender, timestamp, timestamp + 2000))
            assertEquals(row, dao.rows.values.single())
        }
    }
}
