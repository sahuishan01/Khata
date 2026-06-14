package com.khata.app.ui.budgets

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.khata.app.api.Budget
import com.khata.app.api.BudgetStatus
import com.khata.app.ui.components.shared.ButtonVariant
import com.khata.app.ui.components.shared.KhataAmount
import com.khata.app.ui.components.shared.KhataButton
import com.khata.app.ui.components.shared.KhataCard
import com.khata.app.ui.components.shared.KhataCardBody
import com.khata.app.ui.components.shared.KhataEmptyState
import com.khata.app.ui.components.shared.KhataField
import com.khata.app.ui.components.shared.KhataProgressBar
import com.khata.app.ui.theme.KhataColors
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
            KhataCard(Modifier.fillMaxWidth()) {
                KhataCardBody {
                    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        KhataField(value = category, onValueChange = { category = it }, placeholder = "Category", modifier = Modifier.weight(1f))
                        KhataField(value = limit, onValueChange = { limit = it }, placeholder = "Limit (Rs)", modifier = Modifier.weight(1f))
                        KhataButton(onClick = { onCreate(category, limit.toDoubleOrNull() ?: 0.0); category = ""; limit = "" }) { Icon(Icons.Default.Add, contentDescription = null) }
                    }
                    if (error != null) { Spacer(Modifier.height(6.dp)); Text(error, fontSize = 12.sp, color = MaterialTheme.colorScheme.error) }
                }
            }
        }
        items(budgets, key = { it.id }) { b ->
            val s = getStatus(b.category); val pct = s?.pct ?: 0.0
            val barColor = when {
                pct >= 100 -> KhataColors.expense
                pct >= 80 -> KhataColors.warn
                else -> KhataColors.income
            }
            KhataCard(Modifier.fillMaxWidth()) {
                KhataCardBody {
                    Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
                        Surface(shape = RoundedCornerShape(6.dp), color = MaterialTheme.colorScheme.primaryContainer) { Text(b.category, modifier = Modifier.padding(horizontal = 8.dp, vertical = 4.dp), fontSize = 12.sp) }
                        Spacer(Modifier.width(8.dp))
                        Row(verticalAlignment = Alignment.CenterVertically) {
                            KhataAmount(b.monthlyLimit, size = 12.sp)
                            Text("/mo", fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                        }
                        Spacer(Modifier.weight(1f))
                        if (pct >= 80) Icon(Icons.Default.Warning, contentDescription = null, modifier = Modifier.size(16.dp), tint = barColor)
                        Text("${"%.0f".format(pct)}%", fontSize = 15.sp, fontWeight = FontWeight.Bold, color = barColor)
                        Spacer(Modifier.width(8.dp))
                        KhataButton(onClick = { onDelete(b.id) }, variant = ButtonVariant.Ghost) { Icon(Icons.Default.Delete, contentDescription = null, modifier = Modifier.size(16.dp), tint = MaterialTheme.colorScheme.error) }
                    }
                    Spacer(Modifier.height(8.dp))
                    KhataProgressBar(pct = pct)
                    if (s != null) Text("Spent: ${formatINR(s.spent)} / ${formatINR(b.monthlyLimit)}", fontSize = 11.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.padding(top = 4.dp))
                }
            }
        }
        if (budgets.isEmpty()) item {
            KhataEmptyState(
                emoji = "\uD83C\uDFAF",
                title = "No budgets set",
                description = "Set a monthly limit per category to get alerts before you overspend.",
                actionLabel = "Set your first budget",
                onAction = {}
            )
        }
    }
}
