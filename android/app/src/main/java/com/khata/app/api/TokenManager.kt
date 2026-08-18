package com.khata.app.api

import android.content.Context
import android.content.SharedPreferences
import androidx.security.crypto.EncryptedSharedPreferences
import androidx.security.crypto.MasterKey
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class TokenManager @Inject constructor(
    @ApplicationContext private val context: Context
) {
    private val prefs: SharedPreferences by lazy { createPrefs() }

    private val _tokenFlow = MutableStateFlow(getTokenSync())

    val tokenFlow: Flow<String?> = _tokenFlow.asStateFlow()

    suspend fun getToken(): String? = getTokenSync()

    fun getTokenSync(): String? = try {
        prefs.getString(TOKEN_KEY, null)
    } catch (_: Exception) {
        null
    }

    suspend fun saveToken(token: String) {
        try {
            prefs.edit().putString(TOKEN_KEY, token).apply()
        } catch (_: Exception) {}
        _tokenFlow.value = token
    }

    suspend fun clearToken() {
        try {
            prefs.edit().remove(TOKEN_KEY).apply()
        } catch (_: Exception) {}
        _tokenFlow.value = null
    }

    private fun createPrefs(): SharedPreferences {
        return try {
            val masterKey = MasterKey.Builder(context)
                .setKeyScheme(MasterKey.KeyScheme.AES256_GCM)
                .build()
            EncryptedSharedPreferences.create(
                context,
                "khata_secure_prefs",
                masterKey,
                EncryptedSharedPreferences.PrefKeyEncryptionScheme.AES256_SIV,
                EncryptedSharedPreferences.PrefValueEncryptionScheme.AES256_GCM
            )
        } catch (_: Exception) {
            // Keystore key corrupted (reinstall, OS update, etc.) — delete and retry
            context.deleteSharedPreferences("khata_secure_prefs")
            val masterKey = MasterKey.Builder(context)
                .setKeyScheme(MasterKey.KeyScheme.AES256_GCM)
                .build()
            EncryptedSharedPreferences.create(
                context,
                "khata_secure_prefs",
                masterKey,
                EncryptedSharedPreferences.PrefKeyEncryptionScheme.AES256_SIV,
                EncryptedSharedPreferences.PrefValueEncryptionScheme.AES256_GCM
            )
        }
    }

    companion object {
        private const val TOKEN_KEY = "auth_token"
    }
}
