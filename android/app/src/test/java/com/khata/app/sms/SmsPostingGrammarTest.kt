package com.khata.app.sms

import com.khata.app.data.SyncOutboxTest
import kotlinx.coroutines.runBlocking
import org.junit.Assert.*
import org.junit.Test
import java.time.Instant

class SmsPostingGrammarTest {
    private val amount = 73123.45
    private val debit = "Dear Customer, Acct XXXX1234 Dr. INR 73,123.45 on 15/10/26 to CRED Club; UPI: 123456789012; Bal INR 40,033.94.Not you?SMS BLOCKUPI to 9000000000-CanaraBank"

    @Test fun accountPostingGrammarWorksAcrossIssuersAndBothDirections() {
        for (issuer in listOf("HDFC Bank", "AxisBank", "CanaraBank", "ICICI Bank", "SBI", "Kotak Bank", "Example Cooperative Bank")) {
            for ((marker, direction) in listOf("Dr" to "debit", "Cr" to "credit")) {
                for (posting in listOf(
                    "Acct XXXX1234 $marker. INR 73,123.45",
                    "A/c XX1234 $marker Rs. 73123.45",
                    "Account No. 12345678 is $marker. ₹73,123.45",
                    "AC XX1234 INR 73,123.45 $marker.",
                )) {
                    val body = "$posting on 15/10/26 to SHOP; UPI: 123456789012; Bal INR 40,033.94 - $issuer"
                    val parsed = SmsParser.parse(body, "AD-UNKNOWN")
                    assertNotNull("Account posting must not depend on issuer", parsed)
                    assertEquals(amount, parsed!!.amount, 0.0)
                    assertEquals(direction, parsed.direction)
                    assertEquals("SHOP", parsed.payee)
                    assertEquals("123456789012", parsed.refNo)
                }
            }
        }
    }

    @Test fun abbreviationsRequireAnAccountPostingAndKeepNonTransactionGuards() {
        for (body in listOf(
            "Dr. Rao consultation costs INR 500.",
            "Available balance INR 500 Dr.",
            "Credit limit INR 500. Debit card is active.",
            "Acct XXXX1234 balance INR 500 Dr.",
            "UPI request received: Acct XXXX1234 Dr. INR 500. Please authorize.",
            "Acct XXXX1234 Dr. INR 500 will be debited tomorrow.",
            "Get Acct XXXX1234 Cr. INR 500 with a new loan.",
        )) assertNull("Non-posting text must remain rejected", SmsParser.parse(body, "AD-UNKNOWN"))
    }

    @Test fun payeeAndReferenceUseGenericSeparators() {
        for (reference in listOf("UPI: 123456789012", "UPI : 123456789012", "UPI/123456789012", "UPI / 123456789012", "UPI-123456789012", "UPI/P2A/123456789012")) {
            val parsed = SmsParser.parse("INR 73,123.45 debited from A/c XX1234 to CRED Club; $reference; Bal INR 40,033.94")!!
            assertEquals("CRED Club", parsed.payee)
            assertEquals("123456789012", parsed.refNo)
        }
        for (reference in listOf("UPI: 123456789", "UPI: 1234567890123")) {
            assertNull(SmsParser.parse("INR 500 debited from A/c XX1234 to SHOP; $reference")!!.refNo)
        }
    }

    @Test fun bankNameSuffixesAndSenderAliasesOnlyIdentifyTheIssuer() {
        for ((issuer, expected) in listOf("HDFCBank" to "HDFC", "AxisBank" to "AXIS", "CanaraBank" to "CANARA", "ICICIBank" to "ICICI")) {
            assertEquals(expected, SmsParser.parse("Acct XX1234 Dr. INR 500 to SHOP; - $issuer")!!.bank)
        }
        assertEquals("CANARA", SmsParser.parse("Acct XX1234 Dr. INR 500 to SHOP;", "AD-CANBNK")!!.bank)
        assertEquals("debit", SmsParser.parse("Acct XX1234 Dr. INR 500 to SHOP;", "AD-UNKNOWN")!!.direction)
    }

    @Test fun cardReceiptRemainsATransferCreditAndAmountsNeverComeFromTheBalance() {
        val receipt = "Payment of INR 73123.45 has been received towards your Axis Bank Credit Card XX4321 on 15-10-26 - Axis Bank"
        val funding = SmsParser.parse(debit)!!
        val credit = SmsParser.parse(receipt)!!
        assertEquals(amount, funding.amount, 0.0)
        assertEquals("debit", funding.direction)
        assertEquals("CANARA", funding.bank)
        assertEquals("CRED Club", funding.payee)
        assertEquals(funding.amount, credit.amount, 0.0)
        assertEquals("credit", credit.direction)
        assertTrue(credit.isTransfer)
    }

    @Test fun rescanningAMissedPostingCreatesOneStableRowAndPreservesEdits() = runBlocking {
        val timestamp = Instant.parse("2026-10-15T10:00:00Z").toEpochMilli()
        val parsed = SmsParser.parse(debit)!!
        val dao = SyncOutboxTest.MemoryDao(emptyList())
        assertEquals(SmsSaveResult.CREATED, SmsImport.queue(dao, parsed, debit, "AD-CANBNK", timestamp, timestamp + 1000))
        val row = dao.rows.values.single()
        assertEquals("2026-10-15", row.txnDate)
        assertEquals("123456789012", row.bankRef)
        assertEquals(debit, row.notes)
        assertEquals("Auto-captured via SMS (CANARA)", row.description)
        assertEquals(SmsSaveResult.UNCHANGED, SmsImport.queue(dao, parsed, debit, "AD-CANBNK", timestamp, timestamp + 2000))
        val edited = row.copy(description = "My card repayment", notes = "My notes")
        dao.upsert(edited)
        assertEquals(SmsSaveResult.UNCHANGED, SmsImport.queue(dao, parsed, debit, "AD-CANBNK", timestamp, timestamp + 3000))
        assertEquals(edited, dao.rows.values.single())

        // Issuer catalogs can improve without replacing an existing identity.
        val previouslyKnown = debit.replace("Dr. INR", "debited INR")
            .replace("UPI: 123456789012", "Ref 123456789012")
        val recognized = SmsParser.parse(previouslyKnown)!!
        val unknownIssuer = recognized.copy(bank = "Bank")
        val legacy = SmsImport.transaction(unknownIssuer, previouslyKnown, "AD-UNKNOWN", timestamp, timestamp + 1000)
        val previousDao = SyncOutboxTest.MemoryDao(listOf(legacy))
        SmsImport.queue(previousDao, recognized, previouslyKnown, "AD-UNKNOWN", timestamp, timestamp + 2000)
        assertEquals(1, previousDao.rows.size)
        assertEquals(legacy.clientId, previousDao.rows.values.single().clientId)
        assertEquals(legacy.notes, previousDao.rows.values.single().notes)
    }
}
