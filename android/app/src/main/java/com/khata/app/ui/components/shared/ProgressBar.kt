package com.khata.app.ui.components.shared

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.unit.dp
import com.khata.app.ui.theme.KhataColors

@Composable
fun KhataProgressBar(pct: Double, modifier: Modifier = Modifier) {
    val barColor = when {
        pct >= 100 -> KhataColors.expense
        pct >= 80 -> KhataColors.warn
        else -> KhataColors.income
    }
    Box(modifier = modifier.fillMaxWidth().height(6.dp).clip(RoundedCornerShape(3.dp)).background(KhataColors.hairline)) {
        Box(modifier = Modifier.fillMaxWidth(fraction = (pct / 100.0).toFloat().coerceIn(0f, 1f)).height(6.dp).clip(RoundedCornerShape(3.dp)).background(barColor))
    }
}
