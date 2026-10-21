package com.khata.app.sms

/** Keep repayment rules aligned with backend migration 0034. Never infer from an amount match. */
object CardTransfers {
    private val excluded = Regex("\\b(REFUND|REVERSAL|REVERSED|CASHBACK|CHARGEBACK|FEE|FEES|CHARGE|CHARGES|INTEREST|INSURANCE|FAILED|DECLINED|UNSUCCESSFUL)\\b|PAYMENT DUE|DUE DATE|WILL BE|SCHEDULED|DEBIT CARD")
    private val card = Regex("\\b(CREDIT CARD|CREDITCARD|CC)\\b")
    private val bankCard = Regex("\\bCARD\\b")
    private val direct = Regex("\\b(CREDIT CARD|CREDITCARD|CC) (BILL )?(PAYMENT|REPAYMENT|PMT)\\b|\\bCCPMT\\b")
    private val towards = Regex("\\b(PAYMENT|PAID|DEBITED|TOWARDS)\\b.*\\b(TOWARDS|FOR|TO)\\b.*\\b(CREDIT CARD|CREDITCARD|CC)\\b|\\bTOWARDS\\b.*\\b(CREDIT CARD|CREDITCARD|CC)\\b")
    private val received = Regex("\\bPAYMENT\\b.*\\b(RECEIVED|CREDITED|ACKNOWLEDGED|SETTLED|SETTLEMENT)\\b|\\b(RECEIVED|CREDITED)\\b.*\\bPAYMENT\\b|\\bTHANK YOU FOR (PAYING|YOUR PAYMENT)\\b")
    private val purchase = Regex("\\b(CREDIT CARD|CREDITCARD|CC) PAYMENT (AT|TO)\\b")

    fun isRepayment(text: String, direction: String): Boolean {
        val normalized = text.uppercase(java.util.Locale.ROOT).replace(Regex("[^A-Z0-9]+"), " ").trim()
        if (excluded.containsMatchIn(normalized) || purchase.containsMatchIn(normalized)) return false
        return direct.containsMatchIn(normalized) || (direction == "credit" && bankCard.containsMatchIn(normalized) && received.containsMatchIn(normalized)) || (card.containsMatchIn(normalized) &&
            ((direction == "debit" && towards.containsMatchIn(normalized)) ||
                (direction == "credit" && received.containsMatchIn(normalized))))
    }
}
