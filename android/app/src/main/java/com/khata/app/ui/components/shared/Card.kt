package com.khata.app.ui.components.shared

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.khata.app.ui.theme.KhataColors

@Composable
fun KhataCard(modifier: Modifier = Modifier, content: @Composable ColumnScope.() -> Unit) {
    Surface(
        modifier = modifier,
        shape = RoundedCornerShape(13.dp),
        color = KhataColors.surface,
        border = null,
    ) {
        Column(modifier = Modifier.drawBehind {
            drawRoundRect(
                color = KhataColors.hairline,
                cornerRadius = CornerRadius(13.dp.toPx()),
                style = Stroke(1.dp.toPx()),
            )
            // subtle top inner highlight
            drawRect(
                color = Color(0x0AFFFFFF),
                topLeft = Offset(0f, 0f),
                size = Size(size.width, 1.dp.toPx()),
            )
        }, verticalArrangement = Arrangement.spacedBy(12.dp)) {
            content()
        }
    }
}

@Composable
fun KhataCardHeader(title: String, modifier: Modifier = Modifier, action: @Composable () -> Unit = {}) {
    Row(
        modifier = modifier.padding(start = 12.dp, end = 12.dp, top = 12.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Text(title, fontSize = 9.sp, fontWeight = FontWeight.Bold, color = KhataColors.textMuted, letterSpacing = 0.5.sp, modifier = Modifier.weight(1f))
        action()
    }
}

@Composable
fun KhataCardBody(modifier: Modifier = Modifier, content: @Composable ColumnScope.() -> Unit) {
    Column(modifier = modifier.padding(12.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) { content() }
}
