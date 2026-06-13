package com.khata.app.ui.budgets

import androidx.compose.foundation.background
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
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.khata.app.api.Budget
import com.khata.app.api.BudgetStatus
import com.khata.app.util.formatINR

@Composable
fun BudgetsScreen(
    budgets: List<Budget>, status: List<BudgetStatus>, isLoading: Boolean, error: String?,
    onLoad: () -> Unit, onCreate: (String, Double) -> Unit, onDelete: (String) -> Unit
) {
    LaunchedEffect(Unit) { onLoad() }
    var category by remember { mutableStateOf("") }; var limit by remember { mutableStateOf("") }

    fun getStatus(cat: String): BudgetStatus? = status.find { it.category == cat }

    LazyColumn(Modifier.fillMaxSize().padding(16.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
        item { Text("Budgets", fontSize = 20.sp, fontWeight = FontWeight.Bold); Text("Set monthly spending limits", fontSize = 13.sp, color = MaterialTheme.colorScheme.onSurfaceVariant) }
        item {
            Card(Modifier.fillMaxWidth(), shape = RoundedCornerShape(12.dp)) {
                Column(Modifier.padding(14.dp)) {
                    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        OutlinedTextField(value = category, onValueChange = { category = it }, label = { Text("Category", fontSize = 12.sp) }, singleLine = true, modifier = Modifier.weight(1f), textStyle = LocalTextStyle.current.copy(fontSize = 13.sp))
                        OutlinedTextField(value = limit, onValueChange = { limit = it }, label = { Text("Limit (₹)", fontSize = 12.sp) }, singleLine = true, modifier = Modifier.weight(1f), textStyle = LocalTextStyle.current.copy(fontSize = 13.sp))
                        Button(onClick = { onCreate(category, limit.toDoubleOrNull() ?: 0.0); category = ""; limit = "" }, modifier = Modifier.height(52.dp)) { Icon(Icons.Default.Add, contentDescription = null) }
                    }
                    if (error != null) { Spacer(Modifier.height(6.dp)); Text(error, fontSize = 12.sp, color = MaterialTheme.colorScheme.error) }
                }
            }
        }
        items(budgets, key = { it.id }) { b ->
            val s = getStatus(b.category); val pct = s?.pct ?: 0.0
            val barColor = if (pct >= 100) Color(0xFFEE6B4D) else if (pct >= 80) Color(0xFFE0A33A) else Color(0xFF2EC27E)
            Card(Modifier.fillMaxWidth(), shape = RoundedCornerShape(10.dp)) {
                Column(Modifier.padding(14.dp)) {
                    Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
                        Surface(shape = RoundedCornerShape(6.dp), color = MaterialTheme.colorScheme.primaryContainer) { Text(b.category, modifier = Modifier.padding(horizontal = 8.dp, vertical = 4.dp), fontSize = 12.sp) }
                        Spacer(Modifier.width(8.dp))
                        Text("${formatINR(b.monthlyLimit)}/mo", fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                        Spacer(Modifier.weight(1f))
                        if (pct >= 80) Icon(Icons.Default.Warning, contentDescription = null, modifier = Modifier.size(16.dp), tint = barColor)
                        Text("${"%.0f".format(pct)}%", fontSize = 15.sp, fontWeight = FontWeight.Bold, color = barColor)
                        Spacer(Modifier.width(8.dp))
                        IconButton(onClick = { onDelete(b.id) }, modifier = Modifier.size(48.dp)) { Icon(Icons.Default.Delete, contentDescription = null, modifier = Modifier.size(16.dp), tint = MaterialTheme.colorScheme.error) }
                    }
                    Spacer(Modifier.height(8.dp))
                    Box(Modifier.fillMaxWidth().height(6.dp).clip(RoundedCornerShape(3.dp)).background(MaterialTheme.colorScheme.outline.copy(alpha = 0.2f))) {
                        Box(Modifier.fillMaxWidth(fraction = (pct / 100.0).toFloat().coerceIn(0f, 1f)).height(6.dp).clip(RoundedCornerShape(3.dp)).background(barColor))
                    }
                    if (s != null) Text("Spent: ${formatINR(s.spent)} / ${formatINR(b.monthlyLimit)}", fontSize = 11.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.padding(top = 4.dp))
                }
            }
        }
        if (budgets.isEmpty()) item {
            Box(Modifier.fillMaxWidth().padding(vertical = 24.dp), contentAlignment = Alignment.Center) {
                Column(horizontalAlignment = Alignment.CenterHorizontally) {
                    Text("🎯", fontSize = 32.sp)
                    Spacer(Modifier.height(8.dp))
                    Text("No budgets set", fontSize = 16.sp, fontWeight = FontWeight.SemiBold)
                    Spacer(Modifier.height(4.dp))
                    Text("Set a monthly limit per category to get alerts before you overspend.", fontSize = 13.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.padding(horizontal = 24.dp))
                    Spacer(Modifier.height(16.dp))
                    Button(onClick = {}) { Text("Set your first budget") }
                }
            }
        }
    }
}
