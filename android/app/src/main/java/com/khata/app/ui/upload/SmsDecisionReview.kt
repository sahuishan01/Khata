package com.khata.app.ui.upload

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.dp
import com.khata.app.sms.SmsDecisionJob
import com.khata.app.sms.SmsParser
import com.khata.app.sms.SmsDecisionSummary
import com.khata.app.sms.hasMessages
import com.khata.app.sms.isComplete
import com.khata.app.sms.totalCount
import com.khata.app.sms.finishedCount
import com.khata.app.sms.progressFraction
import com.khata.app.sms.progressPercent
import com.khata.app.ui.transactions.validEditMoney

@Composable
fun SmsClassificationStatus(summary: SmsDecisionSummary, onRetry: () -> Unit, scanning: Boolean = false) {
    if (!summary.hasMessages() && !scanning) return
    Column(Modifier.fillMaxWidth().padding(vertical=8.dp),verticalArrangement=Arrangement.spacedBy(4.dp)) {
        Text(when {
            scanning -> "SMS inbox scan in progress"
            summary.pending > 0 -> "SMS upload in progress"
            summary.review > 0 -> "SMS upload needs review"
            else -> "SMS import complete"
        },style=MaterialTheme.typography.titleSmall)
        Text(
            "${summary.finishedCount()} of ${summary.totalCount()} queued SMS finished" +
                if (scanning) "" else " (${summary.progressPercent()}%)",
            style=MaterialTheme.typography.bodySmall.copy(fontFeatureSettings="tnum"),
        )
        if (scanning) {
            LinearProgressIndicator(Modifier.fillMaxWidth(),color=MaterialTheme.colorScheme.onSurfaceVariant)
            Text("Reading inbox; the queued total is still growing.",style=MaterialTheme.typography.bodySmall)
        } else {
            LinearProgressIndicator(
                progress={summary.progressFraction()},modifier=Modifier.fillMaxWidth(),
                color=MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
        Text("${summary.pending} waiting · ${summary.review} need review · ${summary.accepted} SMS handled",style=MaterialTheme.typography.bodySmall)
        Text("${summary.filtered} skipped on this device · ${summary.ignored} ignored",style=MaterialTheme.typography.bodySmall)
        if (!summary.isComplete()) Text("Handled SMS may reuse saved transactions. Review messages under Add Data → Upload.",style=MaterialTheme.typography.bodySmall)
        if (summary.pending > 0) {
            Text("SMS are parsed on this device. Uploads wait for a connection.",style=MaterialTheme.typography.bodySmall)
            TextButton(onClick=onRetry) { Text("Retry SMS uploads") }
        }
    }
}

@Composable
fun SmsDecisionReview(
    jobs: List<SmsDecisionJob>, onRetry: () -> Unit, onIgnore: (String) -> Unit,
    onResolve: (String,Double,String,(String?)->Unit) -> Unit,
) {
    var selected by remember { mutableStateOf<SmsDecisionJob?>(null) }
    var amount by remember { mutableStateOf("") }
    var direction by remember { mutableStateOf("") }
    var saving by remember { mutableStateOf(false) }
    var error by remember { mutableStateOf<String?>(null) }
    if (jobs.isNotEmpty()) {
        jobs.firstOrNull { it.status=="pending" && it.reason.isNotBlank() }?.let { Text(it.reason,style=MaterialTheme.typography.bodySmall) }
        jobs.filter { it.status in listOf("review","conflict") }.take(20).forEach { job ->
            Card(Modifier.fillMaxWidth().padding(vertical=4.dp)) {
                Column(Modifier.padding(12.dp)) {
                    Text(job.body.orEmpty(),maxLines=3)
                    Text(job.reason,style=MaterialTheme.typography.bodySmall)
                    Row {
                        TextButton(onClick={
                            selected=job; error=null; direction=""
                            amount=SmsParser.parse(job.body.orEmpty(),job.sender)?.amount?.let { java.math.BigDecimal.valueOf(it).toPlainString() }.orEmpty()
                        }) { Text("Review") }
                        TextButton(onClick={onIgnore(job.id)}) { Text("Ignore") }
                    }
                }
            }
        }
        if (jobs.any { it.status in listOf("review","conflict") }) TextButton(onClick=onRetry) { Text("Retry SMS needing review") }
    }
    selected?.let { job ->
        AlertDialog(onDismissRequest={if(!saving) selected=null},title={Text("Review SMS")},
            text={Column(Modifier.heightIn(max=450.dp).verticalScroll(rememberScrollState()),verticalArrangement=Arrangement.spacedBy(12.dp)) {
                Text(job.body.orEmpty())
                OutlinedTextField(amount,{amount=it},label={Text("Amount (INR)")},singleLine=true,
                    keyboardOptions=KeyboardOptions(keyboardType=KeyboardType.Decimal),enabled=!saving)
                Row(horizontalArrangement=Arrangement.spacedBy(8.dp)) {
                    FilterChip(direction=="debit",{direction="debit"},label={Text("Expense")},enabled=!saving)
                    FilterChip(direction=="credit",{direction="credit"},label={Text("Income")},enabled=!saving)
                }
                Text("Confirm the amount and direction before adding this transaction. The original SMS will stay in Notes.")
                error?.let { Text(it,color=MaterialTheme.colorScheme.error) }
            }},
            confirmButton={TextButton(enabled=!saving && validEditMoney(amount,true)!=null && direction in listOf("debit","credit"),onClick={
                saving=true; onResolve(job.id,validEditMoney(amount,true)!!,direction) { result ->
                    saving=false; error=result; if(result==null) selected=null
                }
            }) { Text(if(saving) "Saving…" else "Save") }},
            dismissButton={TextButton(onClick={selected=null},enabled=!saving) { Text("Cancel") }})
    }
}
