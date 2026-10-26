package com.khata.app.sms

import org.junit.Assert.*
import org.junit.Test

class SmsCandidateFilterTest {
    @Test fun reportedFormatsRemainEligibleAcrossIssuersAndWordOrders() {
        for (body in listOf(
            "IMPS INR 42,850.25\nsent from HDFC Bank A/c XX1234 on 04-07-26\nTo A/c xxxxxxxxxx4321\nRef-321654987012",
            "Acct XXXX1234 Dr. INR 73,123.45 on 15/10/26 to CRED Club; Bal INR 40,033.94",
            "Payment of INR 73123.45 has been received towards your Axis Bank Credit Card XX4321",
            "Novel issuer: Rs.500 moved from your account", "INR\n1,00,000.00 sent", "500.00 INR paid", "₹500", "Rs: 500",
            "INR\u00a0500 paid", "Rs.\u202f500 sent",
        )) assertTrue(body,SmsCandidateFilter.isCandidate(body))
    }

    @Test fun personalNumericTextsWithoutMoneyDoNotLeaveThePhone() {
        for (body in listOf("Meet me at 7", "Your parcel 123456 is arriving", "Call 9000000000", "Happy birthday", "Offer 50% off", "123456", "RSVP 500", "The server has 500 errors", "INR exchange rate available"))
            assertFalse(body,SmsCandidateFilter.isCandidate(body))
    }

    @Test fun privacyAndSizeGuardsApplyBeforeMonetaryEligibility() {
        for (body in listOf("OTP 123456 for INR 500", "INR 500 paid. Never share your OTP", "Verification code 123456 for Rs 500", " ", "INR 500 "+"a".repeat(16000)))
            assertFalse(SmsCandidateFilter.isCandidate(body))
    }

    @Test fun theGateDoesNotReplaceTheModelsTransactionDecision() {
        assertTrue(SmsCandidateFilter.isCandidate("Get INR 500 cashback when you apply"))
        assertTrue(SmsCandidateFilter.isCandidate("Unknown posting wording with INR 500"))
    }

    @Test fun aLargeOrdinaryInboxDoesNotBecomeALargeUploadQueue() {
        val inbox=List(12000) { "Personal message number $it" }+List(300) { "INR ${it+1}.00 sent from BANK account XX1234" }
        assertEquals(12300,inbox.size)
        assertEquals(300,inbox.count(SmsCandidateFilter::isCandidate))
    }

    @Test fun aFinishedUploadDoesNotCompletePendingOrReviewSms() {
        assertFalse(SmsDecisionSummary(pending=12100,accepted=1).isComplete())
        assertFalse(SmsDecisionSummary(review=3,accepted=1).isComplete())
        assertTrue(SmsDecisionSummary(accepted=300,filtered=12000).isComplete())
    }
}
