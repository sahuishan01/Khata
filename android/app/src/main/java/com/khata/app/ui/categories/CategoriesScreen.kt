package com.khata.app.ui.categories

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.khata.app.api.Category
import com.khata.app.ui.components.shared.ButtonVariant
import com.khata.app.ui.components.shared.ChipColor
import com.khata.app.ui.components.shared.KhataButton
import com.khata.app.ui.components.shared.KhataCard
import com.khata.app.ui.components.shared.KhataChip
import com.khata.app.ui.components.shared.KhataEmptyState
import com.khata.app.ui.components.shared.KhataField

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun CategoriesScreen(
    categories: List<Category>, isLoading: Boolean, error: String?,
    onLoad: () -> Unit, onCreate: (String, String, String?, String?) -> Unit, onDelete: (String) -> Unit
) {
    LaunchedEffect(Unit) { onLoad() }
    var showDialog by remember { mutableStateOf(false) }

    LazyColumn(Modifier.fillMaxSize().padding(16.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
        item {
            Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
                Column { Text("Categories", fontSize = 20.sp, fontWeight = FontWeight.Bold); Text("Manage transaction types", fontSize = 13.sp, color = MaterialTheme.colorScheme.onSurfaceVariant) }
                KhataButton(onClick = { showDialog = true }) { Icon(Icons.Default.Add, contentDescription = null); Spacer(Modifier.width(4.dp)); Text("New") }
            }
        }

        items(categories, key = { it.id }) { cat ->
            val bgColor = try { Color(android.graphics.Color.parseColor(cat.color)) } catch (_: Exception) { MaterialTheme.colorScheme.primary }
            KhataCard(Modifier.fillMaxWidth()) {
                Row(Modifier.padding(14.dp), verticalAlignment = Alignment.CenterVertically) {
                    Box(Modifier.size(14.dp).clip(CircleShape).background(bgColor))
                    Spacer(Modifier.width(10.dp))
                    Column(Modifier.weight(1f)) {
                        Text(cat.name, fontSize = 14.sp, fontWeight = FontWeight.Medium)
                        if (cat.description.isNotBlank()) Text(cat.description, fontSize = 11.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                    KhataChip(
                        text = if (cat.txnType == "income") "Income" else "Expense",
                        color = if (cat.txnType == "income") ChipColor.Green else ChipColor.Red
                    )
                    Spacer(Modifier.width(8.dp))
                    KhataButton(onClick = { onDelete(cat.id) }, variant = ButtonVariant.Ghost) { Icon(Icons.Default.Delete, contentDescription = null, modifier = Modifier.size(16.dp), tint = MaterialTheme.colorScheme.error) }
                }
            }
        }
        if (categories.isEmpty()) item {
            KhataEmptyState(
                emoji = "\uD83C\uDFF7\uFE0F",
                title = "No categories yet",
                description = "Create categories to organize your transactions into income and expense types.",
                actionLabel = "Create a category",
                onAction = { showDialog = true }
            )
        }
    }

    if (showDialog) {
        var name by remember { mutableStateOf("") }
        var description by remember { mutableStateOf("") }
        var txnType by remember { mutableStateOf("expense") }
        var color by remember { mutableStateOf("#6C5CE7") }
        val colors = listOf(
            "#6C5CE7", "#00B894", "#E17055", "#FDCB6E", "#74B9FF", "#00CEC9", "#FD79A8", "#636E72",
            "#FF6B6B", "#48DBFB", "#FF9FF3", "#54A0FF", "#5F27CD", "#01A3A4", "#F368E0", "#EE5A24",
            "#0ABDE3", "#10AC84", "#222F3E", "#ED4C67", "#12CBC4", "#B71540", "#5758BB", "#FFC312"
        )

        AlertDialog(
            onDismissRequest = { showDialog = false },
            title = { Text("New Category") },
            text = {
                Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    KhataField(value = name, onValueChange = { name = it }, placeholder = "Name")
                    KhataField(value = description, onValueChange = { description = it }, placeholder = "Description")
                    Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        FilterChip(selected = txnType == "expense", onClick = { txnType = "expense" }, label = { Text("Expense", fontSize = 12.sp) })
                        FilterChip(selected = txnType == "income", onClick = { txnType = "income" }, label = { Text("Income", fontSize = 12.sp) })
                    }
                    Text("Color", fontSize = 11.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    val rows = colors.chunked(6)
                    rows.forEach { row ->
                        Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                            row.forEach { c ->
                                val isSelected = color == c
                                val cColor = try { Color(android.graphics.Color.parseColor(c)) } catch (_: Exception) { Color.Gray }
                                Surface(
                                    onClick = { color = c },
                                    shape = CircleShape,
                                    modifier = Modifier.size(28.dp),
                                    color = cColor
                                ) {
                                    Box(contentAlignment = Alignment.Center) {
                                        if (isSelected) Icon(Icons.Default.Check, contentDescription = null, modifier = Modifier.size(16.dp), tint = Color.White)
                                    }
                                }
                            }
                        }
                        Spacer(Modifier.height(4.dp))
                    }
                    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                        KhataField(value = color, onValueChange = { newColor ->
                            if (newColor.startsWith("#") && newColor.length in 4..7) color = newColor
                        }, placeholder = "Hex", modifier = Modifier.weight(1f))
                        Box(Modifier.size(28.dp).clip(CircleShape).background(try { Color(android.graphics.Color.parseColor(color)) } catch (_: Exception) { Color.Gray }))
                        Text("Custom hex color", fontSize = 10.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                }
            },
            confirmButton = {
                KhataButton(onClick = { onCreate(name, txnType, color, description); showDialog = false }, variant = ButtonVariant.Ghost) { Text("Create") }
            },
            dismissButton = { KhataButton(onClick = { showDialog = false }, variant = ButtonVariant.Ghost) { Text("Cancel") } }
        )
    }
}
