package com.khata.app.sms

import java.util.Locale
import java.util.UUID
import java.util.regex.Pattern

data class ParsedSmsTxn(
    val amount: Double,
    val direction: String, // "debit" or "credit"
    val bank: String,
    val payee: String,
    val refNo: String? = null,
    val isTransfer: Boolean = false,
)

object SmsParser {
    /** Shared by live capture and inbox backfill; accepted by the UUID API. */
    fun clientId(parsed: ParsedSmsTxn, body: String, sender: String, timestamp: Long): String {
        val identity = parsed.refNo?.let {
            "${parsed.bank.uppercase(Locale.ROOT)}|${parsed.direction}|${parsed.amount}|${it.uppercase(Locale.ROOT)}"
        } ?: "${sender.uppercase(Locale.ROOT)}|$timestamp|${body.trim()}"
        return UUID.nameUUIDFromBytes(identity.toByteArray(Charsets.UTF_8)).toString()
    }

    // Posting grammar is shared by issuers. Abbreviations need an account
    // reference so a doctor, credit limit or balance cannot become a posting.
    private const val CURRENCY = "(?:rs\\.?|inr|₹)"
    private const val AMOUNT = "([0-9,]+(?:\\.[0-9]{1,2})?)"
    private const val ACCOUNT_REFERENCE = "\\b(?:account|acct|acc|a/c|ac)\\s*" +
        "(?:(?:no\\.?|number|ending(?:\\s+(?:in|with))?)\\s*)?[:#.-]?\\s*" +
        "[x*•#0-9][x*•#0-9 -]{0,38}[0-9]{2,}"

    private fun accountPostings(marker: String): List<Pattern> {
        val posting = "\\b${Pattern.quote(marker)}\\b\\.?"
        val money = "$CURRENCY\\s*$AMOUNT"
        // Android rejects UNICODE_CHARACTER_CLASS even though JVM tests accept it.
        return listOf(
            "$ACCOUNT_REFERENCE\\s*(?:(?:is|was|has\\s+been)\\s+)?$posting\\s*$money",
            "$ACCOUNT_REFERENCE\\s*$money\\s*$posting",
        ).map { Pattern.compile(it, Pattern.CASE_INSENSITIVE) }
    }

    private val DEBIT_PATTERNS = listOf(
        Pattern.compile("(?i)(?:debited|sent|paid|spent|txn of|withdrawn|vpa)\\s*(?:by|for|of|with)?\\s*(?:rs\\.?|inr|₹)\\s*([0-9,]+(?:\\.[0-9]{1,2})?)"),
        Pattern.compile("(?i)(?:rs\\.?|inr|₹)\\s*([0-9,]+(?:\\.[0-9]{1,2})?)\\s*(?:has\\s+been\\s+|is\\s+)?(?:debited|spent|paid|withdrawn|transferred)"),
    ) + accountPostings("dr")

    private val CREDIT_PATTERNS = listOf(
        Pattern.compile("(?i)payment\\s+(?:of\\s+)?(?:rs\\.?|inr|₹)\\s*([0-9,]+(?:\\.[0-9]{1,2})?).{0,80}?(?:received|credited|acknowledged)"),
        Pattern.compile("(?i)(?:credited|received|deposited|added)\\s*(?:by|for|of|with)?\\s*(?:rs\\.?|inr|₹)\\s*([0-9,]+(?:\\.[0-9]{1,2})?)"),
        Pattern.compile("(?i)(?:rs\\.?|inr|₹)\\s*([0-9,]+(?:\\.[0-9]{1,2})?)\\s*(?:has\\s+been\\s+|is\\s+)?(?:credited|received|deposited)"),
    ) + accountPostings("cr")

    // "Get Rs ... credited" describes an offer, not money that has arrived.
    // Check the clause governing each amount, rather than banning words such as
    // "loan" or "apply" anywhere in a genuine alert's promotional footer.
    private val OFFER_PREFIX = Pattern.compile(
        "(?i)(?:^|[.!?;:,])\\s*(?:(?:dear\\s+customer|congratulations)[,!:\\s]+)?" +
            "(?:get|avail|borrow|apply\\s+for|you\\s+(?:can|could|may)|" +
            "(?:you\\s+are\\s+)?eligible\\s+for)\\b[^.!?;]{0,100}$"
    )

