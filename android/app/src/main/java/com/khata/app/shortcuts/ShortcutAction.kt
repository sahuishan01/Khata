package com.khata.app.shortcuts

/** Only these destinations may be opened from outside the app. No financial data is carried. */
enum class ShortcutAction(val route: String) {
    ADD_TRANSACTION("add"),
    TRANSACTIONS("transactions"),
    CHAT("chat"),
    ADD_DATA("upload");

    val intentAction: String get() = "com.khata.app.shortcut.$name"

    companion object {
        fun fromIntentAction(action: String?): ShortcutAction? =
            entries.firstOrNull { it.intentAction == action }
    }
}

data class ShortcutNavigation(val route: String, val resetStack: Boolean, val consumeShortcut: Boolean = false)

/** Keep a requested shortcut pending until setup, sign-in and password reset are complete. */
fun resolveShortcutNavigation(
    isLoggedIn: Boolean,
    setupRequired: Boolean,
    mustResetPassword: Boolean,
    currentRoute: String?,
    shortcut: ShortcutAction?,
): ShortcutNavigation? {
    val gate = when {
        mustResetPassword -> "reset_password"
        !isLoggedIn -> if (setupRequired) "setup" else "login"
        else -> null
    }
    if (gate != null) return if (currentRoute == gate) null else ShortcutNavigation(gate, true)
    val onAuthScreen = currentRoute == null || currentRoute in setOf("setup", "login", "reset_password")
    if (shortcut != null) return ShortcutNavigation(shortcut.route, onAuthScreen, true)
    return if (onAuthScreen) ShortcutNavigation("dashboard", true) else null
}
