package com.khata.app.ui.components.shared

import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.khata.app.ui.theme.KhataColors

enum class ChipColor { Purple, Green, Red, Amber, Gray }

@Composable
fun KhataChip(
    text: String,
    color: ChipColor = ChipColor.Gray,
    modifier: Modifier = Modifier,
    onClick: (() -> Unit)? = null,
) {
    val (bg, fg) = when (color) {
        ChipColor.Purple -> KhataColors.brandSoft to KhataColors.brand
        ChipColor.Green -> KhataColors.incomeSoft to KhataColors.income
        ChipColor.Red -> KhataColors.expenseSoft to KhataColors.expense
        ChipColor.Amber -> Color(0x24E0A33A) to KhataColors.warn
        ChipColor.Gray -> KhataColors.surface2 to KhataColors.text2
    }
    Surface(
        onClick = onClick ?: {}, shape = RoundedCornerShape(999.dp),
        color = bg, modifier = modifier,
        enabled = onClick != null
    ) {
        Text(text, fontSize = 11.sp, fontWeight = FontWeight.SemiBold, color = fg,
            maxLines = 1, overflow = TextOverflow.Ellipsis, softWrap = false,
            modifier = Modifier.padding(horizontal = 8.dp, vertical = 4.dp))
    }
}
