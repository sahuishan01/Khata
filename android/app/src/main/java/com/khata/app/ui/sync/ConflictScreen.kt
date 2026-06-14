package com.khata.app.ui.sync

import androidx.compose.foundation.BorderStroke
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
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.khata.app.data.SyncConflict
import com.khata.app.ui.components.shared.KhataCard
import com.khata.app.ui.components.shared.KhataCardBody
import com.khata.app.ui.theme.KhataColors

enum class ConflictStrategy { Override, KeepExisting, Merge }

private data class ConflictMeta(
    val isDeleteConflict: Boolean = false,
    val isAmountConflict: Boolean = false,
    val diffCount: Int = 0,
)

private fun analyzeConflict(c: SyncConflict): ConflictMeta {
    val local = c.localTxn; val remote = c.serverTxn
    var diffCount = 0
    var amtConflict = false
    if (local.amount != remote.amount) { diffCount++; amtConflict = true }
    if (local.description != remote.description) diffCount++
    if (local.category != remote.category) diffCount++
    if (local.direction != remote.direction) diffCount++
    val isDel = local.deleted || remote.deleted
    return ConflictMeta(isDel, amtConflict, diffCount)
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ConflictScreen(
    conflicts: List<SyncConflict>,
    onResolve: (List<Pair<String, ConflictStrategy>>) -> Unit,
    onDismiss: () -> Unit,
) {
    var globalStrategy by remember { mutableStateOf<ConflictStrategy?>(null) }
    var perItemStrategies by remember { mutableStateOf<Map<String, ConflictStrategy>>(emptyMap()) }
    var manualAmounts by remember { mutableStateOf<Map<String, String>>(emptyMap()) }

    val allSame = conflicts.all { perItemStrategies[it.clientId] ?: globalStrategy == (perItemStrategies[conflicts.first().clientId] ?: globalStrategy) }

    fun getStrategy(clientId: String): ConflictStrategy {
        val meta = analyzeConflict(conflicts.first { it.clientId == clientId })
        val base = perItemStrategies[clientId] ?: globalStrategy ?: ConflictStrategy.KeepExisting
        if (meta.isDeleteConflict && base == ConflictStrategy.Merge) return ConflictStrategy.KeepExisting
        return base
    }

    val isMixed = conflicts.any { (perItemStrategies[it.clientId] ?: globalStrategy) != (perItemStrategies[conflicts.first().clientId] ?: globalStrategy) }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Column { Text("Resolve conflicts", fontSize = 16.sp); Text("${conflicts.size} edits clash with server changes", fontSize = 11.sp, color = KhataColors.text2) } },
                navigationIcon = { IconButton(onClick = onDismiss) { Icon(Icons.Default.Close, contentDescription = null) } }
            )
        },
        bottomBar = {
            Surface(color = KhataColors.surface) {
                Row(Modifier.fillMaxWidth().padding(16.dp), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                    OutlinedButton(onClick = onDismiss, modifier = Modifier.weight(1f).height(48.dp), shape = RoundedCornerShape(8.dp)) { Text("Cancel") }
                    Button(
                        onClick = { val r = conflicts.map { it.clientId to getStrategy(it.clientId) }; onResolve(r) },
                        modifier = Modifier.weight(1f).height(48.dp), shape = RoundedCornerShape(8.dp),
                        colors = ButtonDefaults.buttonColors(containerColor = KhataColors.brand)
                    ) { Text("Apply ${conflicts.size} & sync") }
                }
            }
        }
    ) { innerPadding ->
        LazyColumn(Modifier.fillMaxSize().padding(innerPadding).padding(16.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
            // Global strategy bar
            item {
                KhataCard {
                    KhataCardBody {
                        Text("Apply to all", fontSize = 12.sp, fontWeight = FontWeight.SemiBold, color = KhataColors.textMuted)
                        Spacer(Modifier.height(6.dp))
                        val allStrategy = if (!isMixed) globalStrategy else null
                        StrategyDropdown(
                            label = when { isMixed -> "Mixed"; allStrategy != null -> when (allStrategy) { ConflictStrategy.Override -> "Override (keep mine)"; ConflictStrategy.KeepExisting -> "Keep existing"; ConflictStrategy.Merge -> "Merge" }; else -> "Choose…" },
                            selected = allStrategy,
                            onSelect = { globalStrategy = it; perItemStrategies = emptyMap() },
                            showMerge = true,
                        )
                    }
                }
            }

            // Conflict cards
            items(conflicts, key = { it.clientId }) { conflict ->
                val meta = analyzeConflict(conflict)
                val cur = getStrategy(conflict.clientId)
                val local = conflict.localTxn; val remote = conflict.serverTxn
                val isMerge = cur == ConflictStrategy.Merge

                KhataCard {
                    KhataCardBody {
                        // Title row
                        Row(verticalAlignment = Alignment.CenterVertically) {
                            Icon(Icons.Default.Warning, contentDescription = null, tint = KhataColors.warn, modifier = Modifier.size(18.dp))
                            Spacer(Modifier.width(8.dp))
                            Column(Modifier.weight(1f)) {
                                Text(if (local.description.isNotBlank()) local.description else remote.description, fontSize = 14.sp, fontWeight = FontWeight.SemiBold, color = KhataColors.text, maxLines = 1, overflow = TextOverflow.Ellipsis)
                                Text(local.valueDate.ifBlank { remote.valueDate }, fontSize = 11.sp, color = KhataColors.text2)
                            }
                            if (meta.isDeleteConflict) {
                                Surface(shape = RoundedCornerShape(4.dp), color = KhataColors.expenseSoft) { Text("Delete vs edit", fontSize = 9.sp, modifier = Modifier.padding(horizontal = 6.dp, vertical = 2.dp), color = KhataColors.expense) }
                            } else if (meta.isAmountConflict) {
                                Surface(shape = RoundedCornerShape(4.dp), color = KhataColors.warn.copy(alpha = 0.2f)) { Text("Amount conflict", fontSize = 9.sp, modifier = Modifier.padding(horizontal = 6.dp, vertical = 2.dp), color = KhataColors.warn) }
                            } else {
                                Surface(shape = RoundedCornerShape(4.dp), color = KhataColors.brandSoft) { Text("${meta.diffCount} field${if (meta.diffCount != 1) "s" else ""} differ", fontSize = 9.sp, modifier = Modifier.padding(horizontal = 6.dp, vertical = 2.dp), color = KhataColors.brand) }
                            }
                        }
                        Spacer(Modifier.height(8.dp))

                        // Diff rows
                        diffField("Description", local.description, remote.description, isMerge, cur)
                        diffField("Category", local.category, remote.category, isMerge, cur)
                        diffField("Direction", local.direction, remote.direction, isMerge, cur)
                        if (local.amount != remote.amount) {
                            val isBothChanged = local.amount != remote.amount && local.amount != conflict.serverTxn.amount
                            diffAmount(local.amount, remote.amount, isMerge, cur, isBothChanged && isMerge,
                                manualAmount = manualAmounts[conflict.clientId],
                                onPick = { amt -> manualAmounts = manualAmounts + (conflict.clientId to amt) }
                            )
                        }

                        // Delete-vs-edit note
                        if (meta.isDeleteConflict) {
                            Surface(shape = RoundedCornerShape(4.dp), color = KhataColors.expenseSoft, modifier = Modifier.fillMaxWidth()) {
                                Text("This transaction was deleted on one device and edited on the other. Merge is not available.", fontSize = 11.sp, modifier = Modifier.padding(8.dp), color = KhataColors.expense)
                            }
                            Spacer(Modifier.height(4.dp))
                        }

                        // Merge preview for non-delete conflicts
                        if (isMerge && !meta.isDeleteConflict) {
                            Spacer(Modifier.height(4.dp))
                            val mergedDesc = if (local.description != remote.description && local.description.isNotBlank() && remote.description.isNotBlank()) "${local.description} / ${remote.description}" else if (local.description.isNotBlank()) local.description else remote.description
                            val mergedCat = if (local.category != remote.category && local.category.isNotBlank() && remote.category.isNotBlank()) "${local.category} / ${remote.category}" else if (local.category.isNotBlank()) local.category else remote.category
                            Surface(shape = RoundedCornerShape(4.dp), color = KhataColors.incomeSoft, modifier = Modifier.fillMaxWidth()) {
                                Text("Merged: $mergedDesc · $mergedCat", fontSize = 11.sp, modifier = Modifier.padding(8.dp), color = KhataColors.income, maxLines = 2, overflow = TextOverflow.Ellipsis)
                            }
                        }

                        Spacer(Modifier.height(8.dp))
                        Row(verticalAlignment = Alignment.CenterVertically) {
                            Text("Resolution:", fontSize = 11.sp, color = KhataColors.textMuted)
                            Spacer(Modifier.width(8.dp))
                            StrategyDropdown(
                                label = when (cur) { ConflictStrategy.Override -> "Override (keep mine)"; ConflictStrategy.KeepExisting -> "Keep existing"; ConflictStrategy.Merge -> "Merge" },
                                selected = cur,
                                onSelect = { perItemStrategies = perItemStrategies + (conflict.clientId to it) },
                                showMerge = !meta.isDeleteConflict,
                            )
                        }
                    }
                }
            }

            item { Spacer(Modifier.height(80.dp)) }
        }
    }
}

