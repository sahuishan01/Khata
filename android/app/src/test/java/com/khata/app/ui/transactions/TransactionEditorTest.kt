package com.khata.app.ui.transactions

import org.junit.Assert.*
import org.junit.Test

class TransactionEditorTest {
    @Test fun moneyValidationRejectsInvalidAndOverpreciseAmounts() {
        for (value in listOf("", "NaN", "Infinity", "0", "-1", "1.001", "1e3", "1,000", "1000000000000")) {
            assertNull(value, validEditMoney(value, true))
        }
        assertEquals(123.45, validEditMoney("123.45", true)!!, 0.0)
        assertEquals(-123.45, validEditMoney("-123.45", false)!!, 0.0)
        assertEquals(0.0, validEditMoney("0", false)!!, 0.0)
    }
}
