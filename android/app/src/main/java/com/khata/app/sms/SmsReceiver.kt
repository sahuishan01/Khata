package com.khata.app.sms

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.provider.Telephony
import android.util.Log
import com.khata.app.data.KhataDatabase
import com.khata.app.data.LocalTransaction
import com.khata.app.data.SyncEngine
import dagger.hilt.android.AndroidEntryPoint
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale
import java.util.UUID
import javax.inject.Inject

@AndroidEntryPoint
class SmsReceiver : BroadcastReceiver() {

    @Inject
    lateinit var db: KhataDatabase

    @Inject
    lateinit var syncEngine: SyncEngine

    override fun onReceive(context: Context?, intent: Intent?) {
        if (intent?.action != Telephony.Sms.Intents.SMS_RECEIVED_ACTION) return

        val messages = Telephony.Sms.Intents.getMessagesFromIntent(intent) ?: return

        for (sms in messages) {
            val body = sms.messageBody ?: continue
            val sender = sms.originatingAddress ?: ""

            val parsed = SmsParser.parse(body, sender) ?: continue
            Log.d("SmsReceiver", "Real-time bank txn detected: $parsed")

            val today = SimpleDateFormat("yyyy-MM-dd", Locale.US).format(Date())
            val clientId = "sms_" + UUID.randomUUID().toString().take(12)

            val txn = LocalTransaction(
                clientId = clientId,
                description = parsed.payee,
                amount = parsed.amount,
                direction = parsed.direction,
                category = "Uncategorized",
                bank = parsed.bank,
                valueDate = today,
                notes = "Auto-captured via SMS (${parsed.bank})",
                dirty = true,
                pendingOp = "CREATE"
            )

            // Save locally and trigger background sync to server
            val pendingResult = goAsync()
            CoroutineScope(Dispatchers.IO).launch {
                try {
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
