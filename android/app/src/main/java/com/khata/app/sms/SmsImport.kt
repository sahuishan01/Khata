package com.khata.app.sms

import com.khata.app.data.LocalTransaction
import com.khata.app.data.TransactionDao
import java.text.SimpleDateFormat
import java.util.Date
import java.util.Locale
import java.util.UUID
import com.google.gson.Gson

enum class SmsSaveResult { CREATED, UPDATED, UNCHANGED }

/** Live capture and historical rescans use exactly the same date/transfer policy. */
object SmsImport {
    suspend fun queue(dao: TransactionDao, parsed: ParsedSmsTxn, body: String, sender: String, timestamp: Long, queuedAt: Long): SmsSaveResult {
        val clientId = SmsParser.clientId(parsed, body, sender, timestamp)
        val legacyParsed = parsed.copy(refNo = SmsParser.legacyReference(body))
        val previousClientId = SmsParser.clientId(legacyParsed, body, sender, timestamp)
        // Legacy identity used the device timezone and message date; keep that lookup stable.
        val legacyDate = SimpleDateFormat("yyyy-MM-dd", Locale.US).format(Date(timestamp))
        val legacyId = "sms_" + UUID.nameUUIDFromBytes(
            (sender + "|" + parsed.amount + "|" + parsed.direction + "|" + legacyDate + "|" + (legacyParsed.refNo ?: body.take(24))).toByteArray()
        ).toString().take(16)
        val repairedId = UUID.nameUUIDFromBytes(("legacy:" + legacyId).toByteArray(Charsets.UTF_8)).toString()
        val old = dao.getByClientId(clientId) ?: dao.getByClientId(previousClientId)
            ?: dao.getByClientId(legacyId) ?: dao.getByClientId(repairedId)
        val row = old?.let {
            repair(if (it.rev > 0 || it.baseRev > 0) it else it.copy(clientId = clientId), parsed, body, timestamp)
        } ?: transaction(parsed, body, sender, timestamp, queuedAt)
        if (row == old) return SmsSaveResult.UNCHANGED
        dao.upsert(row)
        return if (old == null) SmsSaveResult.CREATED else SmsSaveResult.UPDATED
    }

    fun transaction(parsed: ParsedSmsTxn, body: String, sender: String, timestamp: Long, queuedAt: Long): LocalTransaction {
        val date = SmsDates.resolve(body, timestamp, queuedAt)
        return LocalTransaction(
            clientId = SmsParser.clientId(parsed, body, sender, timestamp),
            description = if (parsed.isTransfer) "Credit card ${if (parsed.direction == "credit") "payment received" else "bill payment"}" else parsed.payee,
            amount = parsed.amount, direction = parsed.direction,
            category = if (parsed.isTransfer) "Transfer" else "Uncategorized",
            bank = parsed.bank, bankRef = parsed.refNo.orEmpty(), txnDate = date, valueDate = date,
            isTransfer = parsed.isTransfer, notes = "Auto-captured via SMS (${parsed.bank})",
            updatedAt = queuedAt, dirty = true, pendingOp = "CREATE",
            smsSource = Gson().toJson(SmsSource.from(body, timestamp, queuedAt)),
        )
    }

    /** Repair previously imported SMS rows without replacing their identity or other edits. */
    fun repair(old: LocalTransaction, parsed: ParsedSmsTxn, body: String, timestamp: Long): LocalTransaction {
        if (old.deleted || !old.notes.startsWith("Auto-captured")) return old
        val oldMessageDate = SmsDates.timestampDate(timestamp)
        val previousSource = old.smsSource?.let { runCatching { Gson().fromJson(it, SmsSource::class.java) }.getOrNull() }
        val incomingSource = SmsSource.from(body, timestamp, old.updatedAt)
        // Another alert for a referenced payment may omit the transaction date.
        // Its receipt day must not replace a known valid date from the earlier alert.
        val source = if (previousSource?.dateSource == "body" && incomingSource.dateSource != "body" &&
            previousSource.parsedDate <= oldMessageDate) incomingSource.copy(
                dateSource = "body", parsedDate = previousSource.parsedDate) else incomingSource
        val resolved = source.parsedDate
        fun repairDate(date: String): String = if (date.isBlank() || date == oldMessageDate ||
            date == previousSource?.parsedDate || date > oldMessageDate) resolved else date
        val newTransfer = parsed.isTransfer && !old.isTransfer && old.category != "Transfer"
        val changed = old.copy(
            txnDate = repairDate(old.txnDate),
            valueDate = repairDate(old.valueDate),
            bankRef = old.bankRef.ifBlank { parsed.refNo.orEmpty() },
            smsSource = Gson().toJson(source.copy(previousTxnDate = previousSource?.previousTxnDate,
                previousValueDate = previousSource?.previousValueDate)),
            isTransfer = old.isTransfer || newTransfer,
            category = if (newTransfer) "Transfer" else old.category,
            description = if (newTransfer) "Credit card ${if (parsed.direction == "credit") "payment received" else "bill payment"}" else old.description,
        )
        return if (changed == old) old else changed.copy(dirty = true, pendingOp = if (old.baseRev > 0) "UPDATE" else "CREATE")
    }
}