@Composable
private fun diffField(label: String, local: String, remote: String, isMerge: Boolean, strategy: ConflictStrategy) {
    if (local == remote || (local.isBlank() && remote.isBlank())) return
    val localWins = strategy == ConflictStrategy.Override || (isMerge && local != remote && remote.isBlank())
    val remoteWins = strategy == ConflictStrategy.KeepExisting || (isMerge && local != remote && local.isBlank())
    val borderColor = if (localWins) KhataColors.brand else if (remoteWins) KhataColors.expense else KhataColors.hairline

    Column(Modifier.padding(vertical = 2.dp)) {
        Text(label, fontSize = 9.sp, color = KhataColors.textMuted)
        Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
            Surface(shape = RoundedCornerShape(4.dp), border = BorderStroke(1.dp, if (localWins) KhataColors.brand else KhataColors.hairline), color = if (localWins) KhataColors.brandSoft else KhataColors.surface2) {
                Text(local.ifBlank { "—" }, fontSize = 11.sp, modifier = Modifier.padding(horizontal = 6.dp, vertical = 3.dp), color = KhataColors.text, maxLines = 1, overflow = TextOverflow.Ellipsis)
            }
            Text("vs", fontSize = 9.sp, color = KhataColors.textMuted, modifier = Modifier.align(Alignment.CenterVertically))
            Surface(shape = RoundedCornerShape(4.dp), border = BorderStroke(1.dp, if (remoteWins) KhataColors.expense else KhataColors.hairline), color = if (remoteWins) KhataColors.expenseSoft else KhataColors.surface2) {
                Text(remote.ifBlank { "—" }, fontSize = 11.sp, modifier = Modifier.padding(horizontal = 6.dp, vertical = 3.dp), color = KhataColors.text, maxLines = 1, overflow = TextOverflow.Ellipsis)
            }
        }
    }
}

