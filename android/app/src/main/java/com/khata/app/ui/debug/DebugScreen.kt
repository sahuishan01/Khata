package com.khata.app.ui.debug

import android.os.Build
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
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
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.khata.app.BuildConfig
import com.khata.app.util.CrashLogWriter
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import java.net.HttpURLConnection
import java.net.URL
import java.text.SimpleDateFormat
import java.util.*

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun DebugScreen(onBack: () -> Unit) {
    val context = LocalContext.current
    val scope = rememberCoroutineScope()

    var logs by remember { mutableStateOf(CrashLogWriter.listLogs(context)) }
    var selectedLog by remember { mutableStateOf<String?>(null) }
    var selectedLogName by remember { mutableStateOf("") }
    var connectivityResult by remember { mutableStateOf<String?>(null) }
    var isChecking by remember { mutableStateOf(false) }
    var showClearDialog by remember { mutableStateOf(false) }

    fun checkConnectivity() {
        isChecking = true
        connectivityResult = null
        scope.launch {
            val result = withContext(Dispatchers.IO) {
                try {
                    val url = URL("${BuildConfig.API_BASE_URL}/health")
                    val conn = url.openConnection() as HttpURLConnection
                    conn.connectTimeout = 10000
                    conn.readTimeout = 10000
                    conn.requestMethod = "GET"
                    val code = conn.responseCode
                    val body = try { conn.inputStream.bufferedReader().readText() } catch (_: Exception) { "" }
                    conn.disconnect()
                    if (code == 200) "OK (${code}) - $body"
                    else "FAIL (${code}) - $body"
                } catch (e: Exception) {
                    "ERROR: ${e.javaClass.simpleName}: ${e.message}"
                }
            }
            connectivityResult = result
            isChecking = false
        }
    }

    if (showClearDialog) {
        AlertDialog(
            onDismissRequest = { showClearDialog = false },
            title = { Text("Delete all logs?") },
            text = { Text("This will remove all crash and info logs.") },
            confirmButton = {
                TextButton(onClick = {
                    CrashLogWriter.deleteAllLogs(context)
                    logs = emptyList()
                    showClearDialog = false
                }) { Text("Delete", color = MaterialTheme.colorScheme.error) }
            },
            dismissButton = { TextButton(onClick = { showClearDialog = false }) { Text("Cancel") } }
        )
    }

    if (selectedLog != null) {
        // Log detail view
        Scaffold(topBar = {
            TopAppBar(
                title = { Text(selectedLogName, fontSize = 14.sp) },
                navigationIcon = { IconButton(onClick = { selectedLog = null }) { Icon(Icons.Default.ArrowBack, "Back") } }
            )
        }) { padding ->
            Column(
                Modifier
                    .fillMaxSize()
                    .padding(padding)
                    .verticalScroll(rememberScrollState())
                    .padding(12.dp)
            ) {
                Text(selectedLog!!, fontSize = 11.sp, fontFamily = FontFamily.Monospace, lineHeight = 16.sp)
            }
        }
        return
    }

    Scaffold(topBar = {
        TopAppBar(
            title = { Text("Debug & Logs") },
            navigationIcon = { IconButton(onClick = onBack) { Icon(Icons.Default.ArrowBack, "Back") } },
            actions = {
                if (logs.isNotEmpty()) {
                    IconButton(onClick = { showClearDialog = true }) { Icon(Icons.Default.DeleteSweep, "Clear logs") }
                }
            }
        )
    }) { padding ->
        LazyColumn(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding)
                .padding(horizontal = 16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
            contentPadding = PaddingValues(vertical = 12.dp)
        ) {
            // Connectivity section
            item {
                Card(
                    modifier = Modifier.fillMaxWidth(),
                    shape = RoundedCornerShape(12.dp),
                    colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surfaceVariant)
                ) {
                    Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                        Text("Server Connectivity", fontWeight = FontWeight.Bold, fontSize = 15.sp)
                        Text("API: ${BuildConfig.API_BASE_URL}", fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                        Button(
                            onClick = { checkConnectivity() },
                            enabled = !isChecking,
                            modifier = Modifier.fillMaxWidth()
                        ) {
                            if (isChecking) {
                                CircularProgressIndicator(Modifier.size(16.dp), strokeWidth = 2.dp, color = MaterialTheme.colorScheme.onPrimary)
                                Spacer(Modifier.width(8.dp))
                            }
                            Text("Test Connection")
                        }
                        connectivityResult?.let { result ->
                            val color = if (result.startsWith("OK")) MaterialTheme.colorScheme.primaryContainer
                            else MaterialTheme.colorScheme.errorContainer
                            Surface(shape = RoundedCornerShape(8.dp), color = color) {
                                Text(result, Modifier.padding(10.dp), fontSize = 12.sp, fontFamily = FontFamily.Monospace)
                            }
                        }
                    }
                }
            }

            // Device info section
            item {
                Card(
                    modifier = Modifier.fillMaxWidth(),
                    shape = RoundedCornerShape(12.dp),
                    colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surfaceVariant)
                ) {
                    Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                        Text("Device Info", fontWeight = FontWeight.Bold, fontSize = 15.sp)
                        val appVersion = try {
                            val pInfo = context.packageManager.getPackageInfo(context.packageName, 0)
                            "${pInfo.versionName} (${pInfo.longVersionCode})"
                        } catch (_: Exception) { "unknown" }
                        DebugInfoRow("App Version", appVersion)
                        DebugInfoRow("Package", context.packageName)
                        DebugInfoRow("Android", "${Build.VERSION.RELEASE} (API ${Build.VERSION.SDK_INT})")
                        DebugInfoRow("Manufacturer", Build.MANUFACTURER)
                        DebugInfoRow("Model", Build.MODEL)
                        DebugInfoRow("Build", Build.DISPLAY)
                    }
                }
            }

            // Crash logs section
            item {
                Row(
                    Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Text("Crash & Info Logs", fontWeight = FontWeight.Bold, fontSize = 15.sp)
                    Text("${logs.size} files", fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                }
            }

            if (logs.isEmpty()) {
                item {
                    Card(
                        modifier = Modifier.fillMaxWidth(),
                        shape = RoundedCornerShape(12.dp)
                    ) {
                        Box(Modifier.fillMaxWidth().padding(24.dp), contentAlignment = Alignment.Center) {
                            Column(horizontalAlignment = Alignment.CenterHorizontally) {
                                Icon(Icons.Default.CheckCircle, null, tint = MaterialTheme.colorScheme.primary, modifier = Modifier.size(32.dp))
                                Spacer(Modifier.height(8.dp))
                                Text("No crash logs found", fontSize = 13.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                            }
                        }
                    }
                }
            }

            items(logs, key = { it.name }) { file ->
                Card(
                    modifier = Modifier
                        .fillMaxWidth()
                        .clickable {
                            selectedLogName = file.name
                            selectedLog = CrashLogWriter.readLog(file)
                        },
                    shape = RoundedCornerShape(10.dp)
                ) {
                    Row(
                        Modifier.padding(12.dp),
                        verticalAlignment = Alignment.CenterVertically
                    ) {
                        Icon(
                            if (file.name.startsWith("crash")) Icons.Default.Error else Icons.Default.Info,
                            null,
                            tint = if (file.name.startsWith("crash")) MaterialTheme.colorScheme.error else MaterialTheme.colorScheme.primary,
                            modifier = Modifier.size(20.dp)
                        )
                        Spacer(Modifier.width(10.dp))
                        Column(Modifier.weight(1f)) {
                            Text(file.name, fontSize = 12.sp, fontFamily = FontFamily.Monospace, fontWeight = FontWeight.Medium)
                            Text(
                                SimpleDateFormat("yyyy-MM-dd HH:mm", Locale.US).format(Date(file.lastModified())),
                                fontSize = 11.sp,
                                color = MaterialTheme.colorScheme.onSurfaceVariant
                            )
                        }
                        Text(
                            formatFileSize(file.length()),
                            fontSize = 11.sp,
                            color = MaterialTheme.colorScheme.onSurfaceVariant
                        )
                    }
                }
            }

            // Bottom spacing
            item { Spacer(Modifier.height(16.dp)) }
        }
    }
}

@Composable
private fun DebugInfoRow(label: String, value: String) {
    Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
        Text(label, fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
        Text(value, fontSize = 12.sp, fontWeight = FontWeight.Medium)
    }
}

private fun formatFileSize(bytes: Long): String {
    return when {
        bytes < 1024 -> "$bytes B"
        bytes < 1024 * 1024 -> "${bytes / 1024} KB"
        else -> "${"%.1f".format(bytes / (1024.0 * 1024.0))} MB"
    }
}
