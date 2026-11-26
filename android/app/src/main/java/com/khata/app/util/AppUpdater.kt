package com.khata.app.util

import android.content.Context
import android.content.Intent
import android.net.Uri
import androidx.core.content.FileProvider
import com.khata.app.BuildConfig
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import okhttp3.OkHttpClient
import okhttp3.Request
import org.json.JSONObject
import java.io.File

data class ReleaseInfo(
    val version: String,
    val apkUrl: String?,
    val releaseUrl: String,
    val isNewer: Boolean,
)

object AppUpdater {
    private const val LATEST_RELEASE_API = "https://api.github.com/repos/sahuishan01/Khata/releases/latest"
    private val client = OkHttpClient()

    suspend fun checkLatestRelease(): Result<ReleaseInfo> = withContext(Dispatchers.IO) {
        runCatching {
            val req = Request.Builder()
                .url(LATEST_RELEASE_API)
                .header("Accept", "application/vnd.github+json")
                .build()
            client.newCall(req).execute().use { resp ->
                if (!resp.isSuccessful) error("GitHub API ${resp.code}")
                val json = JSONObject(resp.body?.string().orEmpty())
                val tag = json.optString("tag_name").removePrefix("v")
                val releaseUrl = json.optString("html_url")
                var apkUrl: String? = null
                val assets = json.optJSONArray("assets")
                if (assets != null) {
                    for (i in 0 until assets.length()) {
                        val asset = assets.getJSONObject(i)
                        val name = asset.optString("name")
                        if (name.endsWith(".apk", ignoreCase = true)) {
                            apkUrl = asset.optString("browser_download_url")
                            break
                        }
                    }
                }
                val current = BuildConfig.VERSION_NAME.removePrefix("v")
                ReleaseInfo(
                    version = tag,
                    apkUrl = apkUrl,
                    releaseUrl = releaseUrl,
                    isNewer = isVersionNewer(tag, current),
                )
            }
        }
    }

    suspend fun downloadAndInstall(context: Context, apkUrl: String): Result<Unit> = withContext(Dispatchers.Main) {
        runCatching {
            val intent = Intent(Intent.ACTION_VIEW, Uri.parse(apkUrl)).apply {
                addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)
            }
            context.startActivity(intent)
        }
    }

    internal fun isVersionNewer(remote: String, current: String): Boolean {
        val r = remote.split(".").mapNotNull { it.toIntOrNull() }
        val c = current.split(".").mapNotNull { it.toIntOrNull() }
        val maxLen = maxOf(r.size, c.size)
        for (i in 0 until maxLen) {
            val rv = r.getOrElse(i) { 0 }
            val cv = c.getOrElse(i) { 0 }
            if (rv != cv) return rv > cv
        }
        return false
    }
}
