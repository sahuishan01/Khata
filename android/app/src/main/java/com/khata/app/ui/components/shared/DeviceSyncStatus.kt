package com.khata.app.ui.components.shared

import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.khata.app.data.SyncUiState
import java.time.Instant
import java.time.ZoneId
import java.time.format.DateTimeFormatter

@Composable
fun DeviceSyncStatus(
    pending: Int,
    state: SyncUiState,
    serverUrl: String,
    lastSuccess: Long?,
    busy: Boolean,
    onRetry: () -> Unit,
) {
    val syncing = state is SyncUiState.Syncing
    val error = state is SyncUiState.Error || state is SyncUiState.Conflicts
    Column(Modifier.fillMaxWidth().padding(vertical = 8.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
        Text("$pending saved transactions awaiting upload", fontSize = 13.sp)
        Text("Server: $serverUrl", fontSize = 11.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
        if (syncing) LinearProgressIndicator(Modifier.fillMaxWidth())
        val message = when (state) {
            is SyncUiState.Syncing -> "Uploading and checking server changes…"
            is SyncUiState.Error -> state.message
            is SyncUiState.Conflicts -> "${state.conflicts.size} transactions need conflict resolution before sync can complete."
            else -> if (pending > 0) "Saved on this phone. Server transaction totals update after upload."
                else if (lastSuccess != null) "Saved transactions last checked " + Instant.ofEpochMilli(lastSuccess).atZone(ZoneId.systemDefault())
                    .format(DateTimeFormatter.ofPattern("dd MMM yyyy, HH:mm:ss")) else "Saved transactions have not been checked in this session."
        }
        Text(message, fontSize = 12.sp, color = if (error) MaterialTheme.colorScheme.error else MaterialTheme.colorScheme.onSurfaceVariant)
        OutlinedButton(onClick = onRetry, enabled = !syncing && !busy, modifier = Modifier.fillMaxWidth()) {
            Text(if (pending > 0 || error) "Retry saved transaction uploads" else "Sync saved transactions")
        }
    }
}
