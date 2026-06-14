package com.khata.app.ui.portfolio

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.khata.app.api.NetWorthSnapshot
import com.khata.app.util.formatINR

@Composable
fun PortfolioScreen(
    snapshot: NetWorthSnapshot?, isLoading: Boolean, error: String?,
    onLoad: () -> Unit,
    onCreateAsset: (String, String, Double) -> Unit, onDeleteAsset: (String) -> Unit,
    onCreateLiability: (String, String, Double) -> Unit, onDeleteLiability: (String) -> Unit
) {
    LaunchedEffect(Unit) { onLoad() }
    var aName by remember { mutableStateOf("") }; var aType by remember { mutableStateOf("bank") }; var aVal by remember { mutableStateOf("") }
    var lName by remember { mutableStateOf("") }; var lType by remember { mutableStateOf("loan") }; var lVal by remember { mutableStateOf("") }

    LazyColumn(Modifier.fillMaxSize().padding(16.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
        item { Text("Portfolio", fontSize = 20.sp, fontWeight = FontWeight.Bold); Text("Net worth tracker", fontSize = 13.sp, color = MaterialTheme.colorScheme.onSurfaceVariant) }

        if (snapshot != null) {
            item {
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp), modifier = Modifier.fillMaxWidth()) {
                    Card(Modifier.weight(1f), shape = RoundedCornerShape(12.dp)) { Column(Modifier.padding(12.dp)) { Text("Assets", fontSize = 11.sp, color = MaterialTheme.colorScheme.onSurfaceVariant); Text(formatINR(snapshot.totalAssets), fontSize = 18.sp, fontWeight = FontWeight.Bold, color = MaterialTheme.colorScheme.secondary, style = LocalTextStyle.current.copy(fontFeatureSettings = "tnum")) } }
                    Card(Modifier.weight(1f), shape = RoundedCornerShape(12.dp)) { Column(Modifier.padding(12.dp)) { Text("Liabilities", fontSize = 11.sp, color = MaterialTheme.colorScheme.onSurfaceVariant); Text(formatINR(snapshot.totalLiabilities), fontSize = 18.sp, fontWeight = FontWeight.Bold, color = MaterialTheme.colorScheme.error, style = LocalTextStyle.current.copy(fontFeatureSettings = "tnum")) } }
                    Card(Modifier.weight(1f), shape = RoundedCornerShape(12.dp)) { Column(Modifier.padding(12.dp)) { Text("Net Worth", fontSize = 11.sp, color = MaterialTheme.colorScheme.onSurfaceVariant); Text(formatINR(snapshot.netWorth), fontSize = 18.sp, fontWeight = FontWeight.Bold, color = if (snapshot.netWorth >= 0) MaterialTheme.colorScheme.secondary else MaterialTheme.colorScheme.error, style = LocalTextStyle.current.copy(fontFeatureSettings = "tnum")) } }
                }
            }
        }

        if (error != null) item { Text(error, fontSize = 13.sp, color = MaterialTheme.colorScheme.error) }

        item {
            Card(Modifier.fillMaxWidth(), shape = RoundedCornerShape(12.dp)) {
                Column(Modifier.padding(14.dp)) {
                    Text("Add Asset", fontSize = 14.sp, fontWeight = FontWeight.SemiBold); Spacer(Modifier.height(8.dp))
                    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                        OutlinedTextField(value = aName, onValueChange = { aName = it }, label = { Text("Name", fontSize = 11.sp) }, singleLine = true, modifier = Modifier.weight(1f), textStyle = LocalTextStyle.current.copy(fontSize = 13.sp))
                        OutlinedTextField(value = aVal, onValueChange = { aVal = it }, label = { Text("Value", fontSize = 11.sp) }, singleLine = true, modifier = Modifier.width(80.dp), textStyle = LocalTextStyle.current.copy(fontSize = 13.sp))
                        Button(onClick = { onCreateAsset(aName, aType, aVal.toDoubleOrNull() ?: 0.0); aName = ""; aVal = "" }, modifier = Modifier.height(52.dp)) { Icon(Icons.Default.Add, contentDescription = null) }
                    }
                    Spacer(Modifier.height(4.dp))
                    Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                        listOf("bank","mutual_fund","stock","fd","cash").forEach { t -> FilterChip(selected = aType == t, onClick = { aType = t }, label = { Text(t.replace("_"," "), fontSize = 10.sp) }) }
                    }
                }
            }
        }

        if (snapshot != null) {
            if (snapshot.assets.isEmpty()) {
                item {
                    Box(Modifier.fillMaxWidth().padding(vertical = 16.dp), contentAlignment = Alignment.Center) {
                        Column(horizontalAlignment = Alignment.CenterHorizontally) {
                            Text("💰", fontSize = 28.sp)
                            Spacer(Modifier.height(4.dp))
                            Text("No assets added", fontSize = 14.sp, fontWeight = FontWeight.SemiBold)
                            Spacer(Modifier.height(2.dp))
                            Text("Add assets like bank accounts, mutual funds, or stocks above.", fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                        }
                    }
                }
            } else {
                snapshot.assets.forEach { a ->
                    item { Card(Modifier.fillMaxWidth(), shape = RoundedCornerShape(10.dp)) { Row(Modifier.padding(12.dp), verticalAlignment = Alignment.CenterVertically) { Icon(Icons.Default.TrendingUp, contentDescription = null, modifier = Modifier.size(18.dp), tint = MaterialTheme.colorScheme.secondary); Spacer(Modifier.width(8.dp)); Column(Modifier.weight(1f)) { Text(a.name, fontSize = 13.sp, fontWeight = FontWeight.Medium); Text(a.assetType, fontSize = 11.sp, color = MaterialTheme.colorScheme.onSurfaceVariant) }; Text(formatINR(a.value), fontSize = 14.sp, fontWeight = FontWeight.Bold, color = MaterialTheme.colorScheme.secondary, style = LocalTextStyle.current.copy(fontFeatureSettings = "tnum")); Spacer(Modifier.width(4.dp)); IconButton(onClick = { onDeleteAsset(a.id) }, modifier = Modifier.size(48.dp)) { Icon(Icons.Default.Delete, contentDescription = null, modifier = Modifier.size(16.dp), tint = MaterialTheme.colorScheme.error) } } } }
                }
            }
        }

        item { HorizontalDivider(); Spacer(Modifier.height(4.dp)) }

        item {
            Card(Modifier.fillMaxWidth(), shape = RoundedCornerShape(12.dp)) {
                Column(Modifier.padding(14.dp)) {
                    Text("Add Liability", fontSize = 14.sp, fontWeight = FontWeight.SemiBold); Spacer(Modifier.height(8.dp))
                    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                        OutlinedTextField(value = lName, onValueChange = { lName = it }, label = { Text("Name", fontSize = 11.sp) }, singleLine = true, modifier = Modifier.weight(1f), textStyle = LocalTextStyle.current.copy(fontSize = 13.sp))
                        OutlinedTextField(value = lVal, onValueChange = { lVal = it }, label = { Text("Value", fontSize = 11.sp) }, singleLine = true, modifier = Modifier.width(80.dp), textStyle = LocalTextStyle.current.copy(fontSize = 13.sp))
                        Button(onClick = { onCreateLiability(lName, lType, lVal.toDoubleOrNull() ?: 0.0); lName = ""; lVal = "" }, modifier = Modifier.height(52.dp)) { Icon(Icons.Default.Add, contentDescription = null) }
                    }
                    Spacer(Modifier.height(4.dp))
                    Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                        listOf("loan","credit_card","other").forEach { t -> FilterChip(selected = lType == t, onClick = { lType = t }, label = { Text(t.replace("_"," "), fontSize = 10.sp) }) }
                    }
                }
            }
        }

        if (snapshot != null) {
            if (snapshot.liabilities.isEmpty()) {
                item {
                    Box(Modifier.fillMaxWidth().padding(vertical = 16.dp), contentAlignment = Alignment.Center) {
                        Column(horizontalAlignment = Alignment.CenterHorizontally) {
                            Text("💳", fontSize = 28.sp)
                            Spacer(Modifier.height(4.dp))
                            Text("No liabilities added", fontSize = 14.sp, fontWeight = FontWeight.SemiBold)
                            Spacer(Modifier.height(2.dp))
                            Text("Add liabilities like loans or credit cards above.", fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                        }
                    }
                }
            } else {
                snapshot.liabilities.forEach { l ->
                    item { Card(Modifier.fillMaxWidth(), shape = RoundedCornerShape(10.dp)) { Row(Modifier.padding(12.dp), verticalAlignment = Alignment.CenterVertically) { Icon(Icons.Default.TrendingDown, contentDescription = null, modifier = Modifier.size(18.dp), tint = MaterialTheme.colorScheme.error); Spacer(Modifier.width(8.dp)); Column(Modifier.weight(1f)) { Text(l.name, fontSize = 13.sp, fontWeight = FontWeight.Medium); Text(l.liabilityType, fontSize = 11.sp, color = MaterialTheme.colorScheme.onSurfaceVariant) }; Text(formatINR(l.value), fontSize = 14.sp, fontWeight = FontWeight.Bold, color = MaterialTheme.colorScheme.error, style = LocalTextStyle.current.copy(fontFeatureSettings = "tnum")); Spacer(Modifier.width(4.dp)); IconButton(onClick = { onDeleteLiability(l.id) }, modifier = Modifier.size(48.dp)) { Icon(Icons.Default.Delete, contentDescription = null, modifier = Modifier.size(16.dp), tint = MaterialTheme.colorScheme.error) } } } }
                }
            }
        }
    }
}
