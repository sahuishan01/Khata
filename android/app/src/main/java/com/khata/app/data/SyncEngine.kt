package com.khata.app.data

import android.content.Context
import com.khata.app.api.KhataApi
import com.khata.app.api.SyncPushOp
import com.khata.app.api.toLocal
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.*
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import javax.inject.Inject
import javax.inject.Singleton

data class SyncConflict(
    val clientId: String,
    val serverRev: Long,
    val serverTxn: LocalTransaction,
    val localTxn: LocalTransaction,
)

sealed class SyncUiState {
    data object Idle : SyncUiState()
    data object Syncing : SyncUiState()
    data class Conflicts(val conflicts: List<SyncConflict>) : SyncUiState()
    data class Error(val message: String) : SyncUiState()
}

@Singleton
class SyncEngine @Inject constructor(
    @ApplicationContext private val context: Context,
    private val api: KhataApi,
    private val db: KhataDatabase,
) {
    private val _state = MutableStateFlow<SyncUiState>(SyncUiState.Idle)
    val state: StateFlow<SyncUiState> = _state

    private val scope = CoroutineScope(Dispatchers.IO + SupervisorJob())

    fun startPeriodicSync(intervalMs: Long = 30_000) {
        scope.launch {
            while (isActive) {
                sync()
                delay(intervalMs)
            }
        }
    }

    suspend fun sync() {
        _state.value = SyncUiState.Syncing
        try {
            pushOutbox()
            pullDeltas()
            _state.value = SyncUiState.Idle
        } catch (e: Exception) {
            _state.value = SyncUiState.Error(e.message ?: "Sync failed")
        }
    }

    private suspend fun pushOutbox() {
        val outbox = db.transactionDao().getOutbox()
        if (outbox.isEmpty()) return

        val ops = outbox.map { txn ->
            SyncPushOp(
                clientId = txn.clientId,
                baseRev = txn.baseRev,
                amount = txn.amount,
                direction = txn.direction,
                description = txn.description,
                category = txn.category,
                txnDate = txn.txnDate,
                valueDate = txn.valueDate,
                notes = txn.notes,
                isTransfer = txn.isTransfer,
                deleted = txn.deleted,
            )
        }

        val result = api.syncPush(ops)
        for (accepted in result.accepted) {
            val local = db.transactionDao().getByClientId(accepted.clientId)
            if (local != null) {
                db.transactionDao().upsert(
                    local.copy(
                        rev = accepted.serverRev,
                        baseRev = accepted.serverRev,
                        dirty = false,
                        pendingOp = null,
                    ),
                )
            }
        }

        if (result.conflicts.isNotEmpty()) {
            val conflicts = result.conflicts.map { c ->
                val local = db.transactionDao().getByClientId(c.clientId)
                SyncConflict(
                    clientId = c.clientId,
                    serverRev = c.serverRev,
                    serverTxn = LocalTransaction(
                        id = c.serverTxn.id,
                        clientId = c.serverTxn.clientId?.toString() ?: "",
                        description = c.serverTxn.description,
                        amount = c.serverTxn.amount,
                        direction = c.serverTxn.direction,
                        category = c.serverTxn.category,
                        valueDate = c.serverTxn.valueDate,
                        rev = c.serverRev,
                    ),
                    localTxn = local ?: LocalTransaction(),
                )
            }
            _state.value = SyncUiState.Conflicts(conflicts)
        }
    }

    private suspend fun pullDeltas() {
        var state = db.syncStateDao().get() ?: SyncState(cursor = 0)
        var hasMore = true

        while (hasMore) {
            val pull = api.syncPull(state.cursor)
            if (pull.txns.isNotEmpty()) {
                val local = pull.txns.map { it.toLocal() }
                db.transactionDao().upsertAll(local)
            }
            state = state.copy(cursor = pull.newRev)
            db.syncStateDao().set(state)
            hasMore = pull.hasMore
        }
    }

    fun resolveConflicts(resolutions: List<Pair<String, SyncPushOp>>) {
        scope.launch {
            try {
                val accepted = api.syncResolve(resolutions.map { it.second })
                for (a in accepted) {
                    val local = db.transactionDao().getByClientId(a.clientId)
                    if (local != null) {
                        db.transactionDao().upsert(
                            local.copy(
                                dirty = false,
                                pendingOp = null,
                                rev = a.serverRev,
                                baseRev = a.serverRev,
                            ),
                        )
                    }
                }
                _state.value = SyncUiState.Idle
                sync()
            } catch (e: Exception) {
                _state.value = SyncUiState.Error(e.message ?: "Resolution failed")
            }
        }
    }

    fun stop() {
        scope.cancel()
    }
}
