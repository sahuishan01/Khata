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

    @Query("SELECT * FROM transactions WHERE dirty = 1 AND pendingOp IS NOT NULL")
    suspend fun getOutbox(): List<LocalTransaction>

    @Query("SELECT MAX(rev) FROM transactions")
    suspend fun maxRev(): Long?

    @Query("DELETE FROM transactions")
    suspend fun clearAll()
}

@Dao
interface SyncStateDao {
    @Query("SELECT * FROM sync_state WHERE id = 1")
    suspend fun get(): SyncState?

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun set(state: SyncState)
}
