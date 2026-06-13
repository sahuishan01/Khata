package com.khata.app.ui.accounts

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
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.khata.app.api.UserAccount
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
            Card(Modifier.fillMaxWidth(), shape = RoundedCornerShape(12.dp)) {
                Column(Modifier.padding(14.dp)) {
                    Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                        OutlinedTextField(value = label, onValueChange = { label = it }, label = { Text("Label", fontSize = 12.sp) }, singleLine = true, modifier = Modifier.weight(1f), textStyle = LocalTextStyle.current.copy(fontSize = 13.sp))
                        OutlinedTextField(value = identifier, onValueChange = { identifier = it }, label = { Text("UPI/Account", fontSize = 12.sp) }, singleLine = true, modifier = Modifier.weight(1f), textStyle = LocalTextStyle.current.copy(fontSize = 13.sp))
                        Button(onClick = { onCreate(label, identifier); label = ""; identifier = "" }, modifier = Modifier.height(52.dp)) { Icon(Icons.Default.Add, contentDescription = null) }
                    }
                    if (error != null) { Spacer(Modifier.height(6.dp)); Text(error, fontSize = 12.sp, color = MaterialTheme.colorScheme.error) }
                }
            }
        }
        item { Text("YOUR ACCOUNTS", fontSize = 11.sp, fontWeight = FontWeight.Bold, color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.6f), letterSpacing = 0.8.sp) }
        items(accounts, key = { it.id }) { a ->
            Card(Modifier.fillMaxWidth(), shape = RoundedCornerShape(10.dp)) {
                Row(Modifier.padding(14.dp), verticalAlignment = Alignment.CenterVertically) {
                    Icon(Icons.Default.AccountBalance, contentDescription = null, modifier = Modifier.size(20.dp), tint = MaterialTheme.colorScheme.primary)
                    Spacer(Modifier.width(10.dp))
                    Column(Modifier.weight(1f)) { Text(a.label, fontSize = 14.sp, fontWeight = FontWeight.Medium); Text(if (blurMode) maskIdentifier(a.identifier) else a.identifier, fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant) }
                    IconButton(onClick = { onDelete(a.id) }) { Icon(Icons.Default.Delete, contentDescription = null, tint = MaterialTheme.colorScheme.error) }
                }
            }
        }
        if (accounts.isEmpty()) item {
            Box(Modifier.fillMaxWidth().padding(vertical = 24.dp), contentAlignment = Alignment.Center) {
                Column(horizontalAlignment = Alignment.CenterHorizontally) {
                    Text("🏦", fontSize = 32.sp)
                    Spacer(Modifier.height(8.dp))
                    Text("No accounts added", fontSize = 16.sp, fontWeight = FontWeight.SemiBold)
                    Spacer(Modifier.height(4.dp))
                    Text("Link your bank, UPI, or wallet accounts to track all your balances and transfers in one place.", fontSize = 13.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.padding(horizontal = 24.dp))
                    Spacer(Modifier.height(16.dp))
                    Button(onClick = {}) { Text("Add an account") }
                }
            }
        }
    }
}
