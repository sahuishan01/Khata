package com.khata.app.sms

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.provider.Telephony
import android.util.Log
import com.khata.app.api.TokenManager
import dagger.hilt.android.AndroidEntryPoint
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import javax.inject.Inject

@AndroidEntryPoint
class SmsReceiver : BroadcastReceiver() {

    @Inject
    lateinit var tokenManager: TokenManager

    @Inject
    lateinit var decisions: SmsDecisionPipeline

    override fun onReceive(context: Context?, intent: Intent?) {
        if (intent?.action != Telephony.Sms.Intents.SMS_RECEIVED_ACTION) return
        if (tokenManager.getTokenSync().isNullOrBlank()) return

        val messages = Telephony.Sms.Intents.getMessagesFromIntent(intent) ?: return

        for (parts in messages.groupBy { it.originatingAddress to it.timestampMillis }.values) {
            val sms = parts.first()
            val body = parts.joinToString("") { it.messageBody.orEmpty() }
            if (SmsPrivacy.containsOtp(body)) continue
            val sender = sms.originatingAddress ?: ""

            // Persist non-OTP work quickly; model/network work belongs to WorkManager.
            val pendingResult = goAsync()
            CoroutineScope(Dispatchers.IO).launch {
                try {
                    decisions.enqueue(body, sender, sms.timestampMillis)
                } catch (_: Exception) {
                    Log.e("SmsReceiver", "Could not queue SMS for background processing")
                } finally {
                    pendingResult.finish()
                }
            }
        }
    }
}
