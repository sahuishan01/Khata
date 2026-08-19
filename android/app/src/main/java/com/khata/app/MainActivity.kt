package com.khata.app

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.khata.app.ui.navigation.KhataNavHost
import com.khata.app.ui.theme.KhataTheme
import com.khata.app.ui.theme.ThemeManager
import com.khata.app.util.CrashLogWriter
import dagger.hilt.android.AndroidEntryPoint
import javax.inject.Inject

@AndroidEntryPoint
class MainActivity : ComponentActivity() {

    @Inject
    lateinit var themeManager: ThemeManager

    private val requestPermissionLauncher = registerForActivityResult(
        ActivityResultContracts.RequestMultiplePermissions()
    ) { permissions ->
        val receiveGranted = permissions[android.Manifest.permission.RECEIVE_SMS] ?: false
        val readGranted = permissions[android.Manifest.permission.READ_SMS] ?: false
        if (receiveGranted || readGranted) {
            // Permission granted, real-time SMS listening is active
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        requestSmsPermissions()

        setContent {
            val isDark by themeManager.isDarkFlow.collectAsState(initial = false)
            KhataTheme(darkTheme = isDark) {
                Surface(modifier = Modifier.fillMaxSize()) {
                    var showCrashDialog by remember { mutableStateOf(false) }
                    var crashReport by remember { mutableStateOf("") }

                    LaunchedEffect(Unit) {
                        CrashLogWriter.getLastCrash(this@MainActivity)?.let { report ->
                            crashReport = report
                            showCrashDialog = true
                            CrashLogWriter.clearLastCrash(this@MainActivity)
                        }
                    }

                    if (showCrashDialog) {
                        val clipboard = LocalClipboardManager.current
                        AlertDialog(
                            onDismissRequest = { showCrashDialog = false },
                            title = { Text("App crashed last time") },
                            text = {
                                Text(
                                    crashReport,
                                    fontSize = 10.sp,
                                    fontFamily = FontFamily.Monospace,
                                    lineHeight = 14.sp,
                                    modifier = Modifier
                                        .fillMaxWidth()
                                        .height(300.dp)
                                        .verticalScroll(rememberScrollState())
                                )
                            },
                            confirmButton = {
                                TextButton(onClick = {
                                    clipboard.setText(AnnotatedString(crashReport))
                                    showCrashDialog = false
                                }) { Text("Copy") }
                            },
                            dismissButton = {
                                TextButton(onClick = { showCrashDialog = false }) { Text("Dismiss") }
                            }
                        )
                    }

                    KhataNavHost(themeManager = themeManager)
                }
            }
        }
    }

    private fun requestSmsPermissions() {
        if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.M) {
            val receiveGranted = checkSelfPermission(android.Manifest.permission.RECEIVE_SMS) == android.content.pm.PackageManager.PERMISSION_GRANTED
            val readGranted = checkSelfPermission(android.Manifest.permission.READ_SMS) == android.content.pm.PackageManager.PERMISSION_GRANTED

            if (!receiveGranted || !readGranted) {
                requestPermissionLauncher.launch(
                    arrayOf(
                        android.Manifest.permission.RECEIVE_SMS,
                        android.Manifest.permission.READ_SMS
                    )
                )
            }
        }
    }
}
