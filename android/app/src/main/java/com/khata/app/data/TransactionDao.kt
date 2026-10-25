package com.khata.app.data

import androidx.room.*
import kotlinx.coroutines.flow.Flow

@Dao
interface TransactionDao {
    @Query("SELECT * FROM transactions WHERE deleted = 0 ORDER BY valueDate DESC")
    fun getAllFlow(): Flow<List<LocalTransaction>>

    @Query("SELECT * FROM transactions WHERE deleted = 0 ORDER BY valueDate DESC")
    suspend fun getAll(): List<LocalTransaction>

    @Query("SELECT * FROM transactions WHERE clientId = :clientId")
    suspend fun getByClientId(clientId: String): LocalTransaction?

    @Query("SELECT * FROM transactions WHERE deleted = 0 AND (:category IS NULL OR category = :category)")
    fun getFiltered(category: String?): Flow<List<LocalTransaction>>

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun upsert(txn: LocalTransaction)

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun upsertAll(txns: List<LocalTransaction>)

    @Query("UPDATE transactions SET dirty = 1, pendingOp = 'delete', deleted = 1 WHERE id = :id")
    suspend fun softDelete(id: String)

    /** A rescan must not overwrite an edit made after its provenance check. */
    @Transaction
    suspend fun excludeSmsIfUnchanged(expected: LocalTransaction): Boolean {
        if (getByClientId(expected.clientId) != expected) return false
        upsert(expected.copy(deleted = true, dirty = true, pendingOp = "DELETE"))
        return true
    }

    /** Do not replace a transaction edited while the inbox scan was parsing it. */
    @Transaction
    suspend fun saveSmsIfUnchanged(expected: LocalTransaction?, row: LocalTransaction): Boolean {
        if (getByClientId(row.clientId) != expected) return false
        upsert(row)
        return true
    }

    @Query("SELECT * FROM transactions WHERE dirty = 1 AND pendingOp IS NOT NULL")
    suspend fun getOutbox(): List<LocalTransaction>

    @Query("SELECT COUNT(*) FROM transactions WHERE dirty = 1 AND pendingOp IS NOT NULL")
    fun pendingCountFlow(): Flow<Int>

    @Query("SELECT MAX(rev) FROM transactions")
    suspend fun maxRev(): Long?

    @Query("DELETE FROM transactions")
    suspend fun clearAll()

    @Query("DELETE FROM transactions WHERE dirty = 0 AND pendingOp IS NULL")
    suspend fun clearSyncedCache()

    @Query("DELETE FROM transactions WHERE dirty = 0 AND pendingOp IS NULL AND clientId = ''")
    suspend fun clearLegacyServerCache()
}

@Dao
interface SyncStateDao {
    @Query("SELECT * FROM sync_state WHERE id = 1")
    suspend fun get(): SyncState?

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun set(state: SyncState)
}
