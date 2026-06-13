package com.khata.app.ui.profile

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.khata.app.api.MeResponse

@Composable
fun ProfileScreen(
    user: MeResponse?,
    isDark: Boolean,
    onToggleDark: () -> Unit,
    blurMode: Boolean,
    onToggleBlur: () -> Unit,
    onResetPassword: () -> Unit,
    onClearAllData: () -> Unit,
    onUpdateEmail: (String) -> Unit,
    onLogout: () -> Unit
) {
    var showClearDialog by remember { mutableStateOf(false) }
    var clearConfirmText by remember { mutableStateOf("") }
    var showEmailDialog by remember { mutableStateOf(false) }
    var newEmail by remember { mutableStateOf(user?.email ?: "") }
    var msg by remember { mutableStateOf("") }

    if (showClearDialog) {
        AlertDialog(onDismissRequest = { showClearDialog = false; clearConfirmText = "" },
            title = { Text("Clear All Data", color = MaterialTheme.colorScheme.error) },
            text = {
                Column {
                    Text("This will permanently delete all transactions, statements, and chat history. This action cannot be undone.", fontSize = 13.sp)
                    Spacer(Modifier.height(12.dp))
                    Text("Type DELETE to confirm:", fontSize = 13.sp)
                    Spacer(Modifier.height(6.dp))
                    OutlinedTextField(value = clearConfirmText, onValueChange = { clearConfirmText = it }, placeholder = { Text("DELETE") }, singleLine = true, modifier = Modifier.fillMaxWidth())
                }
            },
            confirmButton = {
                TextButton(onClick = { showClearDialog = false; clearConfirmText = ""; onClearAllData() }, enabled = clearConfirmText == "DELETE") { Text("Clear Everything", color = MaterialTheme.colorScheme.error) }
            },
            dismissButton = { TextButton(onClick = { showClearDialog = false; clearConfirmText = "" }) { Text("Cancel") } })
    }

    if (showEmailDialog) {
        AlertDialog(onDismissRequest = { showEmailDialog = false }, title = { Text("Change Email") }, text = {
            OutlinedTextField(value = newEmail, onValueChange = { newEmail = it }, singleLine = true, modifier = Modifier.fillMaxWidth())
        }, confirmButton = { TextButton(onClick = { onUpdateEmail(newEmail); showEmailDialog = false }) { Text("Save") } },
            dismissButton = { TextButton(onClick = { showEmailDialog = false }) { Text("Cancel") } })
    }

    Column(Modifier.fillMaxSize().padding(16.dp).verticalScroll(rememberScrollState())) {
        Text("Settings", fontSize = 20.sp, fontWeight = FontWeight.Bold)
        Text("Account & preferences", fontSize = 13.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
        Spacer(Modifier.height(20.dp))

        // Account
        Card(Modifier.fillMaxWidth(), shape = RoundedCornerShape(14.dp)) {
            Column(Modifier.padding(16.dp)) {
                Text("Account", fontSize = 14.sp, fontWeight = FontWeight.SemiBold)
                Spacer(Modifier.height(10.dp))
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Surface(shape = RoundedCornerShape(10.dp), color = MaterialTheme.colorScheme.primaryContainer, modifier = Modifier.size(40.dp)) { Box(contentAlignment = Alignment.Center) { Icon(Icons.Default.Person, contentDescription = null, modifier = Modifier.size(20.dp), tint = MaterialTheme.colorScheme.onPrimaryContainer) } }
                    Spacer(Modifier.width(12.dp))
                    Column(Modifier.weight(1f)) { Text(user?.email ?: "Unknown", fontSize = 14.sp, fontWeight = FontWeight.Medium); Text(user?.role ?: "", fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant) }
                    TextButton(onClick = { newEmail = user?.email ?: ""; showEmailDialog = true }) { Text("Change", fontSize = 12.sp) }
                }
            }
        }
        Spacer(Modifier.height(12.dp))

        // Theme
        Card(Modifier.fillMaxWidth(), shape = RoundedCornerShape(14.dp)) {
            Column(Modifier.padding(16.dp)) {
                Text("Preferences", fontSize = 14.sp, fontWeight = FontWeight.SemiBold)
                Spacer(Modifier.height(10.dp))
                Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.SpaceBetween) {
                    Row(verticalAlignment = Alignment.CenterVertically) { Icon(if (isDark) Icons.Default.DarkMode else Icons.Default.LightMode, contentDescription = null, modifier = Modifier.size(18.dp)); Spacer(Modifier.width(10.dp)); Text("Dark Mode", fontSize = 14.sp) }
                    Switch(checked = isDark, onCheckedChange = { onToggleDark() })
                }
                Spacer(Modifier.height(10.dp))
                Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.SpaceBetween) {
                    Row(verticalAlignment = Alignment.CenterVertically) { Icon(Icons.Default.VisibilityOff, contentDescription = null, modifier = Modifier.size(18.dp)); Spacer(Modifier.width(10.dp)); Text("Privacy Mode", fontSize = 14.sp) }
                    Switch(checked = blurMode, onCheckedChange = { onToggleBlur() })
                }
            }
        }
        Spacer(Modifier.height(12.dp))

        // Actions
        Card(Modifier.fillMaxWidth(), shape = RoundedCornerShape(14.dp)) {
            Column(Modifier.padding(16.dp)) {
                Text("Actions", fontSize = 14.sp, fontWeight = FontWeight.SemiBold)
                Spacer(Modifier.height(10.dp))
                OutlinedButton(onClick = onResetPassword, modifier = Modifier.fillMaxWidth()) { Icon(Icons.Default.Lock, contentDescription = null, modifier = Modifier.size(16.dp)); Spacer(Modifier.width(6.dp)); Text("Reset Password") }
                Spacer(Modifier.height(8.dp))
                OutlinedButton(onClick = { showClearDialog = true }, modifier = Modifier.fillMaxWidth(), colors = ButtonDefaults.outlinedButtonColors(contentColor = MaterialTheme.colorScheme.error)) { Icon(Icons.Default.DeleteForever, contentDescription = null, modifier = Modifier.size(16.dp)); Spacer(Modifier.width(6.dp)); Text("Clear All Data") }
                Spacer(Modifier.height(8.dp))
                OutlinedButton(onClick = onLogout, modifier = Modifier.fillMaxWidth()) { Icon(Icons.Default.Logout, contentDescription = null, modifier = Modifier.size(16.dp)); Spacer(Modifier.width(6.dp)); Text("Logout") }
            }
        }

        if (msg.isNotBlank()) { Spacer(Modifier.height(8.dp)); Text(msg, fontSize = 13.sp, color = MaterialTheme.colorScheme.secondary) }
    }
}
