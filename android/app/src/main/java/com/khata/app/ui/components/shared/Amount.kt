package com.khata.app.ui.components.shared

import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.unit.sp
import com.khata.app.ui.theme.FigureStyle
import com.khata.app.ui.theme.KhataColors
import com.khata.app.util.formatINR

enum class AmountTone { Income, Expense, Neutral, Muted }

/**
 * Money figure (DESIGN.md §3). Signed amounts are colored by sign; unsigned
 * totals stay neutral unless [tone] says otherwise — "Total spent" is not income.
 */
@Composable
fun KhataAmount(
    rupees: Double,
    sign: Boolean = false,
    size: TextUnit = 14.sp,
    modifier: Modifier = Modifier,
    tone: AmountTone? = null,
) {
    val color: Color = when (tone) {
        AmountTone.Income -> KhataColors.income
        AmountTone.Expense -> KhataColors.expense
        AmountTone.Neutral -> KhataColors.text
        AmountTone.Muted -> KhataColors.text2
        null -> when {
            !sign -> KhataColors.text
            rupees < 0 -> KhataColors.expense
            rupees > 0 -> KhataColors.income
            else -> KhataColors.text
        }
    }
    Text(
        text = formatINR(rupees, sign = sign),
        color = color,
        style = FigureStyle.copy(fontSize = size),
        textAlign = TextAlign.End,
        maxLines = 1,
        softWrap = false,
        modifier = modifier,
    )
}
