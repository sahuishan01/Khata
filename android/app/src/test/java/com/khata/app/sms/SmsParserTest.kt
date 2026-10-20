package com.khata.app.sms

import org.junit.Assert.*
import org.junit.Test
import java.util.UUID
import com.google.gson.Gson
import com.khata.app.api.TxnRow
import com.khata.app.api.toLocal

class SmsParserTest {
    @Test fun referencePrefixesNeverBecomeTransactionIdentifiers() {
        listOf("Ref No 123456789012", "UPI Ref: 123456789012", "Ref.no.123456789012", "RRN 123456789012").forEach { ref ->
            val parsed = SmsParser.parse("Rs.500.00 debited from HDFC account. $ref", "VM-HDFCBK")!!
            assertEquals(ref, "123456789012", parsed.refNo)
        }
        assertNull(SmsParser.parse("Rs.500.00 debited from HDFC account. Ref not available", "VM-HDFCBK")!!.refNo)
    }
    @Test fun serverImportsKeepStableIdsAndTransactionDetails() {
        val row = Gson().fromJson("""{"id":"00000000-0000-4000-8000-000000000001","txn_date":"2026-10-19","value_date":"2026-10-20","description":"Transfer","amount":250,"direction":"credit","category":"Income","bank":"CANARA","bank_ref":"1234567890","balance":5000,"notes":"Imported","is_transfer":false,"rev":7,"base_rev":6,"deleted":false}""", TxnRow::class.java)
        val local = row.toLocal()
        assertEquals(row.id, local.id)
        assertEquals(row.id, local.clientId)
        assertEquals(local.id, row.toLocal().id)
        assertEquals(7L, local.baseRev)
        assertEquals("2026-10-19", local.txnDate)
        assertEquals("CANARA", local.bank)
        assertEquals("1234567890", local.bankRef)
        assertEquals(5000.0, local.balance!!, 0.0)
    }
    @Test fun parsesTransactionsFromTheFourBanks() {
        val samples = listOf(
            Triple("Rs.500.00 debited from HDFC A/c XX1234 at SHOP on 20-10-2026 Ref 1234567890", "VM-HDFCBK", "HDFC"),
            Triple("INR 1,250.00 is debited from your A/c at SHOP. Ref 2234567890", "AD-KOTAKB", "KOTAK"),
            Triple("Your Axis Bank account is debited by ₹450.00 at SHOP Ref 3234567890", "VM-AXISBK", "AXIS"),
            Triple("A/c XX1234 credited with INR 2000.00 Ref 4234567890", "AD-CANBNK", "CANARA"),
        )
        samples.forEach { (body, sender, bank) ->
            val parsed = SmsParser.parse(body, sender)
            assertNotNull("Expected a transaction from $bank", parsed)
            assertEquals(bank, parsed!!.bank.uppercase())
            assertTrue(parsed.amount > 0)
            assertEquals(if (bank == "CANARA") "credit" else "debit", parsed.direction)
        }
    }

    @Test fun ignoresOtpAndNonTransactionMessages() {
        assertNull(SmsParser.parse("OTP 123456 for payment of Rs.500.00", "VM-HDFCBK"))
        assertNull(SmsParser.parse("Your account statement is available", "VM-HDFCBK"))
    }

    @Test fun inboxAndLiveCaptureShareAValidStableIdentity() {
        val body = "Rs.500.00 debited from HDFC account. Ref 1234567890"
        val parsed = SmsParser.parse(body, "VM-HDFCBK")!!
        val live = SmsParser.clientId(parsed, body, "VM-HDFCBK", 1000)
        val inbox = SmsParser.clientId(parsed, body, "AD-HDFCBK", 2000)
        assertEquals(live, inbox)
        assertEquals(live, UUID.fromString(live).toString())
        assertNotEquals(live, SmsParser.clientId(parsed.copy(refNo = "2234567890"), body, "VM-HDFCBK", 1000))
    }

    @Test fun separateUnreferencedTransactionsAreNotCollapsed() {
        val body = "Rs.500.00 debited from HDFC account"
        val parsed = SmsParser.parse(body, "VM-HDFCBK")!!
        assertNotEquals(SmsParser.clientId(parsed, body, "VM-HDFCBK", 1000), SmsParser.clientId(parsed, body, "VM-HDFCBK", 2000))
    }
}
