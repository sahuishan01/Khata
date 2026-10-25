package com.khata.app.sms

import org.junit.Assert.*
import org.junit.Test

class SmsDecisionTest {
    private val version="laya:test:sms-choice-v1"

    @Test fun decisionsUseOriginalMoneyAndPaymentMethod() {
        val sms="IMPS INR 42,850.25\nsent from HDFC Bank A/c XX1234 on 04-07-26\nTo A/c xxxxxxxxxx4321\nRef-321654987012\nNot you?Call 18000000000/SMS BLOCK OB to 9000000000"
        val parsed=SmsParser.fromDecision(sms,"HDFCBK","debit",version)!!
        assertEquals(42850.25,parsed.amount,0.0)
        assertEquals("IMPS",parsed.paymentMethod)
        assertEquals(version,parsed.classifierVersion)
        assertNull(SmsParser.fromDecision(sms,"HDFCBK","credit",version))
        val row=SmsImport.transaction(parsed,sms,"HDFCBK",1791972000000,1791972000000)
        assertEquals(sms,row.notes)
        assertEquals("IMPS",SmsSource.decode(row.smsSource)!!.paymentMethod)
    }

    @Test fun aPositiveModelCannotTurnNoticesIntoMoney() {
        for (sms in listOf(
            "Get Rs.276000 credited in minutes with a personal loan. Apply now.",
            "INR 500 will be debited from your HDFC account tomorrow.",
            "UPI request received for INR 500. Please authorize the request in your bank app.",
            "Available balance in your HDFC account is INR 5000.",
            "IPO application received for INR 14820. Approve the collect request to block funds.",
            "IMPS transfer of INR 500 failed. No amount was debited.",
            "Verification code 123456 for INR 500 spent at SHOP.",
            "INR 500 is the price of this item.",
            "INR 500.123 paid to SHOP.",
            "INR 1,2,3 paid to SHOP.",
            "Payment of INR 500 will be received in your account tomorrow.",
        )) for (direction in listOf("debit","credit")) {
            assertNull(sms,SmsParser.fromDecision(sms,"BANK",direction,version))
        }
    }

    @Test fun amountAmbiguityAndCreditNeedExplicitDecisions() {
        assertNull(SmsParser.fromDecision("Purchase completed: INR 100 and INR 200.","BANK","debit",version))
        assertNull(SmsParser.fromDecision("INR 100 paid to SHOP. INR 200 paid to STORE.","BANK","debit",version))
        assertEquals(500.0,SmsParser.fromDecision("Purchase completed for INR 500.","BANK","debit",version)!!.amount,0.0)
        val credit="Payment of INR 73123.45 has been received towards your Axis Bank Credit Card XX4321 on 15-10-26"
        assertNull(SmsParser.fromDecision(credit,"AXIS","debit",version))
        assertEquals("credit",SmsParser.fromDecision(credit,"AXIS","credit",version)!!.direction)
        val dr="Acct XXXX1234 Dr. INR 73,123.45 on 15/10/26 to CRED Club; UPI: 123456789012; Bal INR 40,033.94"
        assertEquals(73123.45,SmsParser.fromDecision(dr,"CANBNK","debit",version)!!.amount,0.0)
    }

    @Test fun ownerScopeSurvivesTokenRefreshButChangesWithAccountAndServer() {
        fun token(id:String,suffix:String)="head."+java.util.Base64.getUrlEncoder().withoutPadding()
            .encodeToString("{\"sub\":\"$id\"}".toByteArray())+".$suffix"
        val a="dfccf726-6e67-4f3a-9964-5708adc349e3"
        val b="fb11c664-18df-4dbf-885c-43f6cbd169ad"
        val first=smsSession(token(a,"first"),"https://example.test/")!!
        assertEquals(first.owner,smsSession(token(a,"second"),"https://example.test")!!.owner)
        assertNotEquals(first.owner,smsSession(token(b,"second"),"https://example.test")!!.owner)
        assertNotEquals(first.owner,smsSession(token(a,"first"),"https://other.test")!!.owner)
        assertNull(smsSession("invalid","https://example.test"))
    }
}
