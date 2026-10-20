package com.khata.app.data

import com.khata.app.api.KhataApi
import com.khata.app.api.SyncPushOp
import com.khata.app.api.toLocal
import java.util.UUID
import com.khata.app.sms.SmsDates

/** Upload persisted work without discarding it when a request fails. */
internal class SyncOutbox(private val dao: TransactionDao, private val api: KhataApi) {
    suspend fun push(): List<SyncConflict> {
        val outbox = dao.getOutbox().map { txn ->
            val clientId = runCatching { UUID.fromString(txn.clientId).toString() }.getOrElse {
                UUID.nameUUIDFromBytes(("legacy:" + txn.clientId).toByteArray(Charsets.UTF_8)).toString()
            }
            if (clientId == txn.clientId) txn
            else txn.copy(clientId = clientId).also { dao.upsert(it) }
        }
        val conflicts = mutableListOf<SyncConflict>()
        for (batch in outbox.chunked(200)) {
            val result = api.syncPush(batch.map { txn ->
                // Older live-SMS entries have valueDate but an empty txnDate.
                // Dates are optional in the API; an empty string is invalid.
                val txnDate = txn.txnDate.trim().takeIf { it.isNotEmpty() }
                val valueDate = txn.valueDate.trim().takeIf { it.isNotEmpty() }
                val queueDate = SmsDates.timestampDate(txn.updatedAt)
                SyncPushOp(
                    clientId = txn.clientId, baseRev = txn.baseRev,
                    amount = txn.amount, direction = txn.direction,
                    description = txn.description, category = txn.category,
                    bank = txn.bank, bankRef = txn.bankRef.ifBlank { null },
                    txnDate = txnDate ?: valueDate ?: queueDate, valueDate = valueDate ?: txnDate ?: queueDate,
                    notes = txn.notes, isTransfer = txn.isTransfer, deleted = txn.deleted,
                )
            })
            val sent = batch.associateBy { it.clientId }
            for (accepted in result.accepted) {
                val original = sent[accepted.clientId] ?: continue
                val local = dao.getByClientId(accepted.clientId) ?: continue
                // A concurrent local edit still needs to be sent in the next sync.
                dao.upsert(local.copy(
                    rev = accepted.serverRev, baseRev = accepted.serverRev,
                    dirty = local != original,
                    pendingOp = if (local == original) null else local.pendingOp,
                ))
            }
            conflicts += result.conflicts.map { c ->
                SyncConflict(c.clientId, c.serverRev, c.serverTxn.toLocal(),
                    dao.getByClientId(c.clientId) ?: LocalTransaction())
            }
            val acknowledged = (result.accepted.map { it.clientId } + result.conflicts.map { it.clientId }).toSet()
            check(batch.all { it.clientId in acknowledged }) {
                "The server did not confirm every upload. Unconfirmed transactions remain queued."
            }
        }
        return conflicts
    }
}
