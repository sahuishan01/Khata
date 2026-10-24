package com.khata.app.shortcuts

import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import com.khata.app.MainActivity
import com.khata.app.R

val ShortcutAction.labelRes: Int get() = when (this) {
    ShortcutAction.ADD_TRANSACTION -> R.string.shortcut_add_transaction
    ShortcutAction.TRANSACTIONS -> R.string.shortcut_transactions
    ShortcutAction.CHAT -> R.string.shortcut_chat
    ShortcutAction.ADD_DATA -> R.string.shortcut_add_data
}

val ShortcutAction.iconRes: Int get() = when (this) {
    ShortcutAction.ADD_TRANSACTION -> R.drawable.ic_shortcut_add
    ShortcutAction.TRANSACTIONS -> R.drawable.ic_shortcut_history
    ShortcutAction.CHAT -> R.drawable.ic_shortcut_chat
    ShortcutAction.ADD_DATA -> R.drawable.ic_shortcut_upload
}

val ShortcutAction.widgetLabelRes: Int get() = when (this) {
    ShortcutAction.ADD_TRANSACTION -> R.string.widget_label_add
    ShortcutAction.TRANSACTIONS -> R.string.widget_label_history
    ShortcutAction.CHAT -> R.string.widget_label_chat
    ShortcutAction.ADD_DATA -> R.string.shortcut_add_data
}

fun ShortcutAction.launchIntent(context: Context): Intent = Intent(context, MainActivity::class.java)
    .setAction(intentAction)
    .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_CLEAR_TOP or Intent.FLAG_ACTIVITY_SINGLE_TOP)

fun ShortcutAction.pendingIntent(context: Context): PendingIntent = PendingIntent.getActivity(
    context, 0, launchIntent(context), PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE,
)
