package com.khata.app

import android.app.Application
import com.khata.app.util.CrashLogWriter
import dagger.hilt.android.HiltAndroidApp

@HiltAndroidApp
class KhataApp : Application() {
    override fun onCreate() {
        super.onCreate()
        CrashLogWriter.install(this)
    }
}
