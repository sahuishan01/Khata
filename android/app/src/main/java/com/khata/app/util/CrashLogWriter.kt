package com.khata.app.util

import android.content.Context
import android.os.Build
import java.io.File
import java.io.PrintWriter
import java.io.StringWriter
import java.text.SimpleDateFormat
import java.util.*

object CrashLogWriter {

    private const val DIR_NAME = "crash_logs"
    private const val MAX_LOG_FILES = 10

    fun install(context: Context) {
        val appContext = context.applicationContext
        Thread.setDefaultUncaughtExceptionHandler { thread, throwable ->
            try {
                writeCrashLog(appContext, thread, throwable)
            } catch (_: Exception) {
            }
            // Delegate to default handler (shows system crash dialog / kills process)
            val default = Thread.getDefaultUncaughtExceptionHandler()
            default?.uncaughtException(thread, throwable)
        }
    }

    fun writeCrashLog(context: Context, thread: Thread, throwable: Throwable) {
        val dir = getLogDir(context)
        dir.mkdirs()

        val timestamp = SimpleDateFormat("yyyyMMdd_HHmmss", Locale.US).format(Date())
        val file = File(dir, "crash_$timestamp.txt")

        val sw = StringWriter()
        val pw = PrintWriter(sw)
        pw.println("=== Khata Crash Log ===")
        pw.println("Timestamp: ${SimpleDateFormat("yyyy-MM-dd HH:mm:ss z", Locale.US).format(Date())}")
        pw.println("Thread: ${thread.name}")
        pw.println("Exception: ${throwable.javaClass.name}")
        pw.println("Message: ${throwable.message}")
        pw.println()
        pw.println("--- Device ---")
        pw.println("Manufacturer: ${Build.MANUFACTURER}")
        pw.println("Model: ${Build.MODEL}")
        pw.println("Device: ${Build.DEVICE}")
        pw.println("Android: ${Build.VERSION.RELEASE} (API ${Build.VERSION.SDK_INT})")
        pw.println("App version: ${getAppVersion(context)}")
        pw.println()
        pw.println("--- Stack Trace ---")
        throwable.printStackTrace(pw)
        pw.println()
        pw.println("--- Cause ---")
        throwable.cause?.let {
            pw.println("Caused by: ${it.javaClass.name}: ${it.message}")
            it.printStackTrace(pw)
        } ?: pw.println("No cause chain")
        pw.flush()

        file.writeText(sw.toString())

        // Prune old logs
        pruneLogs(dir)
    }

    fun writeInfo(context: Context, tag: String, message: String) {
        val dir = getLogDir(context)
        dir.mkdirs()

        val timestamp = SimpleDateFormat("yyyyMMdd_HHmmss", Locale.US).format(Date())
        val file = File(dir, "info_${timestamp}_$tag.txt")
        file.writeText(buildString {
            appendLine("=== Khata Info Log ===")
            appendLine("Timestamp: ${SimpleDateFormat("yyyy-MM-dd HH:mm:ss z", Locale.US).format(Date())}")
            appendLine("Tag: $tag")
            appendLine()
            appendLine(message)
        })

        pruneLogs(dir)
    }

    fun getLogDir(context: Context): File {
        return File(context.getExternalFilesDir(null), DIR_NAME)
    }

    fun listLogs(context: Context): List<File> {
        val dir = getLogDir(context)
        if (!dir.exists()) return emptyList()
        return dir.listFiles()?.sortedByDescending { it.lastModified() } ?: emptyList()
    }

    fun readLog(file: File): String {
        return try {
            file.readText()
        } catch (e: Exception) {
            "Failed to read log: ${e.message}"
        }
    }

    fun deleteAllLogs(context: Context) {
        getLogDir(context).listFiles()?.forEach { it.delete() }
    }

    private fun getAppVersion(context: Context): String {
        return try {
            val pInfo = context.packageManager.getPackageInfo(context.packageName, 0)
            "${pInfo.versionName} (${pInfo.longVersionCode})"
        } catch (_: Exception) {
            "unknown"
        }
    }

    private fun pruneLogs(dir: File) {
        val files = dir.listFiles()?.sortedByDescending { it.lastModified() } ?: return
        if (files.size > MAX_LOG_FILES) {
            files.drop(MAX_LOG_FILES).forEach { it.delete() }
        }
    }
}
