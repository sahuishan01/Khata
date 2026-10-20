package com.khata.app.data

import com.khata.app.api.KhataApi
import com.khata.app.api.toLocal

/** A complete server history, fetched before replacing any reproducible device cache. */
internal data class ServerSnapshot(val rows: List<LocalTransaction>, val cursor: Long) {
    /** Caller runs this in a Room transaction. Pending edits and deletions always survive. */
    suspend fun replaceCache(dao: TransactionDao, syncState: SyncStateDao) {
        dao.clearSyncedCache()
        for (row in rows) {
            val pending = dao.getByClientId(row.clientId)
            if (pending?.dirty == true || pending?.pendingOp != null) continue
            dao.upsert(row)
        }
        syncState.set(SyncState(cursor = cursor))
    }

    companion object {
        suspend fun fetch(api: KhataApi): ServerSnapshot {
            val rows = mutableListOf<LocalTransaction>()
            var cursor = 0L
            while (true) {
                val page = api.syncPull(sinceRev = cursor, limit = 1000)
                check(!page.hasMore || (page.txns.isNotEmpty() && page.newRev > cursor)) {
                    "The server did not advance the history check. Existing device data was kept."
                }
                rows += page.txns.map { it.toLocal() }
                cursor = page.newRev
                if (!page.hasMore) return ServerSnapshot(rows, cursor)
            }
        }
    }
}
