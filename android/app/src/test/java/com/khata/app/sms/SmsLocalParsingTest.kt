package com.khata.app.sms

import org.junit.Assert.*
import org.junit.Test

class SmsLocalParsingTest {
    @Test fun originalAlertFormatsAreParsedWithoutModelDecisions() {
        val messages=listOf(
            "IMPS INR 42,850.25\nsent from HDFC Bank A/c XX1234 on 04-07-26\nTo A/c xxxxxxxxxx4321\nRef-321654987012",
            "Acct XXXX1234 Dr. INR 73,123.45 on 15/10/26 to CRED Club; Bal INR 40,033.94",
            "Payment of INR 73123.45 has been received towards your Axis Bank Credit Card XX4321",
        )
        for ((index,body) in messages.withIndex()) {
            val parsed=SmsParser.parse(body,"BANK")!!
            assertEquals(if(index==2) "credit" else "debit",parsed.direction)
            assertEquals(if(index==0) 42850.25 else 73123.45,parsed.amount,0.0)
            assertNull(parsed.classifierVersion)
            val row=SmsImport.transaction(parsed,body,"BANK",1791972000000,1791972000000)
            assertEquals(body,row.notes)
        }
        assertEquals("IMPS",SmsParser.parse(messages[0],"BANK")!!.paymentMethod)
    }

    @Test fun ordinaryAndUncompletedSmsNeverEnterTheUploadQueue() {
        for(body in listOf(
            "Meet me at 7", "Your parcel 123456 is arriving", "OTP 123456 for INR 500",
            "Get Rs.276000 credited in minutes with a personal loan. Apply now.",
            "INR 500 will be debited from your HDFC account tomorrow.",
            "UPI request received for INR 500. Please authorize the request in your bank app.",
            "Available balance in your HDFC account is INR 5000.",
            "IPO application received for INR 14820. Approve the collect request to block funds.",
            "IMPS transfer of INR 500 failed. No amount was debited.",
            "Verification code 123456 for INR 500 spent at SHOP.",
            "INR 500 is the price of this item.",
            "INR 500.123 paid to SHOP.", "INR 1,2,3 paid to SHOP.",
            "Payment of INR 500 will be received in your account tomorrow.",
        )) assertNull(body,SmsParser.parse(body,"BANK"))
    }

    @Test fun onlyRecognizedTransactionsMatchALargeInbox() {
        val messages=List(12000) { "Personal reminder number $it" }+
            List(300) { "INR ${it+1}.00 sent from BANK account XX1234" }
        assertEquals(300,messages.count { SmsParser.parse(it,"BANK")!=null })
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
