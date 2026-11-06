package com.khata.app.ui.components.shared

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.*
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.khata.app.ui.theme.KhataColors
import com.khata.app.ui.theme.SpaceMono

/** The Khata mark: a ledger page with its red margin rule (matches web .sidebar-logo). */
@Composable
fun KhataMark(size: Dp = 32.dp) {
    val shape = androidx.compose.foundation.shape.RoundedCornerShape(size * 0.2f)
    Box(
        Modifier.size(size).clip(shape).background(KhataColors.surface3).border(1.dp, KhataColors.rule, shape),
        contentAlignment = Alignment.CenterEnd,
    ) {
        Canvas(Modifier.matchParentSize()) {
            val x = this.size.width * 0.3f
            drawLine(KhataColors.expense, Offset(x, 0f), Offset(x, this.size.height), strokeWidth = this.size.width * 0.06f)
        }
        Text(
            "₹", fontFamily = SpaceMono, fontWeight = FontWeight.Bold, color = KhataColors.text,
            fontSize = (size.value * 0.45f).sp, modifier = Modifier.padding(end = size * 0.16f),
        )
    }
}
