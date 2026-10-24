package com.khata.app.shortcuts

import org.junit.Assert.*
import org.junit.Test

class ShortcutNavigationTest {
    @Test fun allEntrypointsUseAnExplicitAllowlist() {
        ShortcutAction.entries.forEach { assertEquals(it, ShortcutAction.fromIntentAction(it.intentAction)) }
        listOf(null, "", "chat", "com.khata.app.shortcut.ADMIN", "android.intent.action.MAIN").forEach {
            assertNull(ShortcutAction.fromIntentAction(it))
        }
    }

    @Test fun coldStartWaitsForLoginThenOpensTheRequestedScreen() {
        ShortcutAction.entries.forEach { action ->
            val login = resolveShortcutNavigation(false, false, false, "setup", action)!!
            assertEquals("login", login.route)
            assertFalse(login.consumeShortcut)
            assertNull(resolveShortcutNavigation(false, false, false, "login", action))
            val signedIn = resolveShortcutNavigation(true, false, false, "login", action)!!
            assertEquals(action.route, signedIn.route)
            assertTrue(signedIn.resetStack)
            assertTrue(signedIn.consumeShortcut)
        }
    }

    @Test fun setupAndForcedResetKeepTheShortcutPending() {
        assertFalse(resolveShortcutNavigation(false, true, false, null, ShortcutAction.CHAT)!!.consumeShortcut)
        val reset = resolveShortcutNavigation(true, false, true, "login", ShortcutAction.CHAT)!!
        assertEquals("reset_password", reset.route)
        assertFalse(reset.consumeShortcut)
        assertEquals("chat", resolveShortcutNavigation(true, false, false, "reset_password", ShortcutAction.CHAT)!!.route)
    }

    @Test fun aWarmLaunchNavigatesOnceAndNormalNavigationIsPreserved() {
        val next = resolveShortcutNavigation(true, false, false, "transactions", ShortcutAction.CHAT)!!
        assertEquals("chat", next.route)
        assertFalse(next.resetStack)
        assertNull(resolveShortcutNavigation(true, false, false, "chat", null))
        assertNull(resolveShortcutNavigation(true, false, false, "accounts", null))
        assertEquals("dashboard", resolveShortcutNavigation(true, false, false, "login", null)!!.route)
    }

    @Test fun logoutCannotLeaveAShortcutDestinationVisible() {
        assertEquals("login", resolveShortcutNavigation(false, false, false, "chat", null)!!.route)
    }
}
