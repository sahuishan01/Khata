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
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.khata.app.BuildConfig
import com.khata.app.api.MeResponse
import com.khata.app.ui.components.shared.ButtonVariant
import com.khata.app.ui.components.shared.KhataButton
import com.khata.app.ui.components.shared.KhataCard
import com.khata.app.ui.components.shared.KhataCardBody
import com.khata.app.ui.components.shared.KhataField
import com.khata.app.ui.theme.KhataColors
import com.khata.app.ui.theme.MonoStyle
import com.khata.app.util.AppUpdater
import com.khata.app.util.ReleaseInfo
import kotlinx.coroutines.launch

@Composable
fun ProfileScreen(
    user: MeResponse?,
    isDark: Boolean,
    onToggleDark: () -> Unit,
    blurMode: Boolean,
    onToggleBlur: () -> Unit,
    serverUrl: String,
    onServerUrlChange: (String) -> Unit,
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
        AlertDialog(onDismissRequest = { showClearDialog = false },
            title = { Text("Clear All Data", color = MaterialTheme.colorScheme.error) },
            text = {
                Text("This will permanently delete server transactions, statements, chat history, and this device's saved and queued transactions. Your SMS inbox is kept and can be scanned again.", fontSize = 13.sp)
            },
            confirmButton = {
                KhataButton(onClick = { showClearDialog = false; onClearAllData() }, variant = ButtonVariant.Ghost) { Text("Clear Everything", color = MaterialTheme.colorScheme.error) }
            },
            dismissButton = { KhataButton(onClick = { showClearDialog = false }, variant = ButtonVariant.Ghost) { Text("Cancel") } })
    }

    if (showEmailDialog) {
        AlertDialog(onDismissRequest = { showEmailDialog = false }, title = { Text("Change Email") }, text = {
            KhataField(value = newEmail, onValueChange = { newEmail = it })
        }, confirmButton = { KhataButton(onClick = { onUpdateEmail(newEmail); showEmailDialog = false }, variant = ButtonVariant.Ghost) { Text("Save") } },
            dismissButton = { KhataButton(onClick = { showEmailDialog = false }, variant = ButtonVariant.Ghost) { Text("Cancel") } })
    }

    Column(Modifier.fillMaxSize().padding(16.dp).verticalScroll(rememberScrollState())) {
        Text("Settings", fontSize = 20.sp, fontWeight = FontWeight.Bold)
        Text("Account & preferences", fontSize = 13.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
        Spacer(Modifier.height(20.dp))

        // Account
        KhataCard(Modifier.fillMaxWidth()) {
            KhataCardBody {
                Text("Account", fontSize = 14.sp, fontWeight = FontWeight.SemiBold)
                Spacer(Modifier.height(10.dp))
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Surface(shape = RoundedCornerShape(10.dp), color = MaterialTheme.colorScheme.primaryContainer, modifier = Modifier.size(40.dp)) { Box(contentAlignment = Alignment.Center) { Icon(Icons.Default.Person, contentDescription = null, modifier = Modifier.size(20.dp), tint = MaterialTheme.colorScheme.onPrimaryContainer) } }
                    Spacer(Modifier.width(12.dp))
                    Column(Modifier.weight(1f)) { Text(user?.email ?: "Unknown", fontSize = 14.sp, fontWeight = FontWeight.Medium); Text(user?.role ?: "", fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant) }
                    KhataButton(onClick = { newEmail = user?.email ?: ""; showEmailDialog = true }, variant = ButtonVariant.Ghost) { Text("Change", fontSize = 12.sp) }
                }
            }
        }
        Spacer(Modifier.height(12.dp))

        // Theme
        KhataCard(Modifier.fillMaxWidth()) {
            KhataCardBody {
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
                Spacer(Modifier.height(12.dp))
                Text("Server URL", fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                Spacer(Modifier.height(4.dp))
                KhataField(value = serverUrl, onValueChange = onServerUrlChange, placeholder = "https://khata.algosculptor.com")
                Text("Restart app to apply changes", fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
        }
        Spacer(Modifier.height(12.dp))

        // App Updates
        val context = LocalContext.current
        val scope = rememberCoroutineScope()
        var checkingUpdate by remember { mutableStateOf(false) }
        var downloadingUpdate by remember { mutableStateOf(false) }
        var releaseInfo by remember { mutableStateOf<ReleaseInfo?>(null) }
        var updateStatus by remember { mutableStateOf<String?>(null) }

        KhataCard(Modifier.fillMaxWidth()) {
            KhataCardBody {
                Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
                    Text("App Updates", fontSize = 14.sp, fontWeight = FontWeight.SemiBold)
                    Text("v${BuildConfig.VERSION_NAME.removePrefix("v")}", style = MonoStyle, color = KhataColors.text2)
                }
                Spacer(Modifier.height(10.dp))
                KhataButton(
                    onClick = {
                        checkingUpdate = true
                        updateStatus = null
                        scope.launch {
                            AppUpdater.checkLatestRelease()
                                .onSuccess { info ->
                                    releaseInfo = info
                                    updateStatus = if (info.isNewer) "Update available: v${info.version}" else "You're on the latest version"
                                }
                                .onFailure { err ->
                                    updateStatus = "Update check failed: ${err.message}"
                                }
                            checkingUpdate = false
                        }
                    },
                    variant = ButtonVariant.Secondary,
                    enabled = !checkingUpdate && !downloadingUpdate,
                    modifier = Modifier.fillMaxWidth()
                ) {
                    Icon(Icons.Default.SystemUpdate, contentDescription = null, modifier = Modifier.size(16.dp))
                    Spacer(Modifier.width(6.dp))
                    Text(if (checkingUpdate) "Checking…" else "Check for updates")
                }
                val available = releaseInfo
                if (available != null && available.isNewer && available.apkUrl != null) {
                    Spacer(Modifier.height(8.dp))
                    KhataButton(
                        onClick = {
                            downloadingUpdate = true
                            updateStatus = "Downloading v${available.version}…"
                            scope.launch {
                                AppUpdater.downloadAndInstall(context, available.apkUrl)
                                    .onFailure { err -> updateStatus = "Install failed: ${err.message}" }
                                downloadingUpdate = false
                            }
                        },
                        variant = ButtonVariant.Primary,
                        enabled = !downloadingUpdate,
                        modifier = Modifier.fillMaxWidth()
                    ) {
                        Icon(Icons.Default.Download, contentDescription = null, modifier = Modifier.size(16.dp))
                        Spacer(Modifier.width(6.dp))
                        Text(if (downloadingUpdate) "Downloading…" else "Download & Install v${available.version}")
                    }
                }
                if (updateStatus != null) {
                    Spacer(Modifier.height(8.dp))
                    Text(
                        updateStatus!!,
                        fontSize = 12.sp,
                        color = if (releaseInfo?.isNewer == true) KhataColors.income else KhataColors.text2
                    )
                }
            }
        }
        Spacer(Modifier.height(12.dp))

        // Actions
        KhataCard(Modifier.fillMaxWidth()) {
            KhataCardBody {
                Text("Actions", fontSize = 14.sp, fontWeight = FontWeight.SemiBold)
                Spacer(Modifier.height(10.dp))
                KhataButton(onClick = onResetPassword, variant = ButtonVariant.Secondary, modifier = Modifier.fillMaxWidth()) { Icon(Icons.Default.Lock, contentDescription = null, modifier = Modifier.size(16.dp)); Spacer(Modifier.width(6.dp)); Text("Reset Password") }
                Spacer(Modifier.height(8.dp))
                KhataButton(onClick = { showClearDialog = true }, variant = ButtonVariant.Danger, modifier = Modifier.fillMaxWidth()) { Icon(Icons.Default.DeleteForever, contentDescription = null, modifier = Modifier.size(16.dp)); Spacer(Modifier.width(6.dp)); Text("Clear All Data") }
                Spacer(Modifier.height(8.dp))
                KhataButton(onClick = onLogout, variant = ButtonVariant.Secondary, modifier = Modifier.fillMaxWidth()) { Icon(Icons.Default.Logout, contentDescription = null, modifier = Modifier.size(16.dp)); Spacer(Modifier.width(6.dp)); Text("Logout") }
            }
        }

        if (msg.isNotBlank()) { Spacer(Modifier.height(8.dp)); Text(msg, fontSize = 13.sp, color = com.khata.app.ui.theme.KhataColors.income) }
    }
}
