package com.khata.app.sms

import com.khata.app.data.SyncOutboxTest
import kotlinx.coroutines.runBlocking
import org.junit.Assert.*
import org.junit.Test
import java.time.Instant

class SmsRequestTest {
    private val mandate = "HP Telecom India Limited has requested Rs. 129600.00. " +
        "Login to Kotak App> BHIM UPI> UPI Mandate> Pending to approve (https://kotak.com/KBANKT/BAUPI)."
    private val request = "UPI mandate request received for Rs.129600.00. Pending to approve. Kotak Bank."
    private val sender = "AD-KOTAKB"

    @Test fun theReportedMandateApprovalRequestIsNotATransaction() {
        for (address in listOf("", sender, "VM-KOTAKB-P")) {
            assertNull(address, SmsParser.parse(mandate, address))
            assertNull(address, SmsParser.parse(mandate.uppercase(), address))
        }
    }

    @Test fun receivedRequestsAndPendingApprovalsNeverBecomeCredits() {
        for (body in listOf(
            request,
            "Payment request for Rs.129600.00 received from HP Telecom. Approve in Kotak UPI.",
            "Request received for INR 129600.00 in Kotak BHIM UPI. Pending approval.",
            "Rs.129600.00 credited pending approval of the UPI mandate. Kotak Bank.",
            "UPI collect request of Rs.129600.00 received. Accept or decline in Kotak App.",
            "UPI request of Rs. 129600.00 has been received. Approve in Kotak App.",
        )) assertNull(body, SmsParser.parse(body, sender))
    }

    @Test fun scheduledAndFutureDebitsDoNotBecomeCompletedPayments() {
        for (body in listOf(
            "Your UPI autopay will be debited INR 129600.00 tomorrow from Kotak A/c XX1234.",
            "The mandate would be paid Rs.129600.00 once approved. Kotak Bank.",
        )) assertNull(body, SmsParser.parse(body, sender))
    }

    @Test fun completedMandatePaymentsAndRefundsStillImport() {
        for ((body, direction) in listOf(
            "Rs.129600.00 has been debited from Kotak A/c XX1234 towards the IPO mandate. Ref 123456789012" to "debit",
            "Kotak UPI mandate executed: debited Rs.129600.00 from A/c XX1234." to "debit",
            "Rs.129600.00 credited to Kotak A/c XX1234 as a mandate refund." to "credit",
            "Your payment request for Rs.129600.00 has been paid from Kotak A/c XX1234." to "debit",
            "Rs.129600.00 debited from Kotak A/c XX1234 as requested." to "debit",
        )) {
            val parsed = SmsParser.parse(body, sender)!!
            assertEquals(body, 129600.0, parsed.amount, 0.0)
            assertEquals(body, direction, parsed.direction)
        }
    }

    @Test fun aRequestElsewhereInTheMessageDoesNotHideOrReplaceARealPosting() {
        val credit = "Rs.500.00 credited to Kotak A/c XX1234. Ref 123456789012."
        for (body in listOf("$request $credit", "$credit $request", "$mandate $credit",
            "UPI mandate request received for Rs.129600.00\n$credit")) {
            val parsed = SmsParser.parse(body, sender)!!
            assertEquals(body, 500.0, parsed.amount, 0.0)
            assertEquals(body, "credit", parsed.direction)
        }
        assertEquals(500.0, SmsParser.parse("Rs.500.00 debited from Kotak A/c XX1234; $request", sender)!!.amount, 0.0)
    }

    @Test fun anUnchangedPreviouslyImportedRequestCanBeExcludedOnRescan() = runBlocking {
        val timestamp = Instant.parse("2026-10-24T04:00:00Z").toEpochMilli()
        val previous = SmsParser.previouslyImportedNonTransaction(request, sender)!!
        val old = SmsImport.transaction(previous, request, sender, timestamp, timestamp + 1000)
            .copy(rev = 7, baseRev = 7, dirty = false, pendingOp = null)
        val dao = SyncOutboxTest.MemoryDao(listOf(old))
        assertTrue(SmsImport.excludeNonTransaction(dao, request, sender, timestamp, timestamp + 2000))
        assertEquals(old.copy(deleted = true, dirty = true, pendingOp = "DELETE"), dao.rows.values.single())
        assertFalse(SmsImport.excludeNonTransaction(dao, request, sender, timestamp, timestamp + 2000))
    }

    @Test fun aRenamedRequestOrAnUnverifiableEntryIsPreserved() = runBlocking {
        val timestamp = Instant.parse("2026-10-24T04:00:00Z").toEpochMilli()
        val previous = SmsParser.previouslyImportedNonTransaction(request, sender)!!
        val imported = SmsImport.transaction(previous, request, sender, timestamp, timestamp + 1000)
            .copy(rev = 7, baseRev = 7, dirty = false, pendingOp = null)
        for (row in listOf(imported.copy(description = "Reviewed title"), imported.copy(smsSource = null))) {
            val dao = SyncOutboxTest.MemoryDao(listOf(row))
            assertFalse(SmsImport.excludeNonTransaction(dao, request, sender, timestamp, timestamp + 2000))
            assertEquals(row, dao.rows.values.single())
        }
        // The exact reported wording was already rejected by older parser rules:
        // do not fabricate an old transaction to delete based on amount alone.
        assertNull(SmsParser.previouslyImportedNonTransaction(mandate, sender))
    }
}
