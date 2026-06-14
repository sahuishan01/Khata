package com.khata.app.ui.components.shared

import androidx.compose.material3.LocalTextStyle
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.unit.sp
import com.khata.app.ui.theme.KhataColors
import com.khata.app.util.formatINR

@Composable
fun KhataAmount(
    rupees: Double,
    sign: Boolean = false,
    size: TextUnit = 11.5.sp,
    modifier: Modifier = Modifier,
) {
    val color = when {
        rupees < 0 -> KhataColors.expense
        rupees > 0 -> KhataColors.income
        else -> KhataColors.text
    }
    Text(
        text = formatINR(rupees, sign = sign),
        color = color,
        fontSize = size,
        fontWeight = FontWeight.Bold,
        fontFamily = FontFamily.Monospace,
        style = LocalTextStyle.current.copy(fontFeatureSettings = "tnum"),
        textAlign = TextAlign.End,
        modifier = modifier,
    )
}
