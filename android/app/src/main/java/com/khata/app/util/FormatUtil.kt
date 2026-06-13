package com.khata.app.util

import java.text.NumberFormat
import java.time.LocalDate
import java.time.format.DateTimeFormatter
import java.util.Locale

fun formatINR(rupees: Double, sign: Boolean = false): String {
    val abs = kotlin.math.abs(rupees)
    val nf = NumberFormat.getInstance(Locale.forLanguageTag("en-IN")).apply {
        maximumFractionDigits = 2
        minimumFractionDigits = if (abs % 1.0 == 0.0) 0 else 2
    }
    val pre = if (sign) (if (rupees < 0) "−" else "+") else (if (rupees < 0) "−" else "")
    return "$pre₹${nf.format(abs)}"
}

fun formatDate(iso: String): String {
    if (iso.isBlank()) return ""
    return try {
        val date = LocalDate.parse(iso.take(10))
        date.format(DateTimeFormatter.ofPattern("dd MMM yyyy"))
    } catch (_: Exception) {
        iso
    }
}
