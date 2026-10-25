package com.khata.app.data

import android.content.Context
import androidx.room.withTransaction
import com.khata.app.api.KhataApi
import com.khata.app.api.SyncPushOp
import com.khata.app.api.toLocal
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.*
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
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
    private val syncMutex = Mutex()
    private val _lastSuccess = MutableStateFlow<Long?>(null)
    val lastSuccess: StateFlow<Long?> = _lastSuccess

    private val scope = CoroutineScope(Dispatchers.IO + SupervisorJob())

    /** A manual inbox scan must see server deletions, including Clear All Data from the web. */
    suspend fun refreshCacheForSmsScan() = syncMutex.withLock {
        // A failed or incomplete fetch leaves both the cache and outbox untouched.
        val snapshot = ServerSnapshot.fetch(api)
        db.withTransaction { snapshot.replaceCache(db.transactionDao(), db.syncStateDao()) }
        context.getSharedPreferences("khata_sync", Context.MODE_PRIVATE)
            .edit().putInt("cursor_format", 2).apply()
    }

    /** Called only after the user's existing DELETE confirmation. */
    suspend fun clearAllData() = syncMutex.withLock {
        api.clearAllData()
        db.withTransaction {
            db.transactionDao().clearAll()
            db.smsDecisionDao().clearAll()
            db.syncStateDao().set(SyncState())
        }
        _state.value = SyncUiState.Idle
        _lastSuccess.value = null
    }

    suspend fun sync(): Boolean = syncMutex.withLock {
        _state.value = SyncUiState.Syncing
        try {
            val conflicts = SyncOutbox(db.transactionDao(), api).push()
            if (conflicts.isNotEmpty()) _state.value = SyncUiState.Conflicts(conflicts)
            pullDeltas()
            val complete = _state.value !is SyncUiState.Conflicts
            if (complete) {
                check(db.transactionDao().getOutbox().isEmpty()) { "New changes remain queued. Sync again to upload them." }
                _state.value = SyncUiState.Idle
                _lastSuccess.value = System.currentTimeMillis()
            }
            complete
        } catch (e: CancellationException) {
            _state.value = SyncUiState.Idle
            throw e
        } catch (e: Exception) {
            _state.value = SyncUiState.Error(e.syncErrorMessage())
            false
        }
    }

    private suspend fun pullDeltas() {
        var state = db.syncStateDao().get() ?: SyncState(cursor = 0)
        val prefs = context.getSharedPreferences("khata_sync", Context.MODE_PRIVATE)
        if (prefs.getInt("cursor_format", 1) < 2) {
            // Replace only reproducible server cache, keeping unsent local work.
            db.transactionDao().clearLegacyServerCache()
            state = state.copy(cursor = 0)
        }
        var hasMore = true

        while (hasMore) {
            val pull = api.syncPull(state.cursor)
            if (pull.txns.isNotEmpty()) {
                val local = pull.txns.mapNotNull { row ->
                    val incoming = row.toLocal()
                    val existing = db.transactionDao().getByClientId(incoming.clientId)
                    if (existing?.dirty == true) null
                    else incoming.copy(id = existing?.id ?: incoming.id)
                }
                db.transactionDao().upsertAll(local)
            }
            state = state.copy(cursor = pull.newRev)
            db.syncStateDao().set(state)
            hasMore = pull.hasMore
        }
        prefs.edit().putInt("cursor_format", 2).apply()
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
