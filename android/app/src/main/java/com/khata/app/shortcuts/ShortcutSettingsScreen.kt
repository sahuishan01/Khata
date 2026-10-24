package com.khata.app.shortcuts

import android.app.StatusBarManager
import android.appwidget.AppWidgetManager
import android.content.ComponentName
import android.graphics.drawable.Icon
import android.os.Build
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import com.khata.app.R

@Composable
fun ShortcutSettingsScreen(onBack: () -> Unit) {
    val context = LocalContext.current
    var message by remember { mutableStateOf<String?>(null) }
    Column(Modifier.fillMaxSize().verticalScroll(rememberScrollState()).padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp)) {
        TextButton(onClick = onBack) { Text(stringResource(R.string.shortcut_back)) }
        Text(stringResource(R.string.shortcut_settings_title), style = MaterialTheme.typography.headlineSmall)
        Text(stringResource(R.string.shortcut_privacy), color = MaterialTheme.colorScheme.onSurfaceVariant)
        Text(stringResource(R.string.shortcut_home_screen), style = MaterialTheme.typography.titleMedium)
        Text(stringResource(R.string.shortcut_widget_help))
        val widgetManager = remember { AppWidgetManager.getInstance(context) }
        if (widgetManager.isRequestPinAppWidgetSupported) {
            OutlinedButton(onClick = {
                val accepted = widgetManager.requestPinAppWidget(ComponentName(context, ShortcutWidgetProvider::class.java), null, null)
                message = context.getString(if (accepted) R.string.shortcut_widget_requested else R.string.shortcut_widget_manual)
            }) { Text(stringResource(R.string.shortcut_pin_widget)) }
        }
        Text(stringResource(R.string.shortcut_quick_settings), style = MaterialTheme.typography.titleMedium)
        Text(stringResource(R.string.shortcut_tile_help))
        ShortcutAction.entries.forEach { action ->
            OutlinedButton(onClick = {
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                    val service = when (action) {
                        ShortcutAction.ADD_TRANSACTION -> AddTransactionTileService::class.java
                        ShortcutAction.TRANSACTIONS -> TransactionsTileService::class.java
                        ShortcutAction.CHAT -> ChatTileService::class.java
                        ShortcutAction.ADD_DATA -> AddDataTileService::class.java
                    }
                    context.getSystemService(StatusBarManager::class.java).requestAddTileService(
                        ComponentName(context, service), context.getString(action.labelRes),
                        Icon.createWithResource(context, action.iconRes), context.mainExecutor,
                    ) { result ->
                        message = context.getString(when (result) {
                            StatusBarManager.TILE_ADD_REQUEST_RESULT_TILE_ADDED -> R.string.shortcut_tile_added
                            StatusBarManager.TILE_ADD_REQUEST_RESULT_TILE_ALREADY_ADDED -> R.string.shortcut_tile_already_added
                            StatusBarManager.TILE_ADD_REQUEST_RESULT_TILE_NOT_ADDED -> R.string.shortcut_tile_not_added
                            else -> R.string.shortcut_tile_manual
                        })
                    }
                } else message = context.getString(R.string.shortcut_tile_manual)
            }, modifier = Modifier.fillMaxWidth().heightIn(min = 48.dp)) {
                Text(stringResource(R.string.shortcut_add_tile, stringResource(action.labelRes)))
            }
        }
        message?.let { Text(it, color = MaterialTheme.colorScheme.primary) }
    }
}
