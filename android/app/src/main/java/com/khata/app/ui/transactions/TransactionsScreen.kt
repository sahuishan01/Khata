package com.khata.app.ui.transactions

import androidx.compose.foundation.background
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
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
import com.khata.app.api.TxnListResponse
import com.khata.app.api.TxnRow
import java.time.LocalDate
import java.time.format.DateTimeFormatter

private fun fmt(n: Double) = "₹${String.format("%,.2f", n)}"

data class DatePreset(val label: String, val from: String?, val to: String?)

private val presets = listOf(
    DatePreset("All time", null, null),
    DatePreset("This week", getDateRange("week").first, getDateRange("week").second),
    DatePreset("This month", getDateRange("month").first, getDateRange("month").second),
    DatePreset("This quarter", getDateRange("quarter").first, getDateRange("quarter").second),
    DatePreset("This year", getDateRange("year").first, getDateRange("year").second),
    DatePreset("Custom", null, null),
)

private fun getDateRange(preset: String): Pair<String?, String?> {
    val today = LocalDate.now()
    val fmt = DateTimeFormatter.ISO_LOCAL_DATE
    return when (preset) {
        "week" -> {
            val start = today.with(java.time.DayOfWeek.MONDAY)
            Pair(start.format(fmt), today.format(fmt))
        }
        "month" -> Pair(today.withDayOfMonth(1).format(fmt), today.format(fmt))
        "quarter" -> {
            val qStart = today.withMonth(((today.monthValue - 1) / 3) * 3 + 1).withDayOfMonth(1)
            Pair(qStart.format(fmt), today.format(fmt))
        }
        "year" -> Pair(today.withDayOfYear(1).format(fmt), today.format(fmt))
        else -> Pair(null, null)
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun TransactionsScreen(
    txnState: TxnListResponse?,
    categories: List<String>,
    isLoading: Boolean,
    error: String?,
    initialCategory: String? = null,
    onLoad: (sortBy: String, sortDir: String, category: String?, from: String?, to: String?) -> Unit,
    onToggleTransfer: (String, Boolean) -> Unit = { _, _ -> },
    onUpdateNotes: (String, String) -> Unit = { _, _ -> },
    onUpdateCategory: ((String, String) -> Unit)? = null
) {
    var sortBy by remember { mutableStateOf("date") }
    var sortDir by remember { mutableStateOf("desc") }
    var selectedCategory by remember { mutableStateOf(initialCategory) }
    var selectedPreset by remember { mutableStateOf(0) }
    var customFrom by remember { mutableStateOf("") }
    var customTo by remember { mutableStateOf("") }
    var showSortMenu by remember { mutableStateOf(false) }
    var showCatFilter by remember { mutableStateOf(false) }
    var showCustomDatePicker by remember { mutableStateOf(false) }
    var showFromPicker by remember { mutableStateOf(false) }
    var showToPicker by remember { mutableStateOf(false) }

    val fmt = DateTimeFormatter.ISO_LOCAL_DATE

    fun getDateParams(): Pair<String?, String?> {
        val preset = presets[selectedPreset]
        return if (selectedPreset == presets.size - 1) { // Custom
            customFrom.ifBlank { null } to customTo.ifBlank { null }
        } else {
            preset.from to preset.to
        }
    }

    fun reload() {
        val (from, to) = getDateParams()
        onLoad(sortBy, sortDir, selectedCategory, from, to)
    }

    LaunchedEffect(Unit) { reload() }
    LaunchedEffect(sortBy, sortDir, selectedCategory, selectedPreset) { reload() }

    // Date picker dialogs
    if (showFromPicker) {
        val datePickerState = rememberDatePickerState(
            initialSelectedDateMillis = customFrom.ifBlank { null }
                ?.let { LocalDate.parse(it, fmt).toEpochDay() * 86400000 }
                ?: System.currentTimeMillis()
        )
        DatePickerDialog(
            onDismissRequest = { showFromPicker = false },
            confirmButton = {
                TextButton(onClick = {
                    datePickerState.selectedDateMillis?.let { millis ->
                        customFrom = LocalDate.ofEpochDay(millis / 86400000).format(fmt)
                    }
                    showFromPicker = false
                    reload()
                }) { Text("OK") }
            },
            dismissButton = {
                TextButton(onClick = { showFromPicker = false }) { Text("Cancel") }
            }
        ) {
            DatePicker(state = datePickerState)
        }
    }

    if (showToPicker) {
        val datePickerState = rememberDatePickerState(
            initialSelectedDateMillis = customTo.ifBlank { null }
                ?.let { LocalDate.parse(it, fmt).toEpochDay() * 86400000 }
                ?: System.currentTimeMillis()
        )
        DatePickerDialog(
            onDismissRequest = { showToPicker = false },
            confirmButton = {
                TextButton(onClick = {
                    datePickerState.selectedDateMillis?.let { millis ->
                        customTo = LocalDate.ofEpochDay(millis / 86400000).format(fmt)
                    }
                    showToPicker = false
                    reload()
                }) { Text("OK") }
            },
            dismissButton = {
                TextButton(onClick = { showToPicker = false }) { Text("Cancel") }
            }
        ) {
            DatePicker(state = datePickerState)
        }
    }

    Column(modifier = Modifier.fillMaxSize().padding(16.dp)) {
        Text("Transactions", fontSize = 22.sp, fontWeight = FontWeight.Bold)
        Spacer(Modifier.height(8.dp))

        // Date presets row (horizontally scrollable)
        Row(
            modifier = Modifier.fillMaxWidth().horizontalScroll(rememberScrollState()),
            horizontalArrangement = Arrangement.spacedBy(4.dp)
        ) {
            presets.forEachIndexed { i, preset ->
                FilterChip(
                    selected = selectedPreset == i,
                    onClick = {
                        selectedPreset = i
                        if (i == presets.size - 1) showCustomDatePicker = true
                    },
                    label = { Text(preset.label, fontSize = 10.sp) },
                    modifier = Modifier.height(30.dp)
                )
            }
        }

        // Custom date inputs
        if (selectedPreset == presets.size - 1) {
            Spacer(Modifier.height(6.dp))
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(8.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                OutlinedTextField(
                    value = customFrom,
                    onValueChange = { customFrom = it },
                    label = { Text("From", fontSize = 11.sp) },
                    placeholder = { Text("YYYY-MM-DD", fontSize = 10.sp) },
                    modifier = Modifier.weight(1f).height(52.dp),
                    singleLine = true,
                    textStyle = LocalTextStyle.current.copy(fontSize = 12.sp),
                    trailingIcon = {
                        IconButton(onClick = { showFromPicker = true }) {
                            Icon(Icons.Default.CalendarMonth, contentDescription = null, modifier = Modifier.size(18.dp))
                        }
                    }
                )
                Text("—", fontSize = 14.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                OutlinedTextField(
                    value = customTo,
                    onValueChange = { customTo = it },
                    label = { Text("To", fontSize = 11.sp) },
                    placeholder = { Text("YYYY-MM-DD", fontSize = 10.sp) },
                    modifier = Modifier.weight(1f).height(52.dp),
                    singleLine = true,
                    textStyle = LocalTextStyle.current.copy(fontSize = 12.sp),
                    trailingIcon = {
                        IconButton(onClick = { showToPicker = true }) {
                            Icon(Icons.Default.CalendarMonth, contentDescription = null, modifier = Modifier.size(18.dp))
                        }
                    }
                )
            }
        }

        Spacer(Modifier.height(8.dp))

        // Sort + Category filter row
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            Box {
                FilledTonalButton(onClick = { showSortMenu = true }, modifier = Modifier.height(36.dp)) {
                    Icon(Icons.Default.Sort, contentDescription = null, modifier = Modifier.size(16.dp))
                    Spacer(Modifier.width(4.dp))
                    Text(
                        when (sortBy) {
                            "amount" -> if (sortDir == "desc") "Amount ↓" else "Amount ↑"
                            else -> if (sortDir == "desc") "Date ↓" else "Date ↑"
                        }, fontSize = 12.sp
                    )
                }
                DropdownMenu(expanded = showSortMenu, onDismissRequest = { showSortMenu = false }) {
                    DropdownMenuItem(text = { Text("Date: newest first") }, onClick = { sortBy = "date"; sortDir = "desc"; showSortMenu = false })
                    DropdownMenuItem(text = { Text("Date: oldest first") }, onClick = { sortBy = "date"; sortDir = "asc"; showSortMenu = false })
                    DropdownMenuItem(text = { Text("Amount: highest first") }, onClick = { sortBy = "amount"; sortDir = "desc"; showSortMenu = false })
                    DropdownMenuItem(text = { Text("Amount: lowest first") }, onClick = { sortBy = "amount"; sortDir = "asc"; showSortMenu = false })
                }
            }

            if (categories.isNotEmpty()) {
                Box {
                    FilledTonalButton(onClick = { showCatFilter = true }, modifier = Modifier.height(36.dp)) {
                        Icon(Icons.Default.FilterList, contentDescription = null, modifier = Modifier.size(16.dp))
                        Spacer(Modifier.width(4.dp))
                        Text(selectedCategory ?: "All", fontSize = 12.sp, maxLines = 1, overflow = TextOverflow.Ellipsis)
                    }
                    DropdownMenu(expanded = showCatFilter, onDismissRequest = { showCatFilter = false }) {
                        DropdownMenuItem(text = { Text("All categories") }, onClick = { selectedCategory = null; showCatFilter = false })
                        categories.forEach { cat ->
                            DropdownMenuItem(text = { Text(cat) }, onClick = { selectedCategory = cat; showCatFilter = false })
                        }
                    }
                }
            }
        }

        Spacer(Modifier.height(8.dp))

        if (error != null) {
            Card(colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.errorContainer), modifier = Modifier.fillMaxWidth()) {
                Text(error, modifier = Modifier.padding(12.dp), fontSize = 13.sp)
            }
            Spacer(Modifier.height(8.dp))
        }

        Box(modifier = Modifier.fillMaxSize()) {
            if (isLoading && txnState == null) {
                Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                    CircularProgressIndicator()
                }
                return@Column
            }

            val txns = txnState?.data ?: emptyList()

            if (txns.isEmpty()) {
                Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                    Column(horizontalAlignment = Alignment.CenterHorizontally) {
                        Text("📄", fontSize = 36.sp)
                        Spacer(Modifier.height(8.dp))
                        Text("No transactions found", fontSize = 14.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                        Text("Try changing the date range", fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                }
                return@Column
            }

            // Content
            Column {
                Text("${txnState?.total ?: 0} transactions", fontSize = 13.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                Spacer(Modifier.height(8.dp))

                LazyColumn(verticalArrangement = Arrangement.spacedBy(6.dp)) {
                    items(txns, key = { it.id }) { txn -> TransactionCard(txn = txn, allCategories = categories, onToggleTransfer = onToggleTransfer, onUpdateNotes = onUpdateNotes, onUpdateCategory = onUpdateCategory) }
                }
            }

            // Loading overlay on top
            if (isLoading && txnState != null) {
                Box(
                    modifier = Modifier
                        .fillMaxSize()
                        .background(MaterialTheme.colorScheme.surface.copy(alpha = 0.7f)),
                    contentAlignment = Alignment.Center
                ) {
                    CircularProgressIndicator(
                        modifier = Modifier.size(28.dp),
                        strokeWidth = 3.dp,
                        color = MaterialTheme.colorScheme.primary
                    )
                }
            }
        }
    }
}

@Composable
private fun TransactionCard(
    txn: TxnRow,
    allCategories: List<String> = emptyList(),
    onToggleTransfer: (String, Boolean) -> Unit,
    onUpdateNotes: (String, String) -> Unit,
    onUpdateCategory: ((String, String) -> Unit)? = null
) {
    var showNotes by remember { mutableStateOf(false) }
    var notesText by remember { mutableStateOf(txn.notes) }
    var showCatMenu by remember { mutableStateOf(false) }
    var newCat by remember { mutableStateOf("") }

    Card(
        modifier = Modifier.fillMaxWidth(),
        shape = RoundedCornerShape(10.dp),
        colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surface),
        elevation = CardDefaults.cardElevation(defaultElevation = 1.dp)
    ) {
        Column(modifier = Modifier.padding(12.dp)) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Surface(
                    shape = RoundedCornerShape(8.dp),
                    color = if (txn.direction == "debit") MaterialTheme.colorScheme.errorContainer else MaterialTheme.colorScheme.secondaryContainer,
                    modifier = Modifier.size(36.dp)
                ) {
                    Box(contentAlignment = Alignment.Center) {
                        Icon(
                            if (txn.direction == "debit") Icons.Default.ArrowDownward else Icons.Default.ArrowUpward,
                            contentDescription = null, modifier = Modifier.size(18.dp),
                            tint = if (txn.direction == "debit") MaterialTheme.colorScheme.error else MaterialTheme.colorScheme.secondary
                        )
                    }
                }
                Spacer(Modifier.width(10.dp))
                Column(modifier = Modifier.weight(1f)) {
                    Text(txn.description, fontSize = 13.sp, fontWeight = FontWeight.Medium, maxLines = 1, overflow = TextOverflow.Ellipsis, color = MaterialTheme.colorScheme.onSurface)
                    if (txn.notes.isNotBlank()) {
                        Text(txn.notes, fontSize = 11.sp, color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.7f), maxLines = 2, overflow = TextOverflow.Ellipsis)
                    }
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Text(txn.valueDate, fontSize = 11.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                        Spacer(Modifier.width(8.dp))
                        if (txn.isTransfer) { Spacer(Modifier.width(4.dp)); Surface(shape = RoundedCornerShape(4.dp), color = MaterialTheme.colorScheme.tertiaryContainer) { Text("↔", fontSize = 10.sp, modifier = Modifier.padding(horizontal = 6.dp, vertical = 2.dp)) } }
                        Surface(shape = RoundedCornerShape(4.dp), color = MaterialTheme.colorScheme.surfaceVariant) {
                            Text(txn.category, fontSize = 10.sp, modifier = Modifier.padding(horizontal = 6.dp, vertical = 2.dp), color = MaterialTheme.colorScheme.onSurfaceVariant)
                        }
                    }
                }
                Text(fmt(txn.amount), fontSize = 15.sp, fontWeight = FontWeight.Bold,
                    color = if (txn.direction == "debit") MaterialTheme.colorScheme.error else MaterialTheme.colorScheme.secondary)
            }

            // Action buttons
            Spacer(Modifier.height(6.dp))
            Row(horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                FilterChip(selected = txn.isTransfer, onClick = { onToggleTransfer(txn.id, !txn.isTransfer) }, label = { Text("↔", fontSize = 10.sp) }, modifier = Modifier.height(28.dp))
                FilterChip(selected = showNotes, onClick = { showNotes = !showNotes; if (showNotes) notesText = txn.notes }, label = { Text("📝", fontSize = 10.sp) }, modifier = Modifier.height(28.dp))
                Box {
                    FilterChip(selected = false, onClick = { showCatMenu = true }, label = { Text(txn.category.take(8), fontSize = 9.sp, maxLines = 1) }, modifier = Modifier.height(28.dp))
                    DropdownMenu(expanded = showCatMenu, onDismissRequest = { showCatMenu = false }, modifier = Modifier.heightIn(max = 240.dp)) {
                        allCategories.forEach { cat ->
                            DropdownMenuItem(text = { Text(cat, fontSize = 11.sp) }, onClick = { onUpdateCategory?.invoke(txn.id, cat); showCatMenu = false }, modifier = Modifier.height(36.dp))
                        }
                        HorizontalDivider()
                        OutlinedTextField(value = newCat, onValueChange = { newCat = it }, placeholder = { Text("New…", fontSize = 12.sp) }, singleLine = true, modifier = Modifier.padding(horizontal = 8.dp).height(40.dp))
                        if (newCat.isNotBlank()) {
                            DropdownMenuItem(text = { Text("Create \"$newCat\"", fontSize = 12.sp) }, onClick = { onUpdateCategory?.invoke(txn.id, newCat); showCatMenu = false; newCat = "" })
                        }
                    }
                }
            }

            if (showNotes) {
                Spacer(Modifier.height(6.dp))
                Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                    OutlinedTextField(
                        value = notesText, onValueChange = { notesText = it },
                        modifier = Modifier.weight(1f),
                        placeholder = { Text("Add notes…", fontSize = 12.sp) },
                        singleLine = true,
                        textStyle = LocalTextStyle.current.copy(fontSize = 13.sp)
                    )
                    IconButton(onClick = { onUpdateNotes(txn.id, notesText); showNotes = false }) { Icon(Icons.Default.Check, contentDescription = "Save") }
                }
            }
        }
    }
}
