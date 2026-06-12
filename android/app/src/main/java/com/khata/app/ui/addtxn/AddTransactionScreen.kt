package com.khata.app.ui.addtxn

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Check
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.khata.app.api.CreateTxnReq
import java.time.LocalDate
import java.time.format.DateTimeFormatter

@Composable
fun AddTransactionScreen(
    isLoading: Boolean, error: String?, onAdd: (CreateTxnReq) -> Unit, onBack: () -> Unit
) {
    val today = LocalDate.now().format(DateTimeFormatter.ISO_LOCAL_DATE)
    var desc by remember { mutableStateOf("") }
    var amount by remember { mutableStateOf("") }
    var direction by remember { mutableStateOf("debit") }
    var txnDate by remember { mutableStateOf(today) }
    var valueDate by remember { mutableStateOf(today) }
    var category by remember { mutableStateOf("") }
    var bankRef by remember { mutableStateOf("") }
    var notes by remember { mutableStateOf("") }

    Column(Modifier.fillMaxSize().padding(16.dp).verticalScroll(rememberScrollState())) {
        Text("Add Transaction", fontSize = 20.sp, fontWeight = FontWeight.Bold)
        Spacer(Modifier.height(16.dp))

        Card(Modifier.fillMaxWidth(), shape = RoundedCornerShape(14.dp)) {
            Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
                OutlinedTextField(value = desc, onValueChange = { desc = it }, label = { Text("Description") }, singleLine = true, modifier = Modifier.fillMaxWidth())
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    OutlinedTextField(value = amount, onValueChange = { amount = it }, label = { Text("Amount") }, singleLine = true, modifier = Modifier.weight(1f))
                    Surface(shape = RoundedCornerShape(10.dp), color = MaterialTheme.colorScheme.surfaceVariant) {
                        Row(Modifier.padding(4.dp)) {
                            FilterChip(selected = direction == "debit", onClick = { direction = "debit" }, label = { Text("Expense", fontSize = 12.sp) })
                            Spacer(Modifier.width(4.dp))
                            FilterChip(selected = direction == "credit", onClick = { direction = "credit" }, label = { Text("Income", fontSize = 12.sp) })
                        }
                    }
                }
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    OutlinedTextField(value = txnDate, onValueChange = { txnDate = it }, label = { Text("Txn Date", fontSize = 12.sp) }, singleLine = true, modifier = Modifier.weight(1f))
                    OutlinedTextField(value = valueDate, onValueChange = { valueDate = it }, label = { Text("Value Date", fontSize = 12.sp) }, singleLine = true, modifier = Modifier.weight(1f))
                }
                OutlinedTextField(value = category, onValueChange = { category = it }, label = { Text("Category") }, singleLine = true, modifier = Modifier.fillMaxWidth())
                OutlinedTextField(value = bankRef, onValueChange = { bankRef = it }, label = { Text("Reference / UTR (optional)") }, singleLine = true, modifier = Modifier.fillMaxWidth())
                OutlinedTextField(value = notes, onValueChange = { notes = it }, label = { Text("Notes (optional)") }, singleLine = true, modifier = Modifier.fillMaxWidth(), maxLines = 3)

                if (error != null) Text(error, fontSize = 13.sp, color = MaterialTheme.colorScheme.error)

                Button(onClick = {
                    onAdd(CreateTxnReq(txnDate, valueDate, desc, amount.toDoubleOrNull() ?: 0.0, direction, category, bankRef.ifBlank { null }, notes.ifBlank { null }))
                }, enabled = !isLoading && desc.isNotBlank() && amount.isNotBlank(), modifier = Modifier.fillMaxWidth().height(48.dp), shape = RoundedCornerShape(10.dp)) {
                    if (isLoading) CircularProgressIndicator(Modifier.size(20.dp), strokeWidth = 2.dp) else { Icon(Icons.Default.Check, contentDescription = null); Spacer(Modifier.width(6.dp)); Text("Add Transaction") }
                }
            }
        }
    }
}
