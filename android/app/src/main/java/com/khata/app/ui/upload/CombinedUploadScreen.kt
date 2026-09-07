package com.khata.app.ui.upload

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
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.khata.app.api.CreateTxnReq
import com.khata.app.api.EmailSyncRun
import com.khata.app.api.SaveEmailConfigReq
import com.khata.app.api.UserEmailConfigResponse
import java.time.LocalDate
import java.time.format.DateTimeFormatter

@Composable
fun CombinedUploadScreen(
    resultMessage: String?,
    onPickFile: () -> Unit,
    onClearResult: () -> Unit,
    onClearAllData: () -> Unit,
    onAddTxn: (CreateTxnReq) -> Unit,
    onSaveGmail: (SaveEmailConfigReq, (String) -> Unit) -> Unit = { _, cb -> cb("Error: not wired") },
    onLoadGmailConfig: ((UserEmailConfigResponse?) -> Unit) -> Unit = { it(null) },
    onLoadLatestRun: ((EmailSyncRun?) -> Unit) -> Unit = { it(null) },
    onSyncNow: ((String) -> Unit) -> Unit = { it("Error: not wired") }
) {
    var tab by remember { mutableStateOf(1) }
    var showClearDialog by remember { mutableStateOf(false) }
    var clearConfirmText by remember { mutableStateOf("") }
    val today = LocalDate.now().format(DateTimeFormatter.ISO_LOCAL_DATE)
    var desc by remember { mutableStateOf("") }; var amount by remember { mutableStateOf("") }
    var direction by remember { mutableStateOf("debit") }; var txnDate by remember { mutableStateOf(today) }
    var valueDate by remember { mutableStateOf(today) }; var category by remember { mutableStateOf("") }
    var notes by remember { mutableStateOf("") }; var error by remember { mutableStateOf("") }; var success by remember { mutableStateOf("") }

    if (showClearDialog) {
        AlertDialog(onDismissRequest = { showClearDialog = false; clearConfirmText = "" },
            title = { Text("Clear All Data", color = MaterialTheme.colorScheme.error) },
            text = {
                Column {
                    Text("This will permanently delete all transactions, statements, and chat history. This action cannot be undone.", fontSize = 13.sp)
                    Spacer(Modifier.height(12.dp))
                    Text("Type DELETE to confirm:", fontSize = 13.sp)
                    Spacer(Modifier.height(6.dp))
                    OutlinedTextField(value = clearConfirmText, onValueChange = { clearConfirmText = it }, placeholder = { Text("DELETE") }, singleLine = true, modifier = Modifier.fillMaxWidth())
                }
            },
            confirmButton = {
                TextButton(onClick = { showClearDialog = false; clearConfirmText = ""; onClearAllData() }, enabled = clearConfirmText == "DELETE") { Text("Clear Everything", color = MaterialTheme.colorScheme.error) }
            },
            dismissButton = { TextButton(onClick = { showClearDialog = false; clearConfirmText = "" }) { Text("Cancel") } })
    }

    Column(Modifier.fillMaxSize().padding(16.dp)) {
        // Title at top
        Text("Add Data", fontSize = 20.sp, fontWeight = FontWeight.Bold)
        Text("Connect Gmail, upload a statement, or enter manually", fontSize = 13.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
        Spacer(Modifier.height(16.dp))

        // Content takes remaining space
        Box(Modifier.weight(1f)) {
            if (tab == 0) {
                Column(Modifier.fillMaxSize().verticalScroll(rememberScrollState())) {
                    Surface(onClick = onPickFile, modifier = Modifier.fillMaxWidth().height(200.dp), shape = MaterialTheme.shapes.medium, color = MaterialTheme.colorScheme.surface, tonalElevation = 2.dp) {
                        Column(Modifier.fillMaxSize().padding(24.dp), horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.Center) {
                            Icon(Icons.Default.UploadFile, contentDescription = null, modifier = Modifier.size(48.dp), tint = MaterialTheme.colorScheme.primary)
                            Spacer(Modifier.height(8.dp)); Text("Tap to select a file"); Text("CSV, XLS, XLSX", color = MaterialTheme.colorScheme.onSurfaceVariant)
                        }
                    }
                    resultMessage?.let { msg ->
                        Spacer(Modifier.height(12.dp))
                        Surface(shape = MaterialTheme.shapes.medium, color = if (msg.startsWith("Error")) MaterialTheme.colorScheme.errorContainer else MaterialTheme.colorScheme.secondaryContainer) { Text(msg, modifier = Modifier.padding(12.dp), fontSize = 13.sp) }
                        if (!msg.startsWith("Error")) { Spacer(Modifier.height(4.dp)); TextButton(onClick = onClearResult) { Text("Dismiss") } }
                    }
                }
            } else if (tab == 2) {
                var emailInput by remember { mutableStateOf("") }
                var appPassInput by remember { mutableStateOf("") }
                var pdfPassInput by remember { mutableStateOf("") }
                var statusMsg by remember { mutableStateOf("") }
                var saving by remember { mutableStateOf(false) }
                var hasStoredKey by remember { mutableStateOf(false) }
                var latestRun by remember { mutableStateOf<EmailSyncRun?>(null) }
                var showErrors by remember { mutableStateOf(false) }

                LaunchedEffect(Unit) {
                    onLoadGmailConfig { cfg ->
                        if (cfg != null) {
                            hasStoredKey = true
                            if (emailInput.isBlank()) emailInput = cfg.emailAddress
                        }
                    }
                }

                // Poll the latest run: fast while one is running, slow otherwise.
                LaunchedEffect(hasStoredKey) {
                    while (hasStoredKey) {
                        onLoadLatestRun { latestRun = it }
                        kotlinx.coroutines.delay(if (latestRun?.isRunning == true) 3000L else 15000L)
                    }
                }

                Column(Modifier.fillMaxSize().verticalScroll(rememberScrollState())) {
                    Text("Automated Gmail Statement Sync", fontWeight = FontWeight.Bold, fontSize = 16.sp)
                    Spacer(Modifier.height(4.dp))
                    Text("Connect your Gmail using an App Password. Credentials are encrypted at rest with AES-256-GCM and protected by Row-Level Security.", fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    Spacer(Modifier.height(16.dp))

                    OutlinedTextField(value = emailInput, onValueChange = { emailInput = it }, placeholder = { Text("Gmail Address (yourname@gmail.com)") }, singleLine = true, modifier = Modifier.fillMaxWidth())
                    Spacer(Modifier.height(8.dp))
                    OutlinedTextField(value = appPassInput, onValueChange = { appPassInput = it }, placeholder = { Text("Google App Password (16 chars)") }, singleLine = true, modifier = Modifier.fillMaxWidth())
                    Spacer(Modifier.height(4.dp))
                    Text("Generate at myaccount.google.com/apppasswords", fontSize = 11.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    Spacer(Modifier.height(8.dp))
                    OutlinedTextField(value = pdfPassInput, onValueChange = { pdfPassInput = it }, placeholder = { Text("Statement Password (optional)") }, singleLine = true, modifier = Modifier.fillMaxWidth())
                    Spacer(Modifier.height(16.dp))

                    if (statusMsg.isNotBlank()) {
                        Text(statusMsg, fontSize = 12.sp, color = MaterialTheme.colorScheme.primary)
                        Spacer(Modifier.height(8.dp))
                    }

                    Button(
                        onClick = {
                            val email = emailInput.trim()
                            val appPass = appPassInput.trim()
                            if (email.isBlank() || (appPass.isBlank() && !hasStoredKey)) {
                                statusMsg = "Enter your Gmail address and App Password."
                                return@Button
                            }
                            saving = true
                            statusMsg = "Sending to server…"
                            onSaveGmail(
                                SaveEmailConfigReq(
                                    emailAddress = email,
                                    appPassword = appPass,
                                    pdfPassword = pdfPassInput.trim().ifBlank { null },
                                )
                            ) { msg ->
                                saving = false
                                statusMsg = msg
                                if (!msg.startsWith("Error")) {
                                    appPassInput = ""; pdfPassInput = ""; hasStoredKey = true
                                }
                            }
                        },
                        enabled = !saving,
                        modifier = Modifier.fillMaxWidth().height(48.dp),
                        shape = RoundedCornerShape(10.dp)
                    ) {
                        Icon(Icons.Default.Lock, contentDescription = null)
                        Spacer(Modifier.width(6.dp))
                        Text(if (saving) "Saving…" else "Save Encrypted Config")
                    }

                    if (hasStoredKey) {
                        Spacer(Modifier.height(16.dp))
                        OutlinedButton(
                            onClick = { statusMsg = "Starting sync…"; onSyncNow { statusMsg = it } },
                            modifier = Modifier.fillMaxWidth()
                        ) { Text("Sync now") }

                        latestRun?.let { run ->
                            Spacer(Modifier.height(12.dp))
                            val bg = when (run.status) {
                                "error" -> MaterialTheme.colorScheme.errorContainer
                                "running" -> MaterialTheme.colorScheme.secondaryContainer
                                else -> MaterialTheme.colorScheme.surfaceVariant
                            }
                            Surface(shape = RoundedCornerShape(10.dp), color = bg, modifier = Modifier.fillMaxWidth()) {
                                Column(Modifier.padding(12.dp)) {
                                    val label = when (run.status) {
                                        "running" -> if (run.fullScan) "Full mailbox scan running…" else "Syncing…"
                                        "ok" -> "Last sync complete"
                                        "error" -> "Last sync failed"
                                        else -> "Last sync: ${run.status}"
                                    }
                                    Text(label, fontWeight = FontWeight.SemiBold, fontSize = 13.sp)
                                    Spacer(Modifier.height(4.dp))
                                    Text(
                                        "${run.messagesScanned} messages · ${run.attachmentsParsed}/${run.attachmentsSeen} statements · " +
                                            "${run.txnsImported} imported · ${run.txnsSkipped} skipped",
                                        fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant
                                    )
                                    run.error?.let {
                                        Spacer(Modifier.height(4.dp))
                                        Text(it, fontSize = 12.sp, color = MaterialTheme.colorScheme.error)
                                    }
                                    if (run.errors.isNotEmpty()) {
                                        Spacer(Modifier.height(4.dp))
                                        TextButton(onClick = { showErrors = !showErrors }, contentPadding = PaddingValues(0.dp)) {
                                            Text(if (showErrors) "Hide ${run.errors.size} issues" else "Show ${run.errors.size} issues", fontSize = 12.sp)
                                        }
                                        if (showErrors) run.errors.forEach { e ->
                                            Text("• ${e.item ?: e.stage ?: ""}: ${e.detail ?: ""}", fontSize = 11.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                Column(Modifier.fillMaxSize().verticalScroll(rememberScrollState())) {
                    // Description
                    OutlinedTextField(value = desc, onValueChange = { desc = it }, placeholder = { Text("Description") }, singleLine = true, modifier = Modifier.fillMaxWidth())
                    Spacer(Modifier.height(8.dp))
                    // Amount + Type row
                    Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        OutlinedTextField(value = amount, onValueChange = { amount = it }, placeholder = { Text("Amount") }, singleLine = true, modifier = Modifier.weight(1f))
                        Surface(shape = RoundedCornerShape(10.dp), color = MaterialTheme.colorScheme.surfaceVariant) {
                            Row(Modifier.padding(4.dp)) {
                                FilterChip(selected = direction == "debit", onClick = { direction = "debit" }, label = { Text("Expense", fontSize = 12.sp) }, colors = FilterChipDefaults.filterChipColors(selectedContainerColor = MaterialTheme.colorScheme.secondaryContainer, selectedLabelColor = MaterialTheme.colorScheme.onSecondaryContainer))
                                FilterChip(selected = direction == "credit", onClick = { direction = "credit" }, label = { Text("Income", fontSize = 12.sp) }, colors = FilterChipDefaults.filterChipColors(selectedContainerColor = MaterialTheme.colorScheme.secondaryContainer, selectedLabelColor = MaterialTheme.colorScheme.onSecondaryContainer))
                            }
                        }
                    }
                    Spacer(Modifier.height(8.dp))
                    // Date + Value Date row
                    Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        OutlinedTextField(value = txnDate, onValueChange = { txnDate = it }, label = { Text("Txn Date", fontSize = 11.sp) }, singleLine = true, modifier = Modifier.weight(1f))
                        OutlinedTextField(value = valueDate, onValueChange = { valueDate = it }, label = { Text("Value Date", fontSize = 11.sp) }, singleLine = true, modifier = Modifier.weight(1f))
                    }
                    Spacer(Modifier.height(8.dp))
                    // Category
                    OutlinedTextField(value = category, onValueChange = { category = it }, placeholder = { Text("Category") }, singleLine = true, modifier = Modifier.fillMaxWidth())
                    Spacer(Modifier.height(8.dp))
                    // Notes
                    OutlinedTextField(value = notes, onValueChange = { notes = it }, placeholder = { Text("Notes (optional)") }, modifier = Modifier.fillMaxWidth(), maxLines = 2)
                    Spacer(Modifier.height(12.dp))
                    if (error.isNotBlank()) { Text(error, fontSize = 13.sp, color = MaterialTheme.colorScheme.error); Spacer(Modifier.height(8.dp)) }
                    if (success.isNotBlank()) { Text(success, fontSize = 13.sp, color = MaterialTheme.colorScheme.secondary); Spacer(Modifier.height(8.dp)) }
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

        Spacer(Modifier.height(12.dp))

        // Tab selector at bottom
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.Center) {
            Surface(
                onClick = { tab = 2 },
                shape = RoundedCornerShape(topStart = 10.dp, bottomStart = 10.dp),
                color = if (tab == 2) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.surfaceVariant
            ) { Text("Gmail Sync", modifier = Modifier.padding(horizontal = 16.dp, vertical = 10.dp), color = if (tab == 2) MaterialTheme.colorScheme.onPrimary else MaterialTheme.colorScheme.onSurface, fontWeight = FontWeight.SemiBold) }
            Surface(
                onClick = { tab = 0 },
                color = if (tab == 0) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.surfaceVariant
            ) { Text("Upload", modifier = Modifier.padding(horizontal = 16.dp, vertical = 10.dp), color = if (tab == 0) MaterialTheme.colorScheme.onPrimary else MaterialTheme.colorScheme.onSurface, fontWeight = FontWeight.SemiBold) }
            Surface(
                onClick = { tab = 1 },
                shape = RoundedCornerShape(topEnd = 10.dp, bottomEnd = 10.dp),
                color = if (tab == 1) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.surfaceVariant
            ) { Text("Manual", modifier = Modifier.padding(horizontal = 16.dp, vertical = 10.dp), color = if (tab == 1) MaterialTheme.colorScheme.onPrimary else MaterialTheme.colorScheme.onSurface, fontWeight = FontWeight.SemiBold) }
        }
    }
}
