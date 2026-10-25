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
import com.khata.app.ui.transactions.validEditMoney

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
        Text(if (jobs.any { it.status=="pending" }) "SMS processing in the background" else "SMS need review")
        jobs.filter { it.status=="review" }.take(20).forEach { job ->
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
        if (jobs.any { it.status=="review" }) TextButton(onClick=onRetry) { Text("Retry undecided SMS") }
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
