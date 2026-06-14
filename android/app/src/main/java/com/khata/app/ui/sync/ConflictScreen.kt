package com.khata.app.ui.sync

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.khata.app.data.SyncConflict
import com.khata.app.ui.theme.KhataColors

enum class ConflictStrategy { Override, KeepExisting, Merge }

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ConflictScreen(
    conflicts: List<SyncConflict>,
    onResolve: (List<Pair<String, ConflictStrategy>>) -> Unit,
    onDismiss: () -> Unit,
) {
    var globalStrategy by remember { mutableStateOf<ConflictStrategy?>(null) }
    var perItemStrategies by remember { mutableStateOf<Map<String, ConflictStrategy>>(emptyMap()) }

    fun getStrategy(clientId: String): ConflictStrategy =
        perItemStrategies[clientId] ?: globalStrategy ?: ConflictStrategy.KeepExisting

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Sync Conflicts", fontSize = 16.sp) },
                navigationIcon = { IconButton(onClick = onDismiss) { Icon(Icons.Default.Close, contentDescription = null) } }
            )
        },
        bottomBar = {
            Surface(tonalElevation = 3.dp) {
                Button(
                    onClick = {
                        val resolved = conflicts.map { it.clientId to getStrategy(it.clientId) }
                        onResolve(resolved)
                    },
                    modifier = Modifier.fillMaxWidth().padding(16.dp).height(48.dp),
                    shape = RoundedCornerShape(8.dp),
                    colors = ButtonDefaults.buttonColors(containerColor = KhataColors.brand)
                ) { Text("Apply Resolutions (${conflicts.size})") }
            }
        }
    ) { innerPadding ->
        LazyColumn(Modifier.fillMaxSize().padding(innerPadding).padding(16.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
            item {
                Text("${conflicts.size} transaction(s) were edited on another device while you were offline.", fontSize = 14.sp, color = KhataColors.text2)
                Spacer(Modifier.height(12.dp))

                // Global strategy dropdown
                Text("Apply to all:", fontSize = 12.sp, fontWeight = FontWeight.SemiBold, color = KhataColors.textMuted)
                Spacer(Modifier.height(4.dp))
                StrategyDropdown(
                    label = globalStrategy?.name ?: "Choose strategy…",
                    selected = globalStrategy,
                    onSelect = { globalStrategy = it }
                )
            }

            items(conflicts, key = { it.clientId }) { conflict ->
                Card(
                    modifier = Modifier.fillMaxWidth(),
                    shape = RoundedCornerShape(12.dp),
                    colors = CardDefaults.cardColors(containerColor = KhataColors.surface)
                ) {
                    Column(Modifier.padding(14.dp)) {
                        Row(verticalAlignment = Alignment.CenterVertically) {
                            Icon(Icons.Default.Warning, contentDescription = null, tint = KhataColors.warn, modifier = Modifier.size(20.dp))
                            Spacer(Modifier.width(8.dp))
                            Text("Conflict", fontSize = 14.sp, fontWeight = FontWeight.SemiBold, color = KhataColors.text)
                        }
                        Spacer(Modifier.height(8.dp))

                        // Local vs Server diff
                        DiffRow("Description", conflict.localTxn.description, conflict.serverTxn.description)
                        DiffRow("Amount", "₹${conflict.localTxn.amount}", "₹${conflict.serverTxn.amount}")
                        DiffRow("Category", conflict.localTxn.category, conflict.serverTxn.category)
                        DiffRow("Direction", conflict.localTxn.direction, conflict.serverTxn.direction)

                        Spacer(Modifier.height(8.dp))
                        val cur = getStrategy(conflict.clientId)
                        StrategyDropdown(
                            label = when (cur) { ConflictStrategy.Override -> "Override (local wins)"; ConflictStrategy.KeepExisting -> "Keep existing (server wins)"; ConflictStrategy.Merge -> "Merge (field-level)" },
                            selected = cur,
                            onSelect = { perItemStrategies = perItemStrategies + (conflict.clientId to it) }
                        )
                    }
                }
            }

            item { Spacer(Modifier.height(80.dp)) }
        }
    }
}

@Composable
private fun DiffRow(label: String, local: String, server: String) {
    if (local == server) return
    Column(Modifier.padding(vertical = 2.dp)) {
        Text(label, fontSize = 10.sp, color = KhataColors.textMuted)
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            Surface(shape = RoundedCornerShape(4.dp), color = KhataColors.brandSoft) {
                Text("Local: $local", fontSize = 12.sp, modifier = Modifier.padding(horizontal = 6.dp, vertical = 2.dp), color = KhataColors.brand)
            }
            Surface(shape = RoundedCornerShape(4.dp), color = KhataColors.expenseSoft) {
                Text("Server: $server", fontSize = 12.sp, modifier = Modifier.padding(horizontal = 6.dp, vertical = 2.dp), color = KhataColors.expense)
            }
        }
    }
}

@Composable
private fun StrategyDropdown(label: String, selected: ConflictStrategy?, onSelect: (ConflictStrategy) -> Unit) {
    var expanded by remember { mutableStateOf(false) }
    Box {
        OutlinedButton(onClick = { expanded = true }, modifier = Modifier.fillMaxWidth().height(40.dp)) {
            Text(label, fontSize = 12.sp)
            Spacer(Modifier.weight(1f))
            Icon(Icons.Default.ArrowDropDown, contentDescription = null, modifier = Modifier.size(16.dp))
        }
        DropdownMenu(expanded = expanded, onDismissRequest = { expanded = false }) {
            DropdownMenuItem(text = { Text("Override (local wins)") }, onClick = { onSelect(ConflictStrategy.Override); expanded = false })
            DropdownMenuItem(text = { Text("Keep existing (server wins)") }, onClick = { onSelect(ConflictStrategy.KeepExisting); expanded = false })
            DropdownMenuItem(text = { Text("Merge (field-level)") }, onClick = { onSelect(ConflictStrategy.Merge); expanded = false })
        }
    }
}
