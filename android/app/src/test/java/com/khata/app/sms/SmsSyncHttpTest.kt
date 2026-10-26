package com.khata.app.sms

import com.google.gson.Gson
import com.google.gson.JsonArray
import com.khata.app.data.LocalTransaction
import java.util.Collections
import java.util.concurrent.TimeUnit
import kotlinx.coroutines.runBlocking
import okhttp3.OkHttpClient
import okhttp3.mockwebserver.Dispatcher
import okhttp3.mockwebserver.MockResponse
import okhttp3.mockwebserver.MockWebServer
import okhttp3.mockwebserver.RecordedRequest
import org.junit.Assert.*
import org.junit.Test

class SmsSyncHttpTest {
    private val id = "00000000-0000-4000-8000-000000000011"

    private fun row(clientId: String) = mapOf(
        "id" to clientId, "client_id" to clientId, "rev" to 1, "base_rev" to 1,
        "txn_date" to "2026-10-26", "value_date" to "2026-10-26",
        "description" to "Synthetic transaction", "notes" to "INR 500 paid",
        "amount" to 500, "direction" to "debit", "bank" to "Bank",
        "category" to "Uncategorized", "is_transfer" to false, "deleted" to false,
    )

    @Test fun pagedHistoryAndBulkUploadsUseTheRealTransactionRoutesAndCapturedOwner() = runBlocking {
        val calls = Collections.synchronizedList(mutableListOf<String>())
        val failures = Collections.synchronizedList(mutableListOf<Throwable>())
        val server = MockWebServer()
        val gson = Gson()
        server.dispatcher = object : Dispatcher() {
            override fun dispatch(request: RecordedRequest): MockResponse {
              return try {
                assertEquals("Bearer captured-token",request.getHeader("Authorization"))
                val url=request.requestUrl!!
                calls += request.method + " " + url.encodedPath
                val result: Any = when (request.method to url.encodedPath) {
                    "GET" to "/api/txns/sync/pull" -> {
                        assertEquals("200",url.queryParameter("limit"))
                        assertEquals("2",url.queryParameter("cursor_version"))
                        if (url.queryParameter("since_rev")=="0")
                            mapOf("txns" to listOf(row(id)),"new_rev" to 7,"has_more" to true)
                        else {
                            assertEquals("7",url.queryParameter("since_rev"))
                            mapOf("txns" to emptyList<Any>(),"new_rev" to 7,"has_more" to false)
                        }
                    }
                    "POST" to "/api/txns/sync/push" -> {
                        val ops = gson.fromJson(request.body.readUtf8(),JsonArray::class.java)
                        assertEquals(1,ops.size())
                        assertEquals(id,ops[0].asJsonObject["client_id"].asString)
                        assertEquals("INR 500 paid",ops[0].asJsonObject["notes"].asString)
                        mapOf("accepted" to listOf(mapOf("client_id" to id,"server_rev" to 2)),"conflicts" to emptyList<Any>())
                    }
                    else -> return MockResponse().setResponseCode(404)
                }
                MockResponse().setHeader("Content-Type","application/json").setBody(gson.toJson(result))
            } catch (failure: Throwable) {
                failures += failure
                MockResponse().setResponseCode(500)
              }
            }
        }
        server.start()
        try {
            val session=SmsSession(id,"captured-owner",server.url("/").toString().trimEnd('/'),"captured-token")
            val other=SmsSession("other","other","https://unused.example.test","other-token")
            val client=SmsDecisionClient({other},OkHttpClient.Builder().readTimeout(3,TimeUnit.SECONDS).build())
            val history=client.history(session)
            assertEquals(7L,history.cursor)
            assertEquals(setOf(id),history.rows.keys)
            val saved=client.saveBatch(session,listOf(LocalTransaction(clientId=id,notes="INR 500 paid",amount=500.0)))
            assertEquals(id,saved.accepted.single().clientId)
            assertEquals(2L,saved.accepted.single().serverRev)
            assertEquals(listOf("GET /api/txns/sync/pull","GET /api/txns/sync/pull","POST /api/txns/sync/push"),calls)
            assertTrue(failures.toString(),failures.isEmpty())
        } finally { server.shutdown() }
    }

    @Test fun aMissingUploadRouteIsActionableAndDoesNotEchoTheResponseBody() = runBlocking {
        val server=MockWebServer()
        server.enqueue(MockResponse().setResponseCode(404).setBody("Private response text must not appear in errors"))
        server.start()
        try {
            val session=SmsSession(id,"owner",server.url("/").toString().trimEnd('/'),"synthetic-token")
            val client=SmsDecisionClient({session},OkHttpClient())
            try { client.history(session); fail("Expected a missing-route failure") }
            catch (failure: SmsDecisionFailure) {
                assertFalse(failure.retry)
                assertTrue(failure.message.orEmpty().contains("endpoint"))
                assertFalse(failure.message.orEmpty().contains("Private response"))
            }
        } finally { server.shutdown() }
    }
}
