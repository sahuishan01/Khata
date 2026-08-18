package com.khata.app.util

import android.content.Context
import android.os.Build
import java.io.PrintWriter
import java.io.StringWriter
import java.text.SimpleDateFormat
import java.util.*

object CrashLogWriter {

    private const val PREFS_NAME = "khata_crash_log"
    private const val KEY_LAST_CRASH = "last_crash"
    private const val DIR_NAME = "crash_logs"
    private const val MAX_LOG_FILES = 10

    fun install(context: Context) {
        val appContext = context.applicationContext
        Thread.setDefaultUncaughtExceptionHandler { thread, throwable ->
            try {
                val report = buildReport(appContext, thread, throwable)
                // Write to SharedPreferences (always accessible without root)
                appContext.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
                    .edit().putString(KEY_LAST_CRASH, report).apply()
                // Also try writing to file
                try { writeFile(appContext, report) } catch (_: Exception) {}
            } catch (_: Exception) {
            }
            val default = Thread.getDefaultUncaughtExceptionHandler()
            default?.uncaughtException(thread, throwable)
        }
    }

    fun getLastCrash(context: Context): String? {
        return context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
            .getString(KEY_LAST_CRASH, null)
    }

    fun clearLastCrash(context: Context) {
        context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
            .edit().remove(KEY_LAST_CRASH).apply()
    }

    fun writeInfo(context: Context, tag: String, message: String) {
        try {
            val dir = getLogDir(context)
            dir.mkdirs()
            val timestamp = SimpleDateFormat("yyyyMMdd_HHmmss", Locale.US).format(Date())
            val file = java.io.File(dir, "info_${timestamp}_$tag.txt")
            file.writeText(buildString {
                appendLine("=== Khata Info Log ===")
                appendLine("Timestamp: ${SimpleDateFormat("yyyy-MM-dd HH:mm:ss z", Locale.US).format(Date())}")
                appendLine("Tag: $tag")
                appendLine()
                appendLine(message)
            })
            pruneLogs(dir)
        } catch (_: Exception) {}
    }

    fun listLogs(context: Context): List<java.io.File> {
        val dir = getLogDir(context)
        if (!dir.exists()) return emptyList()
        return dir.listFiles()?.sortedByDescending { it.lastModified() } ?: emptyList()
    }

    fun readLog(file: java.io.File): String {
        return try { file.readText() } catch (e: Exception) { "Failed to read: ${e.message}" }
    }

    fun deleteAllLogs(context: Context) {
        getLogDir(context).listFiles()?.forEach { it.delete() }
    }

    private fun buildReport(context: Context, thread: Thread, throwable: Throwable): String {
        val sw = StringWriter()
        val pw = PrintWriter(sw)
        pw.println("=== Khata Crash Report ===")
        pw.println("Time: ${SimpleDateFormat("yyyy-MM-dd HH:mm:ss z", Locale.US).format(Date())}")
        pw.println("Thread: ${thread.name}")
        pw.println("Exception: ${throwable.javaClass.name}")
        pw.println("Message: ${throwable.message}")
        pw.println()
        pw.println("--- Device ---")
        pw.println("Manufacturer: ${Build.MANUFACTURER}")
        pw.println("Model: ${Build.MODEL}")
        pw.println("Device: ${Build.DEVICE}")
        pw.println("Android: ${Build.VERSION.RELEASE} (API ${Build.VERSION.SDK_INT})")
        pw.println("App: ${try { context.packageManager.getPackageInfo(context.packageName, 0).versionName } catch (_: Exception) { "?" }}")
        pw.println()
        pw.println("--- Stack Trace ---")
        throwable.printStackTrace(pw)
        throwable.cause?.let {
            pw.println()
            pw.println("--- Caused by ---")
            pw.println("${it.javaClass.name}: ${it.message}")
            it.printStackTrace(pw)
        }
        pw.flush()
        return sw.toString()
    }

    private fun writeFile(context: Context, report: String) {
        val dir = getLogDir(context)
        dir.mkdirs()
        val timestamp = SimpleDateFormat("yyyyMMdd_HHmmss", Locale.US).format(Date())
        java.io.File(dir, "crash_$timestamp.txt").writeText(report)
        pruneLogs(dir)
    }

    private fun getLogDir(context: Context): java.io.File {
        return java.io.File(context.getExternalFilesDir(null), DIR_NAME)
    }

    private fun pruneLogs(dir: java.io.File) {
        val files = dir.listFiles()?.sortedByDescending { it.lastModified() } ?: return
        if (files.size > MAX_LOG_FILES) {
            files.drop(MAX_LOG_FILES).forEach { it.delete() }
        }
    }
}
