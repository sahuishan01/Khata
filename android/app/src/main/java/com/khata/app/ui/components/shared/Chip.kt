package com.khata.app.ui.components.shared

import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.khata.app.ui.theme.KhataColors
import com.khata.app.ui.theme.KhataRadius

/** Purple is kept as an alias of Brand for older call sites. */
enum class ChipColor { Brand, Purple, Green, Red, Amber, Gray }

@Composable
fun KhataChip(
    text: String,
    color: ChipColor = ChipColor.Gray,
    modifier: Modifier = Modifier,
    onClick: (() -> Unit)? = null,
) {
    val (bg, fg) = when (color) {
        ChipColor.Brand, ChipColor.Purple -> KhataColors.brandSoft to KhataColors.brand
        ChipColor.Green -> KhataColors.incomeSoft to KhataColors.income
        ChipColor.Red -> KhataColors.expenseSoft to KhataColors.expense
        ChipColor.Amber -> KhataColors.warnSoft to KhataColors.warn
        ChipColor.Gray -> KhataColors.surface2 to KhataColors.text2
    }
    Surface(
        onClick = onClick ?: {}, shape = KhataRadius.pill,
        color = bg, modifier = modifier,
        enabled = onClick != null
    ) {
        Text(text, fontSize = 12.sp, fontWeight = FontWeight.SemiBold, color = fg,
            maxLines = 1, overflow = TextOverflow.Ellipsis, softWrap = false,
            modifier = Modifier.padding(horizontal = 10.dp, vertical = 4.dp))
    }
}