    private val CLAUSE_BOUNDARY = Pattern.compile("[!?;]\\s*|\\.\\s+|[\\r\\n]+")
    private val CURRENCY_ABBREVIATION = Pattern.compile("(?i)\\b(?:rs|inr)$")
    private val REQUEST_NOTICE = Pattern.compile(
        "(?i)\\b(?:has|have)\\s+requested\\b|" +
            "\\b(?:upi|collect|payment|mandate)\\s+request\\b|" +
            "\\brequest\\s+(?:(?:has\\s+been|was|is)\\s+)?received\\b|" +
            "\\brequest\\s+(?:for|of)\\b"
    )
    private val UNCOMPLETED_NOTICE = Pattern.compile(
        "(?i)\\b(?:pending|awaiting)\\s+(?:(?:to|your|for)\\s+)?(?:approv(?:e|al)|authori[sz](?:e|ation))\\b|" +
            "\\b(?:approval|authori[sz]ation)\\s+(?:is\\s+)?pending\\b|" +
            "\\b(?:will|would|shall)\\s+be\\s+(?:debited|credited|paid|sent|withdrawn|transferred)\\b"
    )
    private val CONFIRMED_POSTING = Pattern.compile(
        "(?i)\\b(?:has|have)\\s+been\\s+(?:debited|credited|spent|paid|withdrawn|transferred)\\b"
    )
    private val APPLICATION_NOTICE = Pattern.compile(
        "(?i)\\b(?:app(?:lication)?\\s+(?:no|number|id)\\b|" +
            "(?:ipo|loan)\\s+application\\b|application\\s+(?:received|acknowledged|accepted)\\b|" +
            "application\\s+(?:for|of)\\s*(?:rs\\.?|inr|₹)|bid\\s+(?:no|number|id)\\b)"
    )
    private val FUND_BLOCK_NOTICE = Pattern.compile(
        "(?i)\\b(?:blocking\\s+of\\s+funds|" +
            "(?:funds?|amount)\\s+(?:(?:has|have)\\s+been\\s+|will\\s+be\\s+|(?:is|are)\\s+)?blocked|" +
            "blocked\\s+(?:funds?|amount))\\b"
    )
    private val RECEIPT = Pattern.compile("(?i)\\b(?:received|acknowledged)\\b")
    private val MONEY_RECEIPT_EVIDENCE = Pattern.compile(
        "(?i)\\b(?:debited|credited|deposited|spent|paid|withdrawn|transferred|" +
            "refund(?:ed)?|cashback|salary|dividend|interest)\\b|" +
            "\\b(?:a/?c|account)\\s*(?:(?:no\\.?|number|ending(?:\\s+in)?)\\s*)?[:#*x\\d\\s-]*\\d{3,}"
    )

    /** "Request received" is not money received; keep other completed alert clauses. */
    private fun nonPostingClauses(body: String): List<IntRange> {
        val excluded = mutableListOf<IntRange>()
        // An application receipt can put its collect/block instruction in the
        // next sentence. In that context, "received" alone is not a posting.
        // Require money evidence in the receipt's own clause so a separate real
        // credit cannot legitimize the application amount (or vice versa).
        val receiptNeedsEvidence = REQUEST_NOTICE.matcher(body).find() ||
            APPLICATION_NOTICE.matcher(body).find() || FUND_BLOCK_NOTICE.matcher(body).find()
        var start = 0
        fun consider(end: Int) {
            val clause = body.substring(start, end)
            if (UNCOMPLETED_NOTICE.matcher(clause).find() ||
                FUND_BLOCK_NOTICE.matcher(clause).find() ||
                (REQUEST_NOTICE.matcher(clause).find() && !CONFIRMED_POSTING.matcher(clause).find()) ||
                (receiptNeedsEvidence && RECEIPT.matcher(clause).find() &&
                    !MONEY_RECEIPT_EVIDENCE.matcher(clause).find())) {
                excluded += start until end
            }
            start = end
        }
        val boundary = CLAUSE_BOUNDARY.matcher(body)
        while (boundary.find()) {
            // The dot in "Rs. 129600" must stay with the request and its amount.
            if (body[boundary.start()] == '.' &&
                CURRENCY_ABBREVIATION.matcher(body.substring(0, boundary.start())).find()) continue
            consider(boundary.end())
        }
        consider(body.length)
        return excluded
    }

    // An issuer catalog identifies bank names/sender aliases; it does not gate
    // transaction recognition or select bank-specific message templates.
    private val BANK_ALIASES = linkedMapOf(
        "HDFC" to listOf("hdfc"), "ICICI" to listOf("icici"), "SBI" to listOf("sbi"),
        "AXIS" to listOf("axis"), "KOTAK" to listOf("kotak"), "PNB" to listOf("pnb"),
        "BOB" to listOf("bob"), "CANARA" to listOf("canara", "canbnk"),
        "PAYTM" to listOf("paytm"), "IDFC" to listOf("idfc"), "YES" to listOf("yesbnk"),
        "INDUSIND" to listOf("indusind", "indusb"),
    )
    private val BANK_PATTERN = Pattern.compile("(?i)\\b(${BANK_ALIASES.keys.joinToString("|") { Pattern.quote(it) }})(?:\\s*BANK)?\\b")
    // \b matters: without it the "to" inside ordinary prose ("...wish to inform
    // you that Rs 514...") matches and the payee becomes "inform you that Rs".
    private val PAYEE_PATTERN = Pattern.compile("(?i)\\b(?:to|at|info|vpa)\\s+([A-Za-z0-9\\s\\.\\@\\-\\_]+?)(?:\\s+on|\\s+ref|\\s+avail|\\s+bal|[.,;:!?]|$)")

