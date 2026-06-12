package com.khata.app.ui.upload

import android.net.Uri
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.khata.app.api.CreateTxnReq
import java.time.LocalDate
import java.time.format.DateTimeFormatter

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun CombinedUploadScreen(
    isDark: Boolean,
    onToggleDark: () -> Unit,
    resultMessage: String?,
    onPickFile: () -> Unit,
    onClearResult: () -> Unit,
    onClearAllData: () -> Unit,
    onAddTxn: (CreateTxnReq) -> Unit
) {
    var tab by remember { mutableStateOf(0) }
    var showClearDialog by remember { mutableStateOf(false) }
    val today = LocalDate.now().format(DateTimeFormatter.ISO_LOCAL_DATE)
    var desc by remember { mutableStateOf("") }; var amount by remember { mutableStateOf("") }
    var direction by remember { mutableStateOf("debit") }; var txnDate by remember { mutableStateOf(today) }
    var valueDate by remember { mutableStateOf(today) }; var category by remember { mutableStateOf("") }
    var notes by remember { mutableStateOf("") }; var loading by remember { mutableStateOf(false) }
    var error by remember { mutableStateOf("") }; var success by remember { mutableStateOf("") }

    if (showClearDialog) {
        AlertDialog(onDismissRequest = { showClearDialog = false }, title = { Text("Clear All Data?") }, text = { Text("This action cannot be undone.") },
            confirmButton = { TextButton(onClick = { showClearDialog = false; onClearAllData() }) { Text("Clear", color = MaterialTheme.colorScheme.error) } },
            dismissButton = { TextButton(onClick = { showClearDialog = false }) { Text("Cancel") } })
    }

    Column(Modifier.fillMaxSize().padding(16.dp).verticalScroll(rememberScrollState())) {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
            Column { Text("Add Data", fontSize = 20.sp, fontWeight = FontWeight.Bold); Text("Upload or enter manually", fontSize = 13.sp, color = MaterialTheme.colorScheme.onSurfaceVariant) }
            IconButton(onClick = onToggleDark) { Icon(if (isDark) Icons.Default.LightMode else Icons.Default.DarkMode, contentDescription = "Theme") }
        }
        Spacer(Modifier.height(12.dp))

        // Tab selector - centered
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.Center) {
            Surface(
                onClick = { tab = 0 },
                shape = RoundedCornerShape(topStart = 10.dp, bottomStart = 10.dp),
                color = if (tab == 0) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.surfaceVariant
            ) { Text("Upload", modifier = Modifier.padding(horizontal = 24.dp, vertical = 10.dp), color = if (tab == 0) MaterialTheme.colorScheme.onPrimary else MaterialTheme.colorScheme.onSurface, fontWeight = FontWeight.SemiBold) }
            Surface(
                onClick = { tab = 1 },
                shape = RoundedCornerShape(topEnd = 10.dp, bottomEnd = 10.dp),
                color = if (tab == 1) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.surfaceVariant
            ) { Text("Manual", modifier = Modifier.padding(horizontal = 24.dp, vertical = 10.dp), color = if (tab == 1) MaterialTheme.colorScheme.onPrimary else MaterialTheme.colorScheme.onSurface, fontWeight = FontWeight.SemiBold) }
        }
        Spacer(Modifier.height(16.dp))

        if (tab == 0) {
            // Upload section
            Surface(onClick = onPickFile, modifier = Modifier.fillMaxWidth().height(160.dp), shape = MaterialTheme.shapes.medium, color = MaterialTheme.colorScheme.surface, tonalElevation = 2.dp) {
                Column(Modifier.fillMaxSize().padding(24.dp), horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.Center) {
                    Icon(Icons.Default.UploadFile, contentDescription = null, modifier = Modifier.size(48.dp), tint = MaterialTheme.colorScheme.primary)
                    Spacer(Modifier.height(8.dp)); Text("Tap to select a file"); Text("CSV, XLS, XLSX", color = MaterialTheme.colorScheme.onSurfaceVariant)
                }
            }
            resultMessage?.let { msg ->
                Spacer(Modifier.height(12.dp))
                Surface(shape = MaterialTheme.shapes.medium, color = if (msg.startsWith("Error")) MaterialTheme.colorScheme.errorContainer else MaterialTheme.colorScheme.secondaryContainer) {
                    Text(msg, modifier = Modifier.padding(12.dp), fontSize = 13.sp)
                }
                if (!msg.startsWith("Error")) { Spacer(Modifier.height(4.dp)); TextButton(onClick = onClearResult) { Text("Dismiss") } }
            }
            Spacer(Modifier.height(16.dp))
            OutlinedButton(onClick = { showClearDialog = true }, modifier = Modifier.fillMaxWidth(), colors = ButtonDefaults.outlinedButtonColors(contentColor = MaterialTheme.colorScheme.error)) {
                Icon(Icons.Default.DeleteForever, contentDescription = null, modifier = Modifier.size(18.dp)); Spacer(Modifier.width(6.dp)); Text("Clear All Data")
            }
        } else {
            // Manual entry
            Card(Modifier.fillMaxWidth(), shape = RoundedCornerShape(14.dp)) {
                Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
                    OutlinedTextField(value = desc, onValueChange = { desc = it }, label = { Text("Description") }, singleLine = true, modifier = Modifier.fillMaxWidth())
                    Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        OutlinedTextField(value = amount, onValueChange = { amount = it }, label = { Text("Amount") }, singleLine = true, modifier = Modifier.weight(1f))
                        Surface(shape = RoundedCornerShape(10.dp), color = MaterialTheme.colorScheme.surfaceVariant) {
                            Row(Modifier.padding(4.dp)) {
                                FilterChip(selected = direction == "debit", onClick = { direction = "debit" }, label = { Text("Expense", fontSize = 12.sp) })
                                FilterChip(selected = direction == "credit", onClick = { direction = "credit" }, label = { Text("Income", fontSize = 12.sp) })
                            }
                        }
                    }
                    Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        OutlinedTextField(value = txnDate, onValueChange = { txnDate = it }, label = { Text("Date", fontSize = 12.sp) }, singleLine = true, modifier = Modifier.weight(1f))
                        OutlinedTextField(value = valueDate, onValueChange = { valueDate = it }, label = { Text("Value Date", fontSize = 12.sp) }, singleLine = true, modifier = Modifier.weight(1f))
                    }
                    OutlinedTextField(value = category, onValueChange = { category = it }, label = { Text("Category") }, singleLine = true, modifier = Modifier.fillMaxWidth())
                    OutlinedTextField(value = notes, onValueChange = { notes = it }, label = { Text("Notes") }, singleLine = true, modifier = Modifier.fillMaxWidth(), maxLines = 3)
                    if (error.isNotBlank()) Text(error, fontSize = 13.sp, color = MaterialTheme.colorScheme.error)
                    if (success.isNotBlank()) Text(success, fontSize = 13.sp, color = MaterialTheme.colorScheme.secondary)
                    Button(onClick = {
                        if (desc.isBlank() || amount.isBlank()) { error = "Fill required fields"; return@Button }
                        onAddTxn(CreateTxnReq(txnDate, valueDate, desc, amount.toDoubleOrNull() ?: 0.0, direction, category, null, notes.ifBlank { null }))
                        desc = ""; amount = ""; category = ""; notes = ""; success = "Transaction added!"
                    }, modifier = Modifier.fillMaxWidth().height(48.dp), shape = RoundedCornerShape(10.dp)) {
                        Icon(Icons.Default.Check, contentDescription = null); Spacer(Modifier.width(6.dp)); Text("Add Transaction")
                    }
                }
            }
        }
    }
}
