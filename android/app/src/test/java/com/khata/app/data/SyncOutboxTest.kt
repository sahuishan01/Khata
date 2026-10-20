package com.khata.app.data

import com.khata.app.api.*
import com.google.gson.Gson
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import org.junit.Assert.*
import org.junit.Test
import java.io.IOException
import java.lang.reflect.Proxy
import java.util.UUID
import java.time.LocalDate
import kotlin.coroutines.Continuation
import kotlin.coroutines.intrinsics.COROUTINE_SUSPENDED

class SyncOutboxTest {
    private class MemoryDao(initial: List<LocalTransaction>) : TransactionDao {
        val rows = initial.associateBy { it.id }.toMutableMap()
        override fun getAllFlow(): Flow<List<LocalTransaction>> = MutableStateFlow(rows.values.toList())
        override suspend fun getAll() = rows.values.filterNot { it.deleted }
        override suspend fun getByClientId(clientId: String) = rows.values.find { it.clientId == clientId }
        override fun getFiltered(category: String?) = getAllFlow()
        override suspend fun upsert(txn: LocalTransaction) { rows[txn.id] = txn }
        override suspend fun upsertAll(txns: List<LocalTransaction>) { txns.forEach { upsert(it) } }
        override suspend fun softDelete(id: String) { rows[id]?.let { upsert(it.copy(deleted = true, dirty = true, pendingOp = "delete")) } }
        override suspend fun getOutbox() = rows.values.filter { it.dirty && it.pendingOp != null }
        override fun pendingCountFlow(): Flow<Int> = MutableStateFlow(rows.values.count { it.dirty && it.pendingOp != null })
        override suspend fun maxRev() = rows.values.maxOfOrNull { it.rev }
        override suspend fun clearAll() { rows.clear() }
        override suspend fun clearLegacyServerCache() { rows.entries.removeAll { !it.value.dirty && it.value.pendingOp == null && it.value.clientId.isEmpty() } }
    }

    private fun api(push: (List<SyncPushOp>) -> SyncPushResult): KhataApi = Proxy.newProxyInstance(
        KhataApi::class.java.classLoader, arrayOf(KhataApi::class.java),
    ) { _, method, args ->
        check(method.name == "syncPush")
        @Suppress("UNCHECKED_CAST")
        val ops = args!![0] as List<SyncPushOp>
        @Suppress("UNCHECKED_CAST")
        val continuation = args.last() as Continuation<SyncPushResult>
        // Retrofit resumes suspend calls with failures rather than throwing a
        // checked IOException directly through Java's InvocationHandler.
        CoroutineScope(continuation.context).launch {
            continuation.resumeWith(runCatching { push(ops) })
        }
        COROUTINE_SUSPENDED
    } as KhataApi

    private fun queued(clientId: String = UUID.randomUUID().toString()) = LocalTransaction(
        clientId = clientId, description = "Test payment", amount = 100.0, bank = "HDFC",
        bankRef = "123456789012", txnDate = "2026-10-20", valueDate = "2026-10-20",
        dirty = true, pendingOp = "CREATE",
    )

    @Test fun failedLegacyQueueCanRetryWithoutScanningAnyNewSms() = runBlocking {
        val original = queued("sms_a123b456c789d012")
        val dao = MemoryDao(listOf(original))
        val attempted = mutableListOf<String>()
        var offline = true
        val outbox = SyncOutbox(dao, api { ops ->
            attempted += ops.single().clientId
            assertEquals("HDFC", ops.single().bank)
            assertEquals("123456789012", ops.single().bankRef)
            assertEquals("2026-10-20", ops.single().txnDate)
            if (offline) throw IOException("offline")
            SyncPushResult(ops.map { SyncAccepted(it.clientId, 1) }, emptyList())
        })
        try { outbox.push(); fail("Expected network failure") } catch (_: IOException) { }
        val retained = dao.getOutbox().single()
        assertEquals(original.id, retained.id)
        assertEquals(retained.clientId, UUID.fromString(retained.clientId).toString())
        offline = false
        assertTrue(outbox.push().isEmpty())
        assertTrue(dao.getOutbox().isEmpty())
        assertEquals(attempted[0], attempted[1])
        assertEquals(1, dao.rows.size)
        outbox.push()
        assertEquals(2, attempted.size)
    }

    @Test fun lostAcknowledgementRetriesWithTheSameIdentity() = runBlocking {
        val dao = MemoryDao(listOf(queued()))
        val server = mutableSetOf<String>()
        var first = true
        val outbox = SyncOutbox(dao, api { ops ->
            server += ops.map { it.clientId }
            if (first) { first = false; throw IOException("response lost") }
            SyncPushResult(ops.map { SyncAccepted(it.clientId, 1) }, emptyList())
        })
        try { outbox.push(); fail("Expected lost response") } catch (_: IOException) { }
        assertEquals(1, dao.getOutbox().size)
        outbox.push()
        assertEquals(1, server.size)
        assertTrue(dao.getOutbox().isEmpty())
    }