    /** First words that mean the capture is English prose, not a merchant. */
    private val PAYEE_STOP_WORDS = setOf(
        "inform", "you", "your", "yours", "help", "know", "the", "a", "an", "be",
        "is", "are", "was", "has", "have", "had", "this", "that", "these", "those",
        "we", "our", "us", "it", "its", "avail", "get", "view", "check", "make",
        "see", "use", "enjoy", "grow", "earn", "save", "claim", "apply", "opt",
        "continue", "confirm", "verify", "update", "download", "click", "read",
        "learn", "discover", "explore", "ensure", "keep", "stay", "find", "start",
        "date", "and", "or", "for", "with", "from", "all", "any", "more", "new",
    )

    private fun isProse(candidate: String): Boolean {
        if (candidate.contains("@")) return false // a VPA is always a real payee
        val first = candidate.trim().substringBefore(' ')
            .trim { !it.isLetterOrDigit() }
            .lowercase(Locale.ROOT)
        return first in PAYEE_STOP_WORDS
    }
    private val REF_PATTERN = Pattern.compile("(?i)\\b(?:upi\\s*ref(?:erence)?|ref(?:erence)?|rrn|txn\\s*(?:id|no|number)|transaction\\s*(?:id|no|number))\\s*\\.?\\s*(?:(?:no|number|id)\\.?\\s*)?[:#-]?\\s*([A-Za-z0-9][A-Za-z0-9/-]{5,})")
    private val EXTRA_REFERENCE = Regex("(?i)\\b(?:utr(?:\\s*(?:no|number))?\\.?\\s*[:#-]?\\s*([A-Za-z0-9]{10,35})|upi\\s*[/:-]\\s*(?:p2[ap]/)?([0-9]{12})(?![A-Za-z0-9]))")

    // Keep old identities discoverable when a new parser learns a previously missed reference.
    fun legacyReference(body: String): String? {
        val matcher = REF_PATTERN.matcher(body.replace("\n", " "))
        return if (matcher.find()) matcher.group(1)?.takeIf { value -> value.any { it.isDigit() } } else null
    }

    fun parse(body: String, sender: String = ""): ParsedSmsTxn? = parseCandidate(body, sender, rejectNonTransactions = true)

    /** Used only to identify a proven, untouched false import during an explicit inbox rescan. */
    internal fun previouslyImportedNonTransaction(body: String, sender: String): ParsedSmsTxn? {
        val previous = parseCandidate(body, sender, rejectNonTransactions = false) ?: return null
        return previous.takeIf { parse(body, sender) == null }
    }

    private fun parseCandidate(body: String, sender: String, rejectNonTransactions: Boolean): ParsedSmsTxn? {
        val original = body.trim()
        val cleanBody = original.replace("\n", " ")
        val excluded = if (rejectNonTransactions) nonPostingClauses(original) else emptyList()
        val lowerSender = sender.lowercase(Locale.ROOT)

        // Ignore non-transactional OTP / promo SMSes
        if (cleanBody.lowercase(Locale.ROOT).contains("otp") || cleanBody.lowercase(Locale.ROOT).contains("login code")) {
            return null
        }

        fun amountFrom(patterns: List<Pattern>): Double? {
            for (pattern in patterns) {
                val matcher = pattern.matcher(cleanBody)
                while (matcher.find()) {
                    if (rejectNonTransactions && OFFER_PREFIX.matcher(cleanBody.substring(0, matcher.start())).find()) continue
                    if (excluded.any { matcher.start() <= it.last && matcher.end() > it.first }) continue
                    val amount = matcher.group(1)?.replace(",", "")?.toDoubleOrNull()
                    if (amount != null && amount.isFinite() && amount > 0) return amount
                }
            }
            return null
        }
        val debit = amountFrom(DEBIT_PATTERNS)
        val amount = debit ?: amountFrom(CREDIT_PATTERNS) ?: return null
        val direction = if (debit != null) "debit" else "credit"

        // Detect Bank
        val bankMatcher = BANK_PATTERN.matcher(cleanBody.uppercase(Locale.ROOT))
        val bank = if (bankMatcher.find()) bankMatcher.group(1) ?: "Bank"
            else BANK_ALIASES.entries.firstOrNull { (_, aliases) -> aliases.any(lowerSender::contains) }?.key ?: "Bank"

        // Detect Payee
        var payee = "Transaction"
        val payeeMatcher = PAYEE_PATTERN.matcher(cleanBody)
        if (payeeMatcher.find()) {
            val rawPayee = payeeMatcher.group(1)?.trim() ?: ""
            if (rawPayee.length > 2 &&
                !rawPayee.lowercase(Locale.ROOT).startsWith("ac") &&
                !rawPayee.lowercase(Locale.ROOT).startsWith("a/c") &&
                !isProse(rawPayee)
            ) {
                payee = rawPayee.take(40)
            }
        }

        // Detect Ref Number
        val refNo = legacyReference(cleanBody) ?: EXTRA_REFERENCE.find(cleanBody)?.let { match ->
            match.groupValues.drop(1).firstOrNull { it.isNotEmpty() && it.any(Char::isDigit) }
        }

        return ParsedSmsTxn(
            amount = amount,
            direction = direction,
            bank = bank,
            payee = payee,
            refNo = refNo,
            isTransfer = CardTransfers.isRepayment(cleanBody, direction),
        )
    }
}