@Composable
private fun diffAmount(local: Double, remote: Double, isMerge: Boolean, strategy: ConflictStrategy, bothChanged: Boolean, manualAmount: String?, onPick: (String) -> Unit) {
    val localWins = strategy == ConflictStrategy.Override || (isMerge && !bothChanged)
    val remoteWins = strategy == ConflictStrategy.KeepExisting || (isMerge && !bothChanged)
    Column(Modifier.padding(vertical = 2.dp)) {
        Text("Amount", fontSize = 9.sp, color = KhataColors.textMuted)
        Row(horizontalArrangement = Arrangement.spacedBy(6.dp), verticalAlignment = Alignment.CenterVertically) {
            Surface(shape = RoundedCornerShape(4.dp), border = BorderStroke(1.dp, if (localWins) KhataColors.brand else KhataColors.hairline), color = if (localWins) KhataColors.brandSoft else KhataColors.surface2) {
                Text("₹${local}", fontSize = 11.sp, modifier = Modifier.padding(horizontal = 6.dp, vertical = 3.dp), color = KhataColors.text)
            }
            Text("vs", fontSize = 9.sp, color = KhataColors.textMuted)
            Surface(shape = RoundedCornerShape(4.dp), border = BorderStroke(1.dp, if (remoteWins) KhataColors.expense else KhataColors.hairline), color = if (remoteWins) KhataColors.expenseSoft else KhataColors.surface2) {
                Text("₹${remote}", fontSize = 11.sp, modifier = Modifier.padding(horizontal = 6.dp, vertical = 3.dp), color = KhataColors.text)
            }
        }
        if (bothChanged && isMerge) {
            Spacer(Modifier.height(4.dp))
            Surface(shape = RoundedCornerShape(4.dp), color = KhataColors.warn.copy(alpha = 0.15f), modifier = Modifier.fillMaxWidth()) {
                Text("Amount changed on both devices — pick manually:", fontSize = 10.sp, modifier = Modifier.padding(6.dp), color = KhataColors.warn)
            }
            Spacer(Modifier.height(4.dp))
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                FilterChip(selected = manualAmount == "local", onClick = { onPick("local") }, label = { Text("₹${local}", fontSize = 11.sp) })
                FilterChip(selected = manualAmount == "remote", onClick = { onPick("remote") }, label = { Text("₹${remote}", fontSize = 11.sp) })
            }
        }
    }
}

@Composable
private fun StrategyDropdown(label: String, selected: ConflictStrategy?, onSelect: (ConflictStrategy) -> Unit, showMerge: Boolean = true) {
    var expanded by remember { mutableStateOf(false) }
    Box {
        OutlinedButton(onClick = { expanded = true }, modifier = Modifier.height(36.dp), shape = RoundedCornerShape(6.dp), border = BorderStroke(1.dp, KhataColors.hairline)) {
            Text(label, fontSize = 11.sp, color = KhataColors.text, maxLines = 1)
            Spacer(Modifier.width(4.dp))
            Icon(Icons.Default.ArrowDropDown, contentDescription = null, modifier = Modifier.size(16.dp), tint = KhataColors.text2)
        }
        DropdownMenu(expanded = expanded, onDismissRequest = { expanded = false }) {
            DropdownMenuItem(text = { Text("Override (keep mine)", fontSize = 12.sp) }, onClick = { onSelect(ConflictStrategy.Override); expanded = false })
            DropdownMenuItem(text = { Text("Keep existing", fontSize = 12.sp) }, onClick = { onSelect(ConflictStrategy.KeepExisting); expanded = false })
            if (showMerge) {
                DropdownMenuItem(text = { Text("Merge", fontSize = 12.sp) }, onClick = { onSelect(ConflictStrategy.Merge); expanded = false })
            }
        }
    }
}
