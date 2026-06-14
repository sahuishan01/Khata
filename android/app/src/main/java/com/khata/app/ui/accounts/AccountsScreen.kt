package com.khata.app.ui.accounts

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
import com.khata.app.api.UserAccount
import com.khata.app.ui.components.shared.ButtonVariant
import com.khata.app.ui.components.shared.KhataButton
import com.khata.app.ui.components.shared.KhataCard
import com.khata.app.ui.components.shared.KhataCardBody
import com.khata.app.ui.components.shared.KhataEmptyState
import com.khata.app.ui.components.shared.KhataField
import com.khata.app.util.maskIdentifier

@Composable
fun AccountsScreen(
    accounts: List<UserAccount>, isLoading: Boolean, error: String?,
    blurMode: Boolean = true,
    onLoad: () -> Unit, onCreate: (String, String) -> Unit, onDelete: (String) -> Unit
) {
    LaunchedEffect(Unit) { onLoad() }
    var label by remember { mutableStateOf("") }; var identifier by remember { mutableStateOf("") }

    LazyColumn(Modifier.fillMaxSize().padding(16.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
        item {
            Text("Your Accounts", fontSize = 20.sp, fontWeight = FontWeight.Bold)
            Text("Transfers between your own accounts won't count as income/expense", fontSize = 13.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
        }
        item {
            KhataCard(Modifier.fillMaxWidth()) {
                KhataCardBody {
                    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        KhataField(value = label, onValueChange = { label = it }, placeholder = "Label", modifier = Modifier.weight(1f))
                        KhataField(value = identifier, onValueChange = { identifier = it }, placeholder = "UPI/Account", modifier = Modifier.weight(1f))
                        KhataButton(onClick = { onCreate(label, identifier); label = ""; identifier = "" }) { Icon(Icons.Default.Add, contentDescription = null) }
                    }
                    if (error != null) { Spacer(Modifier.height(6.dp)); Text(error, fontSize = 12.sp, color = MaterialTheme.colorScheme.error) }
                }
            }
        }
        item { Text("YOUR ACCOUNTS", fontSize = 11.sp, fontWeight = FontWeight.Bold, color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f), letterSpacing = 0.8.sp) }
        items(accounts, key = { it.id }) { a ->
            KhataCard(Modifier.fillMaxWidth()) {
                Row(Modifier.padding(14.dp), verticalAlignment = Alignment.CenterVertically) {
                    Icon(Icons.Default.AccountBalance, contentDescription = null, modifier = Modifier.size(20.dp), tint = MaterialTheme.colorScheme.primary)
                    Spacer(Modifier.width(10.dp))
                    Column(Modifier.weight(1f)) { Text(a.label, fontSize = 14.sp, fontWeight = FontWeight.Medium); Text(if (blurMode) maskIdentifier(a.identifier) else a.identifier, fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant) }
                    KhataButton(onClick = { onDelete(a.id) }, variant = ButtonVariant.Ghost) { Icon(Icons.Default.Delete, contentDescription = null, tint = MaterialTheme.colorScheme.error) }
                }
            }
        }
        if (accounts.isEmpty()) item {
            KhataEmptyState(
                emoji = "\uD83C\uDFE6",
                title = "No accounts added",
                description = "Link your bank, UPI, or wallet accounts to track all your balances and transfers in one place.",
                actionLabel = "Add an account",
                onAction = {}
            )
        }
    }
}
