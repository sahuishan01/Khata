package com.khata.app.sms

import org.junit.Assert.*
import org.junit.Test

class SmsPrivacyTest {
    @Test fun otpAndAuthenticationCodesAreBlockedEvenWhenMoneyIsMentioned() {
        for (body in listOf(
            "OTP 123456 for INR 500 payment", "Your otp is 123456", "INR 500 debited. Do not share your OTP.",
            "Your one-time password is 123456", "Use one time passcode 123456", "Login code: 123456",
            "Verification code 123456 for payment of INR 500", "Authentication code: 123456",
        )) {
            assertTrue(SmsPrivacy.containsOtp(body))
            assertNull(SmsParser.parse(body))
        }
    }

    @Test fun financialMessagesWithoutAuthenticationCodesCanProceed() {
        for (body in listOf("IMPS INR 500 sent from HDFC A/c XX1234", "INR 500 credited as refund", "Your statement is ready")) {
            assertFalse(SmsPrivacy.containsOtp(body))
        }
    }
}
