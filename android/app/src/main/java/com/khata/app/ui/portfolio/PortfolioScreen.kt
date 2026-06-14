package com.khata.app.ui.portfolio

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
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
import com.khata.app.ui.components.shared.ButtonVariant
import com.khata.app.ui.components.shared.KhataAmount
import com.khata.app.ui.components.shared.KhataButton
import com.khata.app.ui.components.shared.KhataCard
import com.khata.app.ui.components.shared.KhataCardBody
import com.khata.app.ui.components.shared.KhataEmptyState
import com.khata.app.ui.components.shared.KhataField

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
                    KhataCard(Modifier.weight(1f)) { KhataCardBody { Text("Assets", fontSize = 11.sp, color = MaterialTheme.colorScheme.onSurfaceVariant); KhataAmount(snapshot.totalAssets, size = 18.sp) } }
                    KhataCard(Modifier.weight(1f)) { KhataCardBody { Text("Liabilities", fontSize = 11.sp, color = MaterialTheme.colorScheme.onSurfaceVariant); KhataAmount(snapshot.totalLiabilities, size = 18.sp) } }
                    KhataCard(Modifier.weight(1f)) { KhataCardBody { Text("Net Worth", fontSize = 11.sp, color = MaterialTheme.colorScheme.onSurfaceVariant); KhataAmount(snapshot.netWorth, size = 18.sp) } }
                }
            }
        }

        if (error != null) item { Text(error, fontSize = 13.sp, color = MaterialTheme.colorScheme.error) }

        item {
            KhataCard(Modifier.fillMaxWidth()) {
                KhataCardBody {
                    Text("Add Asset", fontSize = 14.sp, fontWeight = FontWeight.SemiBold); Spacer(Modifier.height(8.dp))
                    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                        KhataField(value = aName, onValueChange = { aName = it }, placeholder = "Name", modifier = Modifier.weight(1f))
                        KhataField(value = aVal, onValueChange = { aVal = it }, placeholder = "Value", modifier = Modifier.weight(1f))
                        KhataButton(onClick = { onCreateAsset(aName, aType, aVal.toDoubleOrNull() ?: 0.0); aName = ""; aVal = "" }) { Icon(Icons.Default.Add, contentDescription = null) }
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
                    KhataEmptyState(
                        emoji = "\uD83D\uDCB0",
                        title = "No assets added",
                        description = "Add assets like bank accounts, mutual funds, or stocks above."
                    )
                }
            } else {
                snapshot.assets.forEach { a ->
                    item {
                        KhataCard(Modifier.fillMaxWidth()) {
                            Row(Modifier.padding(12.dp), verticalAlignment = Alignment.CenterVertically) {
                                Icon(Icons.Default.TrendingUp, contentDescription = null, modifier = Modifier.size(18.dp), tint = MaterialTheme.colorScheme.secondary)
                                Spacer(Modifier.width(8.dp))
                                Column(Modifier.weight(1f)) { Text(a.name, fontSize = 13.sp, fontWeight = FontWeight.Medium); Text(a.assetType, fontSize = 11.sp, color = MaterialTheme.colorScheme.onSurfaceVariant) }
                                KhataAmount(a.value, size = 14.sp)
                                Spacer(Modifier.width(4.dp))
                                KhataButton(onClick = { onDeleteAsset(a.id) }, variant = ButtonVariant.Ghost) { Icon(Icons.Default.Delete, contentDescription = null, modifier = Modifier.size(16.dp), tint = MaterialTheme.colorScheme.error) }
                            }
                        }
                    }
                }
            }
        }

        item { HorizontalDivider(); Spacer(Modifier.height(4.dp)) }

        item {
            KhataCard(Modifier.fillMaxWidth()) {
                KhataCardBody {
                    Text("Add Liability", fontSize = 14.sp, fontWeight = FontWeight.SemiBold); Spacer(Modifier.height(8.dp))
                    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                        KhataField(value = lName, onValueChange = { lName = it }, placeholder = "Name", modifier = Modifier.weight(1f))
                        KhataField(value = lVal, onValueChange = { lVal = it }, placeholder = "Value", modifier = Modifier.weight(1f))
                        KhataButton(onClick = { onCreateLiability(lName, lType, lVal.toDoubleOrNull() ?: 0.0); lName = ""; lVal = "" }) { Icon(Icons.Default.Add, contentDescription = null) }
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
                    KhataEmptyState(
                        emoji = "\uD83D\uDCB3",
                        title = "No liabilities added",
                        description = "Add liabilities like loans or credit cards above."
                    )
                }
            } else {
                snapshot.liabilities.forEach { l ->
                    item {
                        KhataCard(Modifier.fillMaxWidth()) {
                            Row(Modifier.padding(12.dp), verticalAlignment = Alignment.CenterVertically) {
                                Icon(Icons.Default.TrendingDown, contentDescription = null, modifier = Modifier.size(18.dp), tint = MaterialTheme.colorScheme.error)
                                Spacer(Modifier.width(8.dp))
                                Column(Modifier.weight(1f)) { Text(l.name, fontSize = 13.sp, fontWeight = FontWeight.Medium); Text(l.liabilityType, fontSize = 11.sp, color = MaterialTheme.colorScheme.onSurfaceVariant) }
                                KhataAmount(l.value, size = 14.sp)
                                Spacer(Modifier.width(4.dp))
                                KhataButton(onClick = { onDeleteLiability(l.id) }, variant = ButtonVariant.Ghost) { Icon(Icons.Default.Delete, contentDescription = null, modifier = Modifier.size(16.dp), tint = MaterialTheme.colorScheme.error) }
                            }
                        }
                    }
                }
            }
        }
    }
}
