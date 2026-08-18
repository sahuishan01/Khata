package com.khata.app.api

import android.content.Context
import android.content.SharedPreferences
import androidx.security.crypto.EncryptedSharedPreferences
import androidx.security.crypto.MasterKey
import com.khata.app.BuildConfig
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class TokenManager @Inject constructor(
    @ApplicationContext private val context: Context
) {
    private val prefs: SharedPreferences by lazy { createPrefs() }
    private val serverPrefs: SharedPreferences by lazy {
        context.getSharedPreferences("khata_server", Context.MODE_PRIVATE)
    }

    private val _tokenFlow = MutableStateFlow(getTokenSync())
    private val _serverUrlFlow = MutableStateFlow(getServerUrl())

    val tokenFlow: Flow<String?> = _tokenFlow.asStateFlow()
    val serverUrlFlow: StateFlow<String> = _serverUrlFlow.asStateFlow()

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

    fun getServerUrl(): String {
        return serverPrefs.getString(SERVER_URL_KEY, null)?.takeIf { it.isNotBlank() }
            ?: BuildConfig.API_BASE_URL
    }

    fun setServerUrl(url: String) {
        val normalized = url.trimEnd('/')
        serverPrefs.edit().putString(SERVER_URL_KEY, normalized).apply()
        _serverUrlFlow.value = normalized
    }

    fun getServerUrlSync(): String = getServerUrl()

    companion object {
        private const val TOKEN_KEY = "auth_token"
        private const val SERVER_URL_KEY = "server_url"
    }
}
