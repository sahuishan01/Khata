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
    private fun captureTitle(bank: String) = "Auto-captured via SMS ($bank)"

    /** Exclude only an unchanged auto-import proven to come from this rejected non-transaction SMS. */
    suspend fun excludeNonTransaction(dao: TransactionDao, body: String, sender: String, timestamp: Long, queuedAt: Long): Boolean {
        val parsed = SmsParser.previouslyImportedNonTransaction(body, sender) ?: return false
        // Referenced/repayment records need review rather than automatic cleanup.
        if (parsed.refNo != null || parsed.isTransfer) return false
        val old = dao.getByClientId(SmsParser.clientId(parsed, body, sender, timestamp)) ?: return false
        val source = SmsSource.decode(old.smsSource) ?: return false
        if (source.userEdited == true) return false
        if (source.bodyHash != SmsSource.from(body, timestamp, queuedAt).bodyHash || source.receivedAt != timestamp) return false
        if (old.deleted || (old.dirty && old.baseRev > 0) ||
            old.amount != parsed.amount || old.direction != parsed.direction || old.bank != parsed.bank ||
            !((old.description == captureTitle(parsed.bank) && old.notes == body) ||
                (old.notes == captureTitle(parsed.bank) && old.description in listOf(parsed.payee, body))) ||
            old.category != "Uncategorized" || old.isTransfer ||
            old.bankRef.isNotBlank() || old.balance != null ||
            old.txnDate != source.parsedDate || old.valueDate != source.parsedDate) return false
        return dao.excludeSmsIfUnchanged(old)
    }

    suspend fun queue(dao: TransactionDao, parsed: ParsedSmsTxn, body: String, sender: String, timestamp: Long, queuedAt: Long): SmsSaveResult {
        val (old, row) = prepare(dao, parsed, body, sender, timestamp, queuedAt)
        if (row == old) return SmsSaveResult.UNCHANGED
        if (!dao.saveSmsIfUnchanged(old, row)) return SmsSaveResult.UNCHANGED
        return if (old == null) SmsSaveResult.CREATED else SmsSaveResult.UPDATED
    }

    /** A model worker uploads with its captured owner before adopting the canonical row locally. */
    suspend fun prepare(dao: TransactionDao, parsed: ParsedSmsTxn, body: String, sender: String, timestamp: Long, queuedAt: Long): Pair<LocalTransaction?, LocalTransaction> {
        return prepareWithLookup(parsed,body,sender,timestamp,queuedAt,dao::getByClientId)
    }

    /** Lookup can use an owner-scoped server snapshot instead of a shared device cache. */
    suspend fun prepareWithLookup(parsed: ParsedSmsTxn, body: String, sender: String, timestamp: Long, queuedAt: Long,
        lookup: suspend (String) -> LocalTransaction?): Pair<LocalTransaction?,LocalTransaction> {
        val clientId = SmsParser.clientId(parsed, body, sender, timestamp)
        val legacyParsed = parsed.copy(refNo = SmsParser.legacyReference(body))
        val previousClientId = SmsParser.clientId(legacyParsed, body, sender, timestamp)
        // Legacy identity used the device timezone and message date; keep that lookup stable.
        val legacyDate = SimpleDateFormat("yyyy-MM-dd", Locale.US).format(Date(timestamp))
        val legacyId = "sms_" + UUID.nameUUIDFromBytes(
            (sender + "|" + parsed.amount + "|" + parsed.direction + "|" + legacyDate + "|" + (legacyParsed.refNo ?: body.take(24))).toByteArray()
        ).toString().take(16)
        val repairedId = UUID.nameUUIDFromBytes(("legacy:" + legacyId).toByteArray(Charsets.UTF_8)).toString()
        // Prefer this message's old identity, even if another alert already owns
        // the new reference-based ID. Otherwise its old duplicate is never repaired.
        val old = lookup(legacyId) ?: lookup(repairedId)
            ?: lookup(previousClientId) ?: lookup(clientId)
            ?: run {
                // Learning an issuer alias must not duplicate an older record
                // whose issuer was unknown. Require the exact original receipt.
                val original = SmsSource.from(body, timestamp, queuedAt)
                listOf(parsed, legacyParsed).firstNotNullOfOrNull { previous ->
                    lookup(SmsParser.clientId(previous.copy(bank = "Bank"), body, sender, timestamp))
                        ?.takeIf { row ->
                            val source = SmsSource.decode(row.smsSource)
                            source?.bodyHash == original.bodyHash && source?.receivedAt == timestamp
                        }
                }
            }
        val row = old?.let { repair(it, parsed, body, timestamp) }
            ?: transaction(parsed, body, sender, timestamp, queuedAt)
        return old to row
    }

    fun transaction(parsed: ParsedSmsTxn, body: String, sender: String, timestamp: Long, queuedAt: Long): LocalTransaction {
        val date = SmsDates.resolve(body, timestamp, queuedAt)
        return LocalTransaction(
            clientId = SmsParser.clientId(parsed, body, sender, timestamp),
            description = captureTitle(parsed.bank),
            amount = parsed.amount, direction = parsed.direction,
            category = if (parsed.isTransfer) "Transfer" else "Uncategorized",
            bank = parsed.bank, bankRef = parsed.refNo.orEmpty(), txnDate = date, valueDate = date,
            isTransfer = parsed.isTransfer, notes = body,
            updatedAt = queuedAt, dirty = true, pendingOp = "CREATE",
            smsSource = Gson().toJson(SmsSource.from(body, timestamp, queuedAt, parsed.direction, parsed.bank).copy(
                parsedPayee = parsed.payee, paymentMethod = parsed.paymentMethod, classifierVersion = parsed.classifierVersion)),
        )
    }

    /** Repair previously imported SMS rows without replacing their identity or other edits. */
    fun repair(old: LocalTransaction, parsed: ParsedSmsTxn, body: String, timestamp: Long): LocalTransaction {
        if (old.deleted) return old
        val oldMessageDate = SmsDates.timestampDate(timestamp)
        val previousSource = SmsSource.decode(old.smsSource)
        if (previousSource?.userEdited == true) return old
        val incomingSource = SmsSource.from(body, timestamp, old.updatedAt, parsed.direction, parsed.bank)
            .copy(parsedPayee = parsed.payee)
        val verifiedMessage = previousSource?.bodyHash == incomingSource.bodyHash &&
            previousSource?.receivedAt == timestamp
        val swapped = old.description == captureTitle(parsed.bank) && old.notes == body && verifiedMessage
        if (!old.notes.startsWith("Auto-captured") && !swapped) return old
        val swapLegacy = verifiedMessage && old.notes == captureTitle(parsed.bank) &&
            old.description in listOf(body, parsed.payee, "Credit card payment received", "Credit card bill payment")
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
            // A repair must not turn an unverifiable legacy title into proof
            // that it was generated, allowing the next scan to overwrite it.
            smsSource = Gson().toJson(source.copy(
                bodyHash = source.bodyHash.takeIf { previousSource?.bodyHash != null },
                eventKey = source.eventKey.takeIf { previousSource?.bodyHash != null },
                previousTxnDate = previousSource?.previousTxnDate,
                paymentMethod = parsed.paymentMethod ?: previousSource?.paymentMethod,
                classifierVersion = parsed.classifierVersion ?: previousSource?.classifierVersion,
                previousValueDate = previousSource?.previousValueDate)),
            isTransfer = old.isTransfer || newTransfer,
            category = if (newTransfer) "Transfer" else old.category,
            // Swap only a proven original alert with untouched generated fields.
            description = if (swapLegacy) captureTitle(parsed.bank) else old.description,
            notes = if (swapLegacy) body else old.notes,
        )
        return if (changed == old) old else changed.copy(dirty = true, pendingOp = if (old.baseRev > 0) "UPDATE" else "CREATE")
    }
}
