# Kotlin
-keepattributes *Annotation*
-keepattributes SourceFile,LineNumberTable
-keep class kotlin.** { *; }
-keep class org.jetbrains.kotlin.** { *; }
-dontwarn kotlin.**

# Compose
-dontwarn androidx.compose.**
-keep class androidx.compose.** { *; }

# Hilt / Dagger
-dontwarn dagger.**
-keep class dagger.** { *; }
-keep class javax.inject.** { *; }
-keep class * extends dagger.hilt.android.internal.managers.ViewComponentManager$FragmentContextWrapper { *; }

# Retrofit
-dontwarn retrofit2.**
-keep class retrofit2.** { *; }
-keepclasseswithmembers class * {
    @retrofit2.http.* <methods>;
}
-keepclassmembers,allowshrinking,allowobfuscation interface * {
    @retrofit2.http.* <methods>;
}

# OkHttp
-dontwarn okhttp3.**
-dontwarn okio.**
-keep class okhttp3.** { *; }
-keep class okio.** { *; }

# Gson
-keep class com.khata.app.api.** { *; }
-keepclassmembers class com.khata.app.api.** { *; }

# DataStore
-dontwarn androidx.datastore.**

# Coroutines
-dontwarn kotlinx.coroutines.**
-keep class kotlinx.coroutines.** { *; }

# Tink / EncryptedSharedPreferences
-dontwarn com.google.crypto.tink.**
-keep class com.google.crypto.tink.** { *; }
-dontwarn com.google.errorprone.**
-keep class com.google.errorprone.** { *; }
