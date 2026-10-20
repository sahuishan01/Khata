package com.khata.app.data

import java.io.IOException
import retrofit2.HttpException

internal fun Exception.syncErrorMessage(): String = when {
    this is HttpException && code() == 401 -> "Your session expired. Sign in again to upload queued transactions."
    this is HttpException && code() == 403 -> "The selected server denied access. Check the server address and sign in again."
    this is HttpException && code() == 422 -> {
        // Only expose a schema field and queue position, never echoed SMS data.
        val detail = runCatching { response()?.errorBody()?.string().orEmpty() }.getOrDefault("")
        val match = Regex("\\[(\\d+)]\\.(txn_date|value_date|client_id|base_rev|amount|direction|description|category|bank_ref|bank|notes|is_transfer|deleted):").find(detail)
        val fields = mapOf("txn_date" to "transaction date", "value_date" to "value date", "client_id" to "transaction identifier", "base_rev" to "revision")
        if (match != null) {
            val position = match.groupValues[1].toLongOrNull()?.plus(1)?.toString() ?: "unknown"
            val field = fields[match.groupValues[2]] ?: match.groupValues[2].replace('_', ' ')
            "The server rejected the $field in queued entry $position (HTTP 422). Saved entries remain queued."
        } else "The server rejected queued transaction data (HTTP 422). Saved entries remain queued."
    }
    this is IOException -> "Cannot reach the selected server. Check your connection and retry."
    else -> message ?: "Sync failed"
}
