package com.khata.app.shortcuts

import android.annotation.SuppressLint
import android.os.Build
import android.service.quicksettings.Tile
import android.service.quicksettings.TileService

abstract class ShortcutTileService(private val shortcut: ShortcutAction) : TileService() {
    override fun onStartListening() {
        super.onStartListening()
        qsTile?.apply {
            // These are launch actions, not persistent on/off switches.
            state = Tile.STATE_INACTIVE
            contentDescription = getString(shortcut.labelRes)
            updateTile()
        }
    }

    override fun onClick() {
        super.onClick()
        if (isLocked) unlockAndRun { openShortcut() } else openShortcut()
    }

    @Suppress("DEPRECATION")
    @SuppressLint("StartActivityAndCollapseDeprecated") // The PendingIntent overload only exists on API 34+.
    private fun openShortcut() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
            startActivityAndCollapse(shortcut.pendingIntent(this))
        } else {
            startActivityAndCollapse(shortcut.launchIntent(this))
        }
    }
}

class AddTransactionTileService : ShortcutTileService(ShortcutAction.ADD_TRANSACTION)
class TransactionsTileService : ShortcutTileService(ShortcutAction.TRANSACTIONS)
class ChatTileService : ShortcutTileService(ShortcutAction.CHAT)
class AddDataTileService : ShortcutTileService(ShortcutAction.ADD_DATA)
