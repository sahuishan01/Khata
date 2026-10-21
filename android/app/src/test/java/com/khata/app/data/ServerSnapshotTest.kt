package com.khata.app.data

import com.khata.app.api.*
import com.khata.app.sms.SmsImport
import com.khata.app.sms.SmsParser
import com.khata.app.sms.SmsSaveResult
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import org.junit.Assert.*
import org.junit.Test
import java.io.IOException
import java.lang.reflect.Proxy
import java.time.Instant
import java.util.UUID
import kotlin.coroutines.Continuation
import kotlin.coroutines.intrinsics.COROUTINE_SUSPENDED

class ServerSnapshotTest {
    private class StateDao(initial: SyncState = SyncState(cursor = 99999)) : SyncStateDao {
        var value = initial
        override suspend fun get() = value
        override suspend fun set(state: SyncState) { value = state }
    }

    private fun api(call: (String, Array<out Any?>) -> Any): KhataApi = Proxy.newProxyInstance(
        KhataApi::class.java.classLoader, arrayOf(KhataApi::class.java),
    ) { _, method, args ->
        @Suppress("UNCHECKED_CAST")
        val continuation = args!!.last() as Continuation<Any?>
        CoroutineScope(continuation.context).launch { continuation.resumeWith(runCatching { call(method.name, args) }) }
        COROUTINE_SUSPENDED
    } as KhataApi

    private val timestamp = Instant.parse("2026-10-20T04:00:00Z").toEpochMilli()
    private fun body(index: Int) = "Rs.100 debited from HDFC account to STORE on 18/10/26. Ref ${123456789000L + index}"
    private fun local(index: Int) = SmsImport.transaction(SmsParser.parse(body(index), "VM-HDFCBK")!!,
        body(index), "VM-HDFCBK", timestamp, timestamp).copy(rev = 1, baseRev = 1, dirty = false, pendingOp = null)

    @Test fun clearingServerThenRescanning2961SmsReimportsOnceWithoutDuplicates() = runBlocking {
        val cached = List(2961) { local(it) }
        val dao = SyncOutboxTest.MemoryDao(cached)
        val state = StateDao()
        val server = mutableMapOf<String, SyncPushOp>()
        val api = api { method, args ->
            when (method) {
                "syncPull" -> SyncPullResponse(emptyList(), 0, false)
                "syncPush" -> {
                    @Suppress("UNCHECKED_CAST") val ops = args[0] as List<SyncPushOp>
                    ops.forEach { assertEquals(0L, it.baseRev); server[it.clientId] = it }
                    SyncPushResult(ops.map { SyncAccepted(it.clientId, 1) }, emptyList())
                }
                else -> error(method)
            }
        }
        ServerSnapshot.fetch(api).replaceCache(dao, state)
        assertTrue(dao.rows.isEmpty())
        assertEquals(0L, state.value.cursor)
        for (index in cached.indices) {
            assertEquals(SmsSaveResult.CREATED, SmsImport.queue(dao, SmsParser.parse(body(index), "VM-HDFCBK")!!,
                body(index), "VM-HDFCBK", timestamp, timestamp))
        }
        assertEquals(2961, dao.getOutbox().size)
        assertTrue(SyncOutbox(dao, api).push().isEmpty())
        assertEquals(2961, server.size)
        assertTrue(dao.getOutbox().isEmpty())
        for (index in cached.indices) {
            assertEquals(SmsSaveResult.UNCHANGED, SmsImport.queue(dao, SmsParser.parse(body(index), "VM-HDFCBK")!!,
                body(index), "VM-HDFCBK", timestamp, timestamp))
        }
        assertEquals(2961, dao.rows.size)
        assertEquals(2961, server.size)
        assertTrue(dao.getOutbox().isEmpty())
    }

    @Test fun excludedDuplicateTombstoneSurvivesSnapshotAndSmsRescan() = runBlocking {
        val sms = local(0)
        val dao = SyncOutboxTest.MemoryDao(listOf(sms))
        val excluded = sms.copy(deleted = true, rev = 2, baseRev = 2)
        ServerSnapshot(listOf(excluded), 500).replaceCache(dao, StateDao())
        assertEquals(SmsSaveResult.UNCHANGED, SmsImport.queue(dao,
            SmsParser.parse(body(0), "VM-HDFCBK")!!, body(0), "VM-HDFCBK", timestamp, timestamp))
        assertTrue(dao.getByClientId(sms.clientId)!!.deleted)
        assertTrue(dao.getOutbox().isEmpty())
        assertEquals(1, dao.rows.size)
    }

    @Test fun refreshKeepsPendingCreatesEditsAndDeletions() = runBlocking {
        val create = local(0).copy(rev = 0, baseRev = 0, dirty = true, pendingOp = "CREATE")
        val edit = local(1).copy(notes = "Unsynced edit", dirty = true, pendingOp = "UPDATE")
        val deletion = local(2).copy(deleted = true, dirty = true, pendingOp = "delete")
        val dao = SyncOutboxTest.MemoryDao(listOf(create, edit, deletion, local(3)))
        ServerSnapshot(listOf(local(1), local(2), local(4)), 500).replaceCache(dao, StateDao())
        assertEquals(create, dao.getByClientId(create.clientId))
        assertEquals(edit, dao.getByClientId(edit.clientId))
        assertEquals(deletion, dao.getByClientId(deletion.clientId))
        assertNull(dao.getByClientId(local(3).clientId))
        assertNotNull(dao.getByClientId(local(4).clientId))
        assertEquals(3, dao.getOutbox().size)
    }

    @Test fun failedSnapshotNeverRemovesCachedHistoryOrPendingWork() = runBlocking {
        val original = listOf(local(0), local(1).copy(dirty = true, pendingOp = "UPDATE"))
        val dao = SyncOutboxTest.MemoryDao(original)
        val state = StateDao()
        var calls = 0
        val api = api { _, _ ->
            calls++
            if (calls == 1) SyncPullResponse(listOf(row(0)), 10, true) else throw IOException("offline")
        }
        try { ServerSnapshot.fetch(api).replaceCache(dao, state); fail("Expected offline failure") } catch (_: IOException) { }
        assertEquals(original, dao.rows.values.toList())
        assertEquals(99999L, state.value.cursor)
    }

    @Test fun fullSnapshotFollowsAllPagesAndResetsTheDeltaCursor() = runBlocking {
        val requested = mutableListOf<Long>()
        val api = api { method, args ->
            assertEquals("syncPull", method)
            requested += args[0] as Long
            if (requested.size == 1) SyncPullResponse(listOf(row(0)), 10, true)
            else SyncPullResponse(listOf(row(1)), 20, false)
        }
        val dao = SyncOutboxTest.MemoryDao(emptyList())
        val state = StateDao()
        ServerSnapshot.fetch(api).replaceCache(dao, state)
        assertEquals(listOf(0L, 10L), requested)
        assertEquals(20L, state.value.cursor)
        assertEquals(2, dao.rows.size)
    }

    private fun row(index: Int): TxnRow {
        val local = local(index)
        return TxnRow(id = UUID.randomUUID().toString(), clientId = local.clientId, valueDate = local.valueDate,
            txnDate = local.txnDate, description = local.description, amount = local.amount, direction = local.direction,
            category = local.category, bank = local.bank, isTransfer = local.isTransfer, notes = local.notes,
            bankRef = local.bankRef, rev = 1, baseRev = 1)
    }
}
