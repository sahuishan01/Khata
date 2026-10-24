package com.khata.app.shortcuts

import android.appwidget.AppWidgetManager
import android.appwidget.AppWidgetProvider
import android.content.Context
import android.os.Bundle
import android.widget.RemoteViews
import com.khata.app.R

class ShortcutWidgetProvider : AppWidgetProvider() {
    override fun onUpdate(context: Context, manager: AppWidgetManager, ids: IntArray) {
        ids.forEach { update(context, manager, it) }
    }

    override fun onAppWidgetOptionsChanged(context: Context, manager: AppWidgetManager, id: Int, options: Bundle) {
        update(context, manager, id)
    }

    private fun update(context: Context, manager: AppWidgetManager, id: Int) {
        val views = RemoteViews(context.packageName, R.layout.widget_shortcuts)
        listOf(
            R.id.shortcut_add to ShortcutAction.ADD_TRANSACTION,
            R.id.shortcut_history to ShortcutAction.TRANSACTIONS,
            R.id.shortcut_chat to ShortcutAction.CHAT,
            R.id.shortcut_upload to ShortcutAction.ADD_DATA,
        ).forEach { (viewId, action) -> views.setOnClickPendingIntent(viewId, action.pendingIntent(context)) }
        manager.updateAppWidget(id, views)
    }
}

class SingleShortcutWidgetProvider : AppWidgetProvider() {
    override fun onUpdate(context: Context, manager: AppWidgetManager, ids: IntArray) {
        ids.forEach { update(context, manager, it) }
    }

    override fun onDeleted(context: Context, ids: IntArray) {
        context.getSharedPreferences(PREFERENCES, Context.MODE_PRIVATE).edit().apply {
            ids.forEach { remove(it.toString()) }
        }.apply()
    }

    companion object {
        const val PREFERENCES = "shortcut_widgets"

        fun update(context: Context, manager: AppWidgetManager, id: Int) {
            val stored = context.getSharedPreferences(PREFERENCES, Context.MODE_PRIVATE).getString(id.toString(), null)
            val action = ShortcutAction.entries.firstOrNull { it.name == stored } ?: ShortcutAction.ADD_TRANSACTION
            val views = RemoteViews(context.packageName, R.layout.widget_single_shortcut)
            views.setTextViewText(R.id.shortcut_single, context.getString(action.labelRes))
            views.setTextViewCompoundDrawables(R.id.shortcut_single, 0, action.iconRes, 0, 0)
            views.setOnClickPendingIntent(R.id.shortcut_single, action.pendingIntent(context))
            manager.updateAppWidget(id, views)
        }
    }
}
