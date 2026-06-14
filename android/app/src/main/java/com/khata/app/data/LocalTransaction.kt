package com.khata.app.data

import androidx.room.ColumnInfo
import androidx.room.Entity
import androidx.room.Index
import androidx.room.PrimaryKey
import java.util.UUID

@Entity(
    tableName = "transactions",
    indices = [Index(value = ["clientId"]), Index(value = ["rev"])]
)
data class LocalTransaction(
    @PrimaryKey val id: String = UUID.randomUUID().toString(),
    val clientId: String = UUID.randomUUID().toString(),
    val description: String = "",
    val amount: Double = 0.0,
    val direction: String = "debit",
    val category: String = "Miscellaneous",
    val txnDate: String = "",
    val valueDate: String = "",
    val notes: String = "",
    val isTransfer: Boolean = false,
    val bank: String = "",
    val bankRef: String = "",
    val balance: Double? = null,
    val rev: Long = 0,
    val baseRev: Long = 0,
    val deleted: Boolean = false,
    val updatedAt: Long = System.currentTimeMillis(),
    val dirty: Boolean = false,
    val pendingOp: String? = null,
)

@Entity(tableName = "sync_state")
data class SyncState(
    @PrimaryKey val id: Int = 1,
    val cursor: Long = 0,
    val lastSyncAt: Long = 0,
)
