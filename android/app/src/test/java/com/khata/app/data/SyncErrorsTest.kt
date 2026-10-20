package com.khata.app.data

import okhttp3.ResponseBody.Companion.toResponseBody
import org.junit.Assert.*
import org.junit.Test
import retrofit2.HttpException
import retrofit2.Response
import java.io.IOException

class SyncErrorsTest {
    private fun httpError(code: Int, body: String) = HttpException(Response.error<Any>(code, body.toResponseBody()))

    @Test fun validationErrorsIdentifyTheFieldWithoutEchoingFinancialData() {
        val error = httpError(422, "Failed to deserialize: [2].txn_date: invalid value SECRET_BANK_TEXT")
        val message = error.syncErrorMessage()
        assertTrue(message.contains("transaction date"))
        assertTrue(message.contains("entry 3"))
        assertTrue(message.contains("422"))
        assertFalse(message.contains("SECRET_BANK_TEXT"))
    }

    @Test fun unknownValidationErrorsDoNotExposeResponseContents() {
        val message = httpError(422, "PRIVATE_SMS_BODY").syncErrorMessage()
        assertTrue(message.contains("422"))
        assertTrue(message.contains("remain queued"))
        assertFalse(message.contains("PRIVATE_SMS_BODY"))
    }

    @Test fun authenticationAndNetworkFailuresStayActionable() {
        assertTrue(httpError(401, "").syncErrorMessage().contains("Sign in again"))
        assertTrue(httpError(403, "").syncErrorMessage().contains("server address"))
        assertTrue(IOException("private network detail").syncErrorMessage().contains("Check your connection"))
    }
}
