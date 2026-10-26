package com.khata.app.sms

import com.google.gson.Gson
import kotlinx.coroutines.runBlocking
import org.junit.Assert.*
import org.junit.Test

class SmsTimestampsTest {
    private val sent=1792477800000L
    private val arrival=sent+1042
    private val body="INR 500 spent using HDFC credit card at SHOP on 20/10/26."
    private val parsed=SmsParser.parse(body,"HDFCBK")!!

    @Test fun liveReceiptAndInboxScanUseTheSameIdentity() {
        assertEquals(sent,SmsTimestamps.fromInbox(arrival,sent))
        val live=SmsParser.clientId(parsed,body,"15550003003",sent)
        val inbox=SmsParser.clientId(parsed,body,"15550003003",SmsTimestamps.fromInbox(arrival,sent))
        assertEquals(live,inbox)
        // Two separate messages with the same text still represent distinct events.
        assertNotEquals(live,SmsParser.clientId(parsed,body,"15550003003",sent+1000))
    }

    @Test fun missingOrInvalidSentTimesUseArrival() {
        for (invalid in listOf(0L,-1L,arrival+300001)) assertEquals(arrival,SmsTimestamps.fromInbox(arrival,invalid))
    }

    @Test fun olderArrivalBasedImportsKeepTheirIdentityAndEdits() = runBlocking {
        for (edited in listOf(false,true)) {
            val original=SmsImport.transaction(parsed,body,"15550003003",arrival,arrival+10000)
            val old=if(edited) original.copy(amount=777.0,description="My title",notes="My notes",
                smsSource=Gson().toJson(SmsSource.decode(original.smsSource)!!.copy(userEdited=true))) else original
            val (previous,row)=SmsImport.prepareWithLookup(parsed,body,"15550003003",sent,arrival+20000,
                alternateTimestamp=arrival) { id -> old.takeIf { it.clientId==id } }
            assertEquals(old,previous);assertEquals(old.clientId,row.clientId)
            assertEquals(old.amount,row.amount,0.0);assertEquals(old.notes,row.notes)
            assertEquals(arrival,SmsSource.decode(row.smsSource)!!.receivedAt)
            if(edited) assertEquals(old,row)
        }
    }

    @Test fun unverifiableOlderImportsRequireReview() = runBlocking {
        val original=SmsImport.transaction(parsed,body,"15550003003",arrival,arrival+10000)
        val old=original.copy(smsSource=Gson().toJson(SmsSource.decode(original.smsSource)!!.copy(bodyHash=null)))
        try {
            SmsImport.prepareWithLookup(parsed,body,"15550003003",sent,arrival+20000,alternateTimestamp=arrival) { id -> old.takeIf { it.clientId==id } }
            fail("Unverified older provenance must not create a duplicate or overwrite the saved row")
        } catch (_:SmsIdentityConflict) { }
    }
}
