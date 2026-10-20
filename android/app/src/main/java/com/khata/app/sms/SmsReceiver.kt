package com.khata.app.sms

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.provider.Telephony
import android.util.Log
import com.khata.app.data.KhataDatabase
import com.khata.app.data.SyncEngine
import com.khata.app.api.TokenManager
import dagger.hilt.android.AndroidEntryPoint
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import javax.inject.Inject

@AndroidEntryPoint
class SmsReceiver : BroadcastReceiver() {

    @Inject
    lateinit var db: KhataDatabase

    @Inject
    lateinit var syncEngine: SyncEngine

    @Inject
    lateinit var tokenManager: TokenManager

    override fun onReceive(context: Context?, intent: Intent?) {
        if (intent?.action != Telephony.Sms.Intents.SMS_RECEIVED_ACTION) return
        if (tokenManager.getTokenSync().isNullOrBlank()) return

        val messages = Telephony.Sms.Intents.getMessagesFromIntent(intent) ?: return

        for (parts in messages.groupBy { it.originatingAddress to it.timestampMillis }.values) {
            val sms = parts.first()
            val body = parts.joinToString("") { it.messageBody.orEmpty() }
            val sender = sms.originatingAddress ?: ""

            val parsed = SmsParser.parse(body, sender) ?: continue
            val txn = SmsImport.transaction(parsed, body, sender, sms.timestampMillis, System.currentTimeMillis())

            // Save locally and trigger background sync to server
            val pendingResult = goAsync()
            CoroutineScope(Dispatchers.IO).launch {
                try {
                    if (db.transactionDao().getByClientId(txn.clientId) != null) return@launch
                    db.transactionDao().upsert(txn)
                    syncEngine.sync()
                } catch (e: Exception) {
                    Log.e("SmsReceiver", "Failed to save SMS txn", e)
                } finally {
                    pendingResult.finish()
                }
            }
        }
    }
}
