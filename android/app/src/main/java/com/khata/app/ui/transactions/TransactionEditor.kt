package com.khata.app.ui.transactions

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.dp
import com.khata.app.api.TxnRow
import com.khata.app.api.UpdateTxnReq
import kotlinx.coroutines.launch
import java.time.LocalDate

internal fun validEditMoney(text: String, positive: Boolean): Double? {
    if (!Regex("-?\\d+(?:\\.\\d{1,2})?").matches(text)) return null
    return text.toDoubleOrNull()?.takeIf { it.isFinite() && kotlin.math.abs(it) <= 999_999_999_999.99 && (!positive || it > 0) }
}

@Composable
fun TransactionEditor(txn: TxnRow, onSave: suspend (String, UpdateTxnReq) -> String?, onClose: () -> Unit) {
    // The revision belongs to this draft, even if the background list refreshes.
    val baseRev = rememberSaveable { txn.rev }
    var description by rememberSaveable { mutableStateOf(txn.description) }
    var amount by rememberSaveable { mutableStateOf(java.math.BigDecimal.valueOf(txn.amount).toPlainString()) }
    var direction by rememberSaveable { mutableStateOf(txn.direction) }
    var txnDate by rememberSaveable { mutableStateOf(txn.txnDate) }
    var valueDate by rememberSaveable { mutableStateOf(txn.valueDate) }
    var category by rememberSaveable { mutableStateOf(txn.category) }
    var bank by rememberSaveable { mutableStateOf(txn.bank) }
    var bankRef by rememberSaveable { mutableStateOf(txn.bankRef.orEmpty()) }
    var balance by rememberSaveable { mutableStateOf(txn.balance?.let { java.math.BigDecimal.valueOf(it).toPlainString() }.orEmpty()) }
    var transfer by rememberSaveable { mutableStateOf(txn.isTransfer) }
    var notes by rememberSaveable { mutableStateOf(txn.notes) }
    var error by remember { mutableStateOf<String?>(null) }
    var saving by remember { mutableStateOf(false) }
    val scope = rememberCoroutineScope()
    val valid = description.isNotBlank() && description.length <= 16_000 &&
        validEditMoney(amount, true) != null && (balance.isBlank() || validEditMoney(balance, false) != null) &&
        runCatching { LocalDate.parse(txnDate); LocalDate.parse(valueDate) }.isSuccess &&
        category.isNotBlank() && category.length <= 100 && bank.isNotBlank() && bank.length <= 100 &&
        bankRef.length <= 500 && notes.length <= 10_000
    AlertDialog(
        onDismissRequest = { if (!saving) onClose() },
        title = { Text("Edit transaction") },
        text = {
            Column(Modifier.fillMaxWidth().verticalScroll(rememberScrollState()), verticalArrangement = Arrangement.spacedBy(12.dp)) {
                OutlinedTextField(description, { description = it }, label = { Text("Description") },
                    minLines = 3, maxLines = 6, modifier = Modifier.fillMaxWidth(), enabled = !saving)
                OutlinedTextField(amount, { amount = it }, label = { Text("Amount (INR)") },
                    keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Decimal), singleLine = true,
                    isError = validEditMoney(amount, true) == null, modifier = Modifier.fillMaxWidth(), enabled = !saving)
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    FilterChip(direction == "debit", { direction = "debit" }, label = { Text("Expense") }, enabled = !saving)
                    FilterChip(direction == "credit", { direction = "credit" }, label = { Text("Income") }, enabled = !saving)
                }
                OutlinedTextField(txnDate, { txnDate = it }, label = { Text("Transaction date (YYYY-MM-DD)") },
                    singleLine = true, modifier = Modifier.fillMaxWidth(), enabled = !saving)
                OutlinedTextField(valueDate, { valueDate = it }, label = { Text("Value date (YYYY-MM-DD)") },
                    singleLine = true, modifier = Modifier.fillMaxWidth(), enabled = !saving)
                OutlinedTextField(category, { category = it }, label = { Text("Category") },
                    singleLine = true, modifier = Modifier.fillMaxWidth(), enabled = !saving)
                OutlinedTextField(bank, { bank = it }, label = { Text("Bank") },
                    singleLine = true, modifier = Modifier.fillMaxWidth(), enabled = !saving)
                OutlinedTextField(bankRef, { bankRef = it }, label = { Text("Bank reference") },
                    singleLine = true, modifier = Modifier.fillMaxWidth(), enabled = !saving)
                OutlinedTextField(balance, { balance = it }, label = { Text("Balance (optional)") },
                    keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Decimal), singleLine = true,
                    isError = balance.isNotBlank() && validEditMoney(balance, false) == null,
                    modifier = Modifier.fillMaxWidth(), enabled = !saving)
                Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
                    Text("Transfer", Modifier.weight(1f))
                    Switch(transfer, { transfer = it }, enabled = !saving)
                }
                OutlinedTextField(notes, { notes = it }, label = { Text("Notes") },
                    minLines = 2, maxLines = 4, modifier = Modifier.fillMaxWidth(), enabled = !saving)
                Text("Use a positive amount with up to two decimal places and dates in YYYY-MM-DD format.", style = MaterialTheme.typography.bodySmall)
                error?.let { Text(it, color = MaterialTheme.colorScheme.error) }
            }
        },
        confirmButton = {
            TextButton(enabled = valid && !saving, onClick = {
                saving = true; error = null
                scope.launch {
                    try {
                        error = onSave(txn.id, UpdateTxnReq(baseRev, txnDate, valueDate, description,
                            validEditMoney(amount, true)!!, direction, category.trim(), bank.trim(), bankRef,
                            balance.takeIf { it.isNotBlank() }?.let { validEditMoney(it, false) }, transfer, notes))
                        if (error == null) onClose()
                    } finally { saving = false }
                }
            }) { Text(if (saving) "Saving…" else "Save") }
        },
        dismissButton = { TextButton(enabled = !saving, onClick = onClose) { Text("Cancel") } },
    )
}
