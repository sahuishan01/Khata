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

@Composable
fun RulesScreen(
    rules: List<CategoryRule>, isLoading: Boolean, error: String?,
    onLoad: () -> Unit, onCreate: (String, String) -> Unit, onDelete: (String) -> Unit, onApply: () -> Unit
) {
    LaunchedEffect(Unit) { onLoad() }
    var pattern by remember { mutableStateOf("") }; var category by remember { mutableStateOf("") }

    LazyColumn(Modifier.fillMaxSize().padding(16.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
        item {
            Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
                Column { Text("Category Rules", fontSize = 20.sp, fontWeight = FontWeight.Bold); Text("Auto-categorize by payee keyword", fontSize = 13.sp, color = MaterialTheme.colorScheme.onSurfaceVariant) }
                FilledTonalButton(onClick = onApply, modifier = Modifier.height(36.dp)) { Icon(Icons.Default.Refresh, contentDescription = null, modifier = Modifier.size(16.dp)); Spacer(Modifier.width(4.dp)); Text("Apply", fontSize = 12.sp) }
            }
        }
        item {
            Card(Modifier.fillMaxWidth(), shape = RoundedCornerShape(12.dp)) {
                Column(Modifier.padding(14.dp)) {
                    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        OutlinedTextField(value = pattern, onValueChange = { pattern = it }, label = { Text("Keyword", fontSize = 12.sp) }, singleLine = true, modifier = Modifier.weight(1f), textStyle = LocalTextStyle.current.copy(fontSize = 13.sp))
                        OutlinedTextField(value = category, onValueChange = { category = it }, label = { Text("Category", fontSize = 12.sp) }, singleLine = true, modifier = Modifier.weight(1f), textStyle = LocalTextStyle.current.copy(fontSize = 13.sp))
                        Button(onClick = { onCreate(pattern, category); pattern = ""; category = "" }, modifier = Modifier.height(52.dp)) { Icon(Icons.Default.Add, contentDescription = null) }
                    }
                    if (error != null) { Spacer(Modifier.height(6.dp)); Text(error, fontSize = 12.sp, color = MaterialTheme.colorScheme.error) }
                }
            }
        }
        var patternDialog by remember { mutableStateOf<String?>(null) }
        patternDialog?.let { p ->
            AlertDialog(onDismissRequest = { patternDialog = null }, title = { Text("Keyword Pattern") }, text = { Text(p, fontFamily = androidx.compose.ui.text.font.FontFamily.Monospace) }, confirmButton = { TextButton(onClick = { patternDialog = null }) { Text("OK") } })
        }
        items(rules, key = { it.id }) { r ->
            Card(Modifier.fillMaxWidth(), shape = RoundedCornerShape(10.dp)) {
                Row(Modifier.padding(14.dp), verticalAlignment = Alignment.CenterVertically) {
                    Surface(shape = RoundedCornerShape(6.dp), color = MaterialTheme.colorScheme.primaryContainer, modifier = Modifier.widthIn(max = 200.dp).clickable { patternDialog = r.pattern }) { Text(r.pattern, modifier = Modifier.padding(horizontal = 8.dp, vertical = 4.dp), fontSize = 12.sp, fontFamily = androidx.compose.ui.text.font.FontFamily.Monospace, maxLines = 1, overflow = TextOverflow.Ellipsis, softWrap = false) }
                    Spacer(Modifier.width(8.dp))
                    Icon(Icons.Default.ArrowForward, contentDescription = null, modifier = Modifier.size(16.dp), tint = MaterialTheme.colorScheme.onSurfaceVariant)
                    Spacer(Modifier.width(8.dp))
                    Surface(shape = RoundedCornerShape(6.dp), color = MaterialTheme.colorScheme.secondaryContainer) { Text(r.category, modifier = Modifier.padding(horizontal = 8.dp, vertical = 4.dp), fontSize = 12.sp) }
                    Spacer(Modifier.weight(1f))
                    IconButton(onClick = { onDelete(r.id) }) { Icon(Icons.Default.Delete, contentDescription = null, tint = MaterialTheme.colorScheme.error) }
                }
            }
        }
        if (rules.isEmpty()) item {
            Box(Modifier.fillMaxWidth().padding(vertical = 24.dp), contentAlignment = Alignment.Center) {
                Column(horizontalAlignment = Alignment.CenterHorizontally) {
                    Text("📋", fontSize = 32.sp)
                    Spacer(Modifier.height(8.dp))
                    Text("No rules yet", fontSize = 16.sp, fontWeight = FontWeight.SemiBold)
                    Spacer(Modifier.height(4.dp))
                    Text("Add keyword-based rules to auto-categorize transactions. E.g. ZOMATO → Food & Dining.", fontSize = 13.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.padding(horizontal = 24.dp))
                    Spacer(Modifier.height(16.dp))
                    Button(onClick = { /* form is always visible above */ }) { Text("Add your first rule") }
                }
            }
        }
    }
}
