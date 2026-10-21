package com.khata.app.sms

import java.time.Instant
import java.time.LocalDate
import java.time.ZoneId
import java.time.format.DateTimeFormatter
import java.time.format.DateTimeFormatterBuilder
import java.time.format.ResolverStyle
import java.time.temporal.ChronoField
import java.util.Locale

/** Bank dates are Indian calendar dates, independent of the device's current zone. */
object SmsDates {
    val bankZone: ZoneId = ZoneId.of("Asia/Kolkata")
    private val datePattern = Regex(
        "(?i)(?<![A-Za-z0-9/])(?:\\d{4}-\\d{2}-\\d{2}|\\d{1,2}[-/.]\\d{1,2}[-/.]\\d{2,4}|\\d{1,2}[-/ ]?(?:Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec)[a-z]*[-/ ]?\\d{2,4})(?![A-Za-z0-9/])"
    )
    private val formats = listOf("d-M-uuuu", "d-MMM-uuuu", "d-MMMM-uuuu", "dMMMuuuu", "uuuu-MM-dd").map {
        DateTimeFormatterBuilder().parseCaseInsensitive().appendPattern(it).toFormatter(Locale.ENGLISH)
            .withResolverStyle(ResolverStyle.STRICT)
    } + listOf("d-M-", "d-MMM-", "d-MMMM-", "dMMM").map {
        DateTimeFormatterBuilder().parseCaseInsensitive().appendPattern(it)
            .appendValueReduced(ChronoField.YEAR, 2, 2, 2000).toFormatter(Locale.ENGLISH)
            .withResolverStyle(ResolverStyle.STRICT)
    }
    private val unrelatedDate = Regex("(?i)(?:due|pay by|statement|billing|bill period|available bal|avl bal|balance as|valid till|expires)")
    private val transactionDate = Regex("(?i)(?:\\bon|\\bdated|txn date|transaction date|value date)\\s*[:=-]?\\s*$")

    fun inBody(body: String, latest: LocalDate? = null): LocalDate? {
        val candidates = datePattern.findAll(body).mapNotNull { match ->
            // Stop at sentence/clause boundaries, without treating decimal points as dates.
            val prefix = body.substring(0, match.range.first).takeLast(80)
                .substringAfterLast(';').substringAfterLast('\n').substringAfterLast(". ")
            if (unrelatedDate.containsMatchIn(prefix)) return@mapNotNull null
            // A clock (12.02.26), reference or URL is not a transaction date.
            if (Regex("(?i)(?:\\btime|\\bat|\\bref(?:erence)?|\\brrn)\\s*[:#=-]?\\s*$").containsMatchIn(prefix)) return@mapNotNull null
            val normalized = match.value.replace('/', '-').replace('.', '-').replace(' ', '-')
            val date = formats.firstNotNullOfOrNull { format ->
                runCatching { LocalDate.parse(normalized, format) }.getOrNull()?.takeIf { it.year in 2000..2099 }
            }?.takeIf { latest == null || it <= latest } ?: return@mapNotNull null
            date to transactionDate.containsMatchIn(prefix)
        }.toList()
        return candidates.firstOrNull { it.second }?.first ?: candidates.firstOrNull()?.first
    }

    fun timestampDate(timestamp: Long): String = Instant.ofEpochMilli(timestamp).atZone(bankZone).toLocalDate().toString()

    data class Resolution(val date: String, val source: String)

    /** Posted transactions cannot be later than their alert. Never swap ambiguous day/month values. */
    fun resolution(body: String, messageTimestamp: Long?, queuedAt: Long): Resolution {
        val received = messageTimestamp?.takeIf { it > 0 && it <= queuedAt }
        val latest = LocalDate.parse(timestampDate(received ?: queuedAt))
        return inBody(body, latest)?.let { Resolution(it.toString(), "body") }
            ?: Resolution(latest.toString(), if (received != null) "message" else "queue")
    }

    /** Valid body date > original SMS timestamp > persisted queue creation date. */
    fun resolve(body: String, messageTimestamp: Long?, queuedAt: Long): String =
        resolution(body, messageTimestamp, queuedAt).date
}
