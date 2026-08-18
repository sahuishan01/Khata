package com.khata.app.ui.charts

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.LocalTextStyle
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.nativeCanvas
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.khata.app.api.CategoryBucket
import com.khata.app.util.formatINR

private val PieColors = listOf(
    Color(0xFF8479F2), Color(0xFF2EC27E), Color(0xFFEE6B4D),
    Color(0xFFE0A33A), Color(0xFF74B9FF), Color(0xFFA29BFE),
    Color(0xFF00F0FF), Color(0xFF05FFB0), Color(0xFFFF2A6D),
    Color(0xFFF59E0B), Color(0xFF10B981), Color(0xFF3B82F6),
)

@Composable
fun CategoryPieChart(
    data: List<CategoryBucket>,
    modifier: Modifier = Modifier,
    onCategoryClick: ((String) -> Unit)? = null
) {
    if (data.isEmpty()) return

    val total = data.sumOf { it.amount }

    Row(
        modifier = modifier.fillMaxWidth(),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(16.dp)
    ) {
        Canvas(
            modifier = Modifier.size(150.dp)
        ) {
            val strokeWidth = 32f
            var startAngle = -90f
            data.forEachIndexed { index, bucket ->
                val sweepAngle = (bucket.amount / total * 360).toFloat()
                drawArc(
                    color = PieColors[index % PieColors.size],
                    startAngle = startAngle,
                    sweepAngle = sweepAngle,
                    useCenter = false,
                    style = Stroke(width = strokeWidth),
                    size = Size(size.width - strokeWidth, size.height - strokeWidth),
                    topLeft = Offset(strokeWidth / 2, strokeWidth / 2)
                )
                startAngle += sweepAngle
            }
            drawContext.canvas.nativeCanvas.drawText(
                formatINR(total),
                size.width / 2,
                size.height / 2 + 10,
                android.graphics.Paint().apply {
                    color = android.graphics.Color.WHITE
                    textSize = 34f
                    textAlign = android.graphics.Paint.Align.CENTER
                    isFakeBoldText = true
                }
            )
        }

        Column(modifier = Modifier.weight(1f)) {
            data.forEachIndexed { index, bucket ->
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .then(if (onCategoryClick != null) Modifier.clickable { onCategoryClick(bucket.category) } else Modifier)
                        .padding(vertical = 2.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(6.dp)
                ) {
                    Box(
                        modifier = Modifier
                            .size(10.dp)
                            .clip(CircleShape)
                            .background(PieColors[index % PieColors.size])
                    )
                    Text(
                        bucket.category, fontSize = 12.sp,
                        color = MaterialTheme.colorScheme.onSurface,
                        modifier = Modifier.weight(1f),
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis
                    )
                    Text(formatINR(bucket.amount), fontSize = 12.sp,
                        fontWeight = FontWeight.SemiBold, color = MaterialTheme.colorScheme.onSurface,
                        style = LocalTextStyle.current.copy(fontFeatureSettings = "tnum"))
                    Text("${"%.1f".format(bucket.pct)}%", fontSize = 10.sp,
                        color = MaterialTheme.colorScheme.onSurfaceVariant)
                }
            }
        }
    }
}
