package com.khata.app.ui.transactions

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowDownward
import androidx.compose.material.icons.filled.ArrowUpward
import androidx.compose.material.icons.filled.FilterList
import androidx.compose.material.icons.filled.Sort
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.khata.app.api.TxnListResponse
import com.khata.app.api.TxnRow

private fun fmt(n: Double) = "₹${String.format("%,.2f", n)}"

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun TransactionsScreen(
    txnState: TxnListResponse?,
    categories: List<String>,
    isLoading: Boolean,
    error: String?,
    onLoad: (sortBy: String, sortDir: String, category: String?) -> Unit
) {
    var sortBy by remember { mutableStateOf("date") }
    var sortDir by remember { mutableStateOf("desc") }
    var selectedCategory by remember { mutableStateOf<String?>(null) }
    var showSortMenu by remember { mutableStateOf(false) }
    var showCatFilter by remember { mutableStateOf(false) }

    LaunchedEffect(Unit) { onLoad(sortBy, sortDir, selectedCategory) }
    LaunchedEffect(sortBy, sortDir, selectedCategory) { onLoad(sortBy, sortDir, selectedCategory) }

    Column(modifier = Modifier.fillMaxSize().padding(16.dp)) {
        Text("Transactions", fontSize = 22.sp, fontWeight = FontWeight.Bold)
        Spacer(Modifier.height(4.dp))

        // Filters row
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            // Sort button
            Box {
                FilledTonalButton(
                    onClick = { showSortMenu = true },
                    modifier = Modifier.height(36.dp)
                ) {
                    Icon(Icons.Default.Sort, contentDescription = null, modifier = Modifier.size(16.dp))
                    Spacer(Modifier.width(4.dp))
                    Text(
                        when (sortBy) {
                            "amount" -> if (sortDir == "desc") "Amount ↓" else "Amount ↑"
                            else -> if (sortDir == "desc") "Date ↓" else "Date ↑"
                        },
                        fontSize = 12.sp
                    )
                }
                DropdownMenu(expanded = showSortMenu, onDismissRequest = { showSortMenu = false }) {
                    DropdownMenuItem(
                        text = { Text("Date: newest first") },
                        onClick = { sortBy = "date"; sortDir = "desc"; showSortMenu = false }
                    )
                    DropdownMenuItem(
                        text = { Text("Date: oldest first") },
                        onClick = { sortBy = "date"; sortDir = "asc"; showSortMenu = false }
                    )
                    DropdownMenuItem(
                        text = { Text("Amount: highest first") },
                        onClick = { sortBy = "amount"; sortDir = "desc"; showSortMenu = false }
                    )
                    DropdownMenuItem(
                        text = { Text("Amount: lowest first") },
                        onClick = { sortBy = "amount"; sortDir = "asc"; showSortMenu = false }
                    )
                }
            }

            // Category filter button
            if (categories.isNotEmpty()) {
                Box {
                    FilledTonalButton(
                        onClick = { showCatFilter = true },
                        modifier = Modifier.height(36.dp)
                    ) {
                        Icon(Icons.Default.FilterList, contentDescription = null, modifier = Modifier.size(16.dp))
                        Spacer(Modifier.width(4.dp))
                        Text(
                            selectedCategory ?: "All categories",
                            fontSize = 12.sp,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis
                        )
                    }
                    DropdownMenu(expanded = showCatFilter, onDismissRequest = { showCatFilter = false }) {
                        DropdownMenuItem(
                            text = { Text("All categories") },
                            onClick = { selectedCategory = null; showCatFilter = false }
                        )
                        categories.forEach { cat ->
                            DropdownMenuItem(
                                text = { Text(cat) },
                                onClick = { selectedCategory = cat; showCatFilter = false }
                            )
                        }
                    }
                }
            }
        }

        Spacer(Modifier.height(8.dp))

        if (error != null) {
            Card(
                colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.errorContainer),
                modifier = Modifier.fillMaxWidth()
            ) {
                Text(error, modifier = Modifier.padding(12.dp), fontSize = 13.sp)
            }
            Spacer(Modifier.height(8.dp))
        }

        if (isLoading && txnState == null) {
            Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                CircularProgressIndicator()
            }
            return
        }

        val txns = txnState?.data ?: emptyList()

        if (txns.isEmpty()) {
            Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                Column(horizontalAlignment = Alignment.CenterHorizontally) {
                    Text("📄", fontSize = 36.sp)
                    Spacer(Modifier.height(8.dp))
                    Text("No transactions found", fontSize = 14.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    Text("Upload a statement to get started", fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                }
            }
            return
        }

        Text("${txnState?.total ?: 0} transactions", fontSize = 13.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
        Spacer(Modifier.height(8.dp))

        LazyColumn(verticalArrangement = Arrangement.spacedBy(6.dp)) {
            items(txns) { txn ->
                TransactionCard(txn)
            }
        }
    }
}

@Composable
private fun TransactionCard(txn: TxnRow) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        shape = RoundedCornerShape(10.dp),
        colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surface),
        elevation = CardDefaults.cardElevation(defaultElevation = 1.dp)
    ) {
        Row(
            modifier = Modifier.padding(12.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Surface(
                shape = RoundedCornerShape(8.dp),
                color = if (txn.direction == "debit")
                    MaterialTheme.colorScheme.errorContainer
                else
                    MaterialTheme.colorScheme.secondaryContainer,
                modifier = Modifier.size(36.dp)
            ) {
                Box(contentAlignment = Alignment.Center) {
                    Icon(
                        if (txn.direction == "debit") Icons.Default.ArrowDownward else Icons.Default.ArrowUpward,
                        contentDescription = null,
                        modifier = Modifier.size(18.dp),
                        tint = if (txn.direction == "debit")
                            MaterialTheme.colorScheme.error
                        else
                            MaterialTheme.colorScheme.secondary
                    )
                }
            }
            Spacer(Modifier.width(10.dp))
            Column(modifier = Modifier.weight(1f)) {
                Text(
                    txn.description, fontSize = 13.sp, fontWeight = FontWeight.Medium,
                    maxLines = 1, overflow = TextOverflow.Ellipsis,
                    color = MaterialTheme.colorScheme.onSurface
                )
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Text(txn.valueDate, fontSize = 11.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    Spacer(Modifier.width(8.dp))
                    Surface(
                        shape = RoundedCornerShape(4.dp),
                        color = MaterialTheme.colorScheme.surfaceVariant
                    ) {
                        Text(
                            txn.category, fontSize = 10.sp,
                            modifier = Modifier.padding(horizontal = 6.dp, vertical = 2.dp),
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }
                }
            }
            Text(
                fmt(txn.amount),
                fontSize = 15.sp,
                fontWeight = FontWeight.Bold,
                color = if (txn.direction == "debit")
                    MaterialTheme.colorScheme.error
                else
                    MaterialTheme.colorScheme.secondary
            )
        }
    }
}
