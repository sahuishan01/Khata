package com.khata.app.sms

import com.khata.app.data.SyncOutboxTest
import kotlinx.coroutines.runBlocking
import org.junit.Assert.*
import org.junit.Test
import java.time.Instant

class SmsWordOrderTest {
    private val body = "IMPS INR 42,850.25\nsent from HDFC Bank A/c XX1234 on 04-07-26\nTo A/c xxxxxxxxxx4321\nRef-321654987012\nNot you?Call 18000000000/SMS BLOCK OB to 9000000000"

    @Test fun multilineAmountFirstSentPostingKeepsDateReferenceAndExactNotes() {
        val parsed = SmsParser.parse(body, "AD-UNKNOWN")!!
        assertEquals(42850.25, parsed.amount, 0.0)
        assertEquals("debit", parsed.direction)
        assertEquals("HDFC", parsed.bank)
        assertEquals("321654987012", parsed.refNo)
        assertEquals("Transaction", parsed.payee)
        assertFalse(parsed.isTransfer)
        val timestamp = Instant.parse("2026-07-04T10:00:00Z").toEpochMilli()
        val row = SmsImport.transaction(parsed, body, "AD-UNKNOWN", timestamp, timestamp + 86400000)
        assertEquals("2026-07-04", row.txnDate)
        assertEquals("2026-07-04", row.valueDate)
        assertEquals(body, row.notes)
    }

    @Test fun everyDebitVerbWorksInBothWordOrdersAcrossIssuersAndWhitespace() {
        for (verb in listOf("debited", "sent", "paid", "spent", "withdrawn", "transferred")) {
            for (issuer in listOf("HDFC Bank", "AxisBank", "CanaraBank", "Example Cooperative Bank")) {
                for (space in listOf(" ", "\n", "\r\n")) {
                    for (posting in listOf("$verb${space}INR 42,850.25", "INR 42,850.25${space}$verb")) {
                        val parsed = SmsParser.parse("IMPS $posting from $issuer A/c XX1234 on 04-07-26. Ref-321654987012", "AD-UNKNOWN")
                        assertNotNull("Debit verbs must share both word orders", parsed)
                        assertEquals(42850.25, parsed!!.amount, 0.0)
                        assertEquals("debit", parsed.direction)
                    }
                }
            }
        }
    }

    @Test fun everyCreditVerbWorksInBothWordOrdersWithoutAnIssuerRequirement() {
        for (verb in listOf("credited", "received", "deposited", "added")) {
            for (space in listOf(" ", "\n", "\r\n")) {
                for (posting in listOf("$verb${space}INR 42,850.25", "INR 42,850.25${space}$verb")) {
                    val parsed = SmsParser.parse("$posting to A/c XX1234. Ref-321654987012", "AD-UNKNOWN")
                    assertNotNull("Credit verbs must share both word orders", parsed)
                    assertEquals(42850.25, parsed!!.amount, 0.0)
                    assertEquals("credit", parsed.direction)
                }
            }
        }
    }

    @Test fun pendingFutureAndOfferClausesCannotBecomeSentPostings() {
        for (message in listOf(
            "IMPS INR 42,850.25\nwill be sent from HDFC Bank A/c XX1234",
            "IMPS INR 42,850.25\nsent from HDFC Bank A/c XX1234, pending approval",
            "UPI request received for INR 42,850.25\nsent from HDFC Bank A/c XX1234",
            "Get INR 42,850.25 sent instantly with a new loan.",
            "Please send INR 42,850.25 from A/c XX1234.",
            "Unsent INR 42,850.25 from A/c XX1234.",
            "INR 42,850.25 unsent from A/c XX1234.",
        )) assertNull("Non-posting wording must remain rejected", SmsParser.parse(message, "AD-UNKNOWN"))
    }

    @Test fun thePostingAmountCannotComeFromABalanceOrSecurityFooter() {
        val parsed = SmsParser.parse(body + "\nAvailable balance INR 9,99,999.00")!!
        assertEquals(42850.25, parsed.amount, 0.0)
        assertEquals("321654987012", parsed.refNo)
        assertNull(SmsParser.parse("IMPS transfer notice. Available balance INR 42,850.25. Not you? Call 18000000000."))
    }

    @Test fun repeatingTheImportPreservesItsIdentityAndSavedEdits() = runBlocking {
        val parsed = SmsParser.parse(body)!!
        val timestamp = Instant.parse("2026-07-04T10:00:00Z").toEpochMilli()
        val dao = SyncOutboxTest.MemoryDao(emptyList())
        assertEquals(SmsSaveResult.CREATED, SmsImport.queue(dao, parsed, body, "AD-UNKNOWN", timestamp, timestamp + 1000))
        val old = dao.rows.values.single()
        assertEquals(SmsSaveResult.UNCHANGED, SmsImport.queue(dao, parsed, body, "AD-UNKNOWN", timestamp, timestamp + 2000))
        assertEquals(old.clientId, SmsParser.clientId(parsed, body, "AD-OTHER", timestamp + 10000))
        val edited = old.copy(description = "My transfer", notes = "My notes")
        dao.upsert(edited)
        assertEquals(SmsSaveResult.UNCHANGED, SmsImport.queue(dao, parsed, body, "AD-UNKNOWN", timestamp, timestamp + 3000))
        assertEquals(edited, dao.rows.values.single())
    }
}
