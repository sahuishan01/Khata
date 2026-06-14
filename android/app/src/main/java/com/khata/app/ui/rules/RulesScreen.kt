package com.khata.app.ui.rules

import androidx.compose.foundation.clickable
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
import com.khata.app.api.CategoryRule
import com.khata.app.ui.components.shared.ButtonVariant
import com.khata.app.ui.components.shared.KhataButton
import com.khata.app.ui.components.shared.KhataCard
import com.khata.app.ui.components.shared.KhataCardBody
import com.khata.app.ui.components.shared.KhataEmptyState
import com.khata.app.ui.components.shared.KhataField

@Composable
fun RulesScreen(
    rules: List<CategoryRule>, isLoading: Boolean, error: String?,
    onLoad: () -> Unit, onCreate: (String, String) -> Unit, onDelete: (String) -> Unit, onApply: () -> Unit
) {
    LaunchedEffect(Unit) { onLoad() }
    var pattern by remember { mutableStateOf("") }; var category by remember { mutableStateOf("") }
    var patternDialog by remember { mutableStateOf<String?>(null) }

    patternDialog?.let { p ->
        AlertDialog(onDismissRequest = { patternDialog = null }, title = { Text("Keyword Pattern") }, text = { Text(p, fontFamily = androidx.compose.ui.text.font.FontFamily.Monospace) }, confirmButton = { KhataButton(onClick = { patternDialog = null }, variant = ButtonVariant.Ghost) { Text("OK") } })
    }

    LazyColumn(Modifier.fillMaxSize().padding(16.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
        item {
            Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
                Column { Text("Category Rules", fontSize = 20.sp, fontWeight = FontWeight.Bold); Text("Auto-categorize by payee keyword", fontSize = 13.sp, color = MaterialTheme.colorScheme.onSurfaceVariant) }
                KhataButton(onClick = onApply) { Icon(Icons.Default.Refresh, contentDescription = null, modifier = Modifier.size(16.dp)); Spacer(Modifier.width(4.dp)); Text("Apply", fontSize = 12.sp) }
            }
        }
        item {
            KhataCard(Modifier.fillMaxWidth()) {
                KhataCardBody {
                    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        KhataField(value = pattern, onValueChange = { pattern = it }, placeholder = "Keyword", modifier = Modifier.weight(1f))
                        KhataField(value = category, onValueChange = { category = it }, placeholder = "Category", modifier = Modifier.weight(1f))
                        KhataButton(onClick = { onCreate(pattern, category); pattern = ""; category = "" }) { Icon(Icons.Default.Add, contentDescription = null) }
                    }
                    if (error != null) { Spacer(Modifier.height(6.dp)); Text(error, fontSize = 12.sp, color = MaterialTheme.colorScheme.error) }
                }
            }
        }
        items(rules, key = { it.id }) { r ->
            KhataCard(Modifier.fillMaxWidth()) {
                Row(Modifier.padding(14.dp), verticalAlignment = Alignment.CenterVertically) {
                    Surface(shape = RoundedCornerShape(6.dp), color = MaterialTheme.colorScheme.primaryContainer, modifier = Modifier.widthIn(max = 200.dp).clickable { patternDialog = r.pattern }) { Text(r.pattern, modifier = Modifier.padding(horizontal = 8.dp, vertical = 4.dp), fontSize = 12.sp, fontFamily = androidx.compose.ui.text.font.FontFamily.Monospace, maxLines = 1, overflow = TextOverflow.Ellipsis, softWrap = false) }
                    Spacer(Modifier.width(8.dp))
                    Icon(Icons.Default.ArrowForward, contentDescription = null, modifier = Modifier.size(16.dp), tint = MaterialTheme.colorScheme.onSurfaceVariant)
                    Spacer(Modifier.width(8.dp))
                    Surface(shape = RoundedCornerShape(6.dp), color = MaterialTheme.colorScheme.secondaryContainer) { Text(r.category, modifier = Modifier.padding(horizontal = 8.dp, vertical = 4.dp), fontSize = 12.sp) }
                    Spacer(Modifier.weight(1f))
                    KhataButton(onClick = { onDelete(r.id) }, variant = ButtonVariant.Ghost) { Icon(Icons.Default.Delete, contentDescription = null, tint = MaterialTheme.colorScheme.error) }
                }
            }
        }
        if (rules.isEmpty()) item {
            KhataEmptyState(
                emoji = "\uD83D\uDCCB",
                title = "No rules yet",
                description = "Add keyword-based rules to auto-categorize transactions. E.g. ZOMATO -> Food & Dining.",
                actionLabel = "Add your first rule",
                onAction = {}
            )
        }
    }
}
