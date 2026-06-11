package com.khata.app.ui.theme

import android.content.Context
import androidx.datastore.preferences.core.booleanPreferencesKey
import androidx.datastore.preferences.core.edit
import androidx.datastore.preferences.preferencesDataStore
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.runBlocking
import javax.inject.Inject
import javax.inject.Singleton

private val Context.themeStore by preferencesDataStore(name = "theme_prefs")

@Singleton
class ThemeManager @Inject constructor(
    @ApplicationContext private val context: Context
) {
    companion object {
        private val DARK_KEY = booleanPreferencesKey("dark_mode")
    }

    val isDarkFlow: Flow<Boolean> = context.themeStore.data.map { prefs ->
        prefs[DARK_KEY] ?: false
    }

    suspend fun isDark(): Boolean = isDarkFlow.first()

    suspend fun setDark(dark: Boolean) {
        context.themeStore.edit { prefs ->
            prefs[DARK_KEY] = dark
        }
    }

    fun toggle() {
        val current = runBlocking { isDark() }
        runBlocking { setDark(!current) }
    }
}
