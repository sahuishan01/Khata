package com.khata.app.shortcuts

import android.appwidget.AppWidgetManager
import android.content.ComponentName
import android.content.Intent
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import com.khata.app.R
import com.khata.app.ui.theme.KhataTheme

class ShortcutWidgetConfigureActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setResult(RESULT_CANCELED)
        val id = intent.getIntExtra(AppWidgetManager.EXTRA_APPWIDGET_ID, AppWidgetManager.INVALID_APPWIDGET_ID)
        val manager = AppWidgetManager.getInstance(this)
        if (id == AppWidgetManager.INVALID_APPWIDGET_ID ||
            manager.getAppWidgetInfo(id)?.provider != ComponentName(this, SingleShortcutWidgetProvider::class.java)) {
            finish()
            return
        }
        setContent {
            KhataTheme(darkTheme = isSystemInDarkTheme()) {
                Surface(Modifier.fillMaxSize()) {
                    Column(Modifier.padding(24.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
                        Text(stringResource(R.string.widget_choose_shortcut), style = MaterialTheme.typography.headlineSmall)
                        ShortcutAction.entries.forEach { action ->
                            OutlinedButton(onClick = {
                                getSharedPreferences(SingleShortcutWidgetProvider.PREFERENCES, MODE_PRIVATE)
                                    .edit().putString(id.toString(), action.name).apply()
                                SingleShortcutWidgetProvider.update(this@ShortcutWidgetConfigureActivity, manager, id)
                                setResult(RESULT_OK, Intent().putExtra(AppWidgetManager.EXTRA_APPWIDGET_ID, id))
                                finish()
                            }, modifier = Modifier.fillMaxWidth().heightIn(min = 48.dp)) {
                                Text(stringResource(action.labelRes))
                            }
                        }
                    }
                }
            }
        }
    }
}
