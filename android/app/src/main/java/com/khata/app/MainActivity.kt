package com.khata.app

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.Surface
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import com.khata.app.ui.navigation.KhataNavHost
import com.khata.app.ui.theme.KhataTheme
import com.khata.app.ui.theme.ThemeManager
import dagger.hilt.android.AndroidEntryPoint
import javax.inject.Inject

@AndroidEntryPoint
class MainActivity : ComponentActivity() {

    @Inject
    lateinit var themeManager: ThemeManager

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        setContent {
            val isDark by themeManager.isDarkFlow.collectAsState(initial = false)
            KhataTheme(darkTheme = isDark) {
                Surface(modifier = Modifier.fillMaxSize()) {
                    KhataNavHost(themeManager = themeManager)
                }
            }
        }
    }
}
