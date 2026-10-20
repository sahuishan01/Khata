package com.khata.app.sms

import com.khata.app.data.LocalTransaction

/** Live capture and historical rescans use exactly the same date/transfer policy. */
object SmsImport {
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
        )
    }

    /** Repair previously imported SMS rows without replacing their identity or other edits. */
    fun repair(old: LocalTransaction, parsed: ParsedSmsTxn, body: String, timestamp: Long): LocalTransaction {
        if (old.deleted || !old.notes.startsWith("Auto-captured")) return old
        val oldMessageDate = SmsDates.timestampDate(timestamp)
        val resolved = SmsDates.resolve(body, timestamp, old.updatedAt)
        val newTransfer = parsed.isTransfer && !old.isTransfer && old.category != "Transfer"
        val changed = old.copy(
            txnDate = if (old.txnDate.isBlank() || old.txnDate == oldMessageDate) resolved else old.txnDate,
            valueDate = if (old.valueDate.isBlank() || old.valueDate == oldMessageDate) resolved else old.valueDate,
            isTransfer = old.isTransfer || newTransfer,
            category = if (newTransfer) "Transfer" else old.category,
            description = if (newTransfer) "Credit card ${if (parsed.direction == "credit") "payment received" else "bill payment"}" else old.description,
        )
        return if (changed == old) old else changed.copy(dirty = true, pendingOp = if (old.baseRev > 0) "UPDATE" else "CREATE")
    }
}