    @Test fun partialServerConfirmationDoesNotReportSuccess() = runBlocking {
        val dao = MemoryDao(listOf(queued(), queued()))
        val outbox = SyncOutbox(dao, api { ops -> SyncPushResult(listOf(SyncAccepted(ops.first().clientId, 1)), emptyList()) })
        try { outbox.push(); fail("Expected incomplete confirmation") } catch (_: IllegalStateException) { }
        assertEquals(1, dao.getOutbox().size)
        assertEquals(2, dao.rows.size)
    }

    @Test fun uploadsLargeQueueInBatchesAndLeavesNoPendingRows() = runBlocking {
        val dao = MemoryDao(List(405) { queued() })
        val sizes = mutableListOf<Int>()
        val outbox = SyncOutbox(dao, api { ops ->
            sizes += ops.size
            SyncPushResult(ops.map { SyncAccepted(it.clientId, 1) }, emptyList())
        })
        assertTrue(outbox.push().isEmpty())
        assertEquals(listOf(200, 200, 5), sizes)
        assertTrue(dao.getOutbox().isEmpty())
        assertEquals(405, dao.rows.size)
    }

    @Test fun editDuringUploadRemainsQueued() = runBlocking {
        val original = queued()
        val dao = MemoryDao(listOf(original))
        val outbox = SyncOutbox(dao, api { ops ->
            dao.rows[original.id] = original.copy(notes = "Edited during upload")
            SyncPushResult(ops.map { SyncAccepted(it.clientId, 2) }, emptyList())
        })
        outbox.push()
        val pending = dao.getOutbox().single()
        assertEquals("Edited during upload", pending.notes)
        assertEquals(2L, pending.baseRev)
    }

    @Test fun legacyLiveSmsWithAnEmptyTransactionDateDoesNotBlockTheBatch() = runBlocking {
        val legacy = queued("sms_a123b456c789d012").copy(txnDate = "")
        val normal = queued().copy(txnDate = "2026-10-18", valueDate = "2026-10-20")
        val dao = MemoryDao(listOf(legacy, normal))
        var uploaded = 0
        SyncOutbox(dao, api { ops ->
            val json = Gson().toJsonTree(ops).asJsonArray
            assertEquals(2, json.size())
            json.forEach { item ->
                val row = item.asJsonObject
                UUID.fromString(row["client_id"].asString)
                LocalDate.parse(row["txn_date"].asString)
                LocalDate.parse(row["value_date"].asString)
            }
            assertEquals("2026-10-20", json[0].asJsonObject["txn_date"].asString)
            assertEquals("2026-10-18", json[1].asJsonObject["txn_date"].asString)
            assertEquals("2026-10-20", json[1].asJsonObject["value_date"].asString)
            uploaded = ops.size
            SyncPushResult(ops.map { SyncAccepted(it.clientId, 1) }, emptyList())
        }).push()
        assertEquals(2, uploaded)
        assertTrue(dao.getOutbox().isEmpty())
        assertEquals(2, dao.rows.size)
    }

    @Test fun missingValueDateUsesTheKnownTransactionDate() = runBlocking {
        val dao = MemoryDao(listOf(queued().copy(valueDate = "  ")))
        SyncOutbox(dao, api { ops ->
            assertEquals("2026-10-20", ops.single().valueDate)
            SyncPushResult(ops.map { SyncAccepted(it.clientId, 1) }, emptyList())
        }).push()
        assertTrue(dao.getOutbox().isEmpty())
    }

    @Test fun absentDatesAreOmittedInsteadOfSerializedAsInvalidEmptyStrings() = runBlocking {
        val dao = MemoryDao(listOf(queued().copy(txnDate = "", valueDate = "  ")))
        SyncOutbox(dao, api { ops ->
            val json = Gson().toJsonTree(ops.single()).asJsonObject
            assertFalse(json.has("txn_date"))
            assertFalse(json.has("value_date"))
            SyncPushResult(ops.map { SyncAccepted(it.clientId, 1) }, emptyList())
        }).push()
    }

    @Test fun javaUuidShorthandIsCanonicalizedForTheServer() = runBlocking {
        val dao = MemoryDao(listOf(queued("1-1-1-1-1")))
        SyncOutbox(dao, api { ops ->
            assertEquals("00000001-0001-0001-0001-000000000001", ops.single().clientId)
            SyncPushResult(ops.map { SyncAccepted(it.clientId, 1) }, emptyList())
        }).push()
        assertTrue(dao.getOutbox().isEmpty())
        assertEquals(1, dao.rows.size)
    }
}
