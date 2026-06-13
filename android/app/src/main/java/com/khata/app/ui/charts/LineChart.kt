package com.khata.app.ui.charts

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.layout.*
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.StrokeJoin
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.khata.app.api.MonthBucket

private val NetColor = Color(0xFF8479F2)
private val GreenColor = Color(0xFF2EC27E)
private val RedColor = Color(0xFFEE6B4D)

@Composable
fun NetWorthLineChart(
    data: List<MonthBucket>,
    modifier: Modifier = Modifier
) {
    if (data.isEmpty()) return

    val reversed = data.reversed()
    val nets = reversed.map { it.earned - it.spent }
    val maxNet = nets.maxOf { it }
    val minNet = nets.minOf { it }
    val range = (maxNet - minNet).coerceAtLeast(1.0)

    Column(modifier = modifier.fillMaxWidth()) {
        Canvas(
            modifier = Modifier.fillMaxWidth().height(180.dp)
        ) {
            val stepX = size.width / (nets.size - 1).coerceAtLeast(1)
            val chartHeight = size.height * 0.85f
            val zeroY = size.height - (if (range > 0) (0.0 - minNet) / range * chartHeight else chartHeight / 2).toFloat()

            // Grid lines
            for (i in 0..3) {
                val y = size.height - (size.height * 0.85f * i / 3)
                drawLine(
                    color = Color.LightGray.copy(alpha = 0.3f),
                    start = Offset(0f, y),
                    end = Offset(size.width, y),
                    strokeWidth = 1f
                )
            }

            // Zero line
            if (minNet < 0 && maxNet > 0) {
                drawLine(
                    color = Color.LightGray.copy(alpha = 0.5f),
                    start = Offset(0f, zeroY),
                    end = Offset(size.width, zeroY),
                    strokeWidth = 1f
                )
            }

            // Area fill
            val areaPath = Path().apply {
                nets.forEachIndexed { i, net ->
                    val x = i * stepX
                    val y = size.height - ((net - minNet) / range * chartHeight).toFloat()
                    if (i == 0) {
                        moveTo(x, zeroY)
                        lineTo(x, y)
                    } else {
                        lineTo(x, y)
                    }
                }
                lineTo((nets.size - 1) * stepX, zeroY)
                close()
            }
            drawPath(areaPath, NetColor.copy(alpha = 0.15f))

            // Line
            val linePath = Path().apply {
                nets.forEachIndexed { i, net ->
                    val x = i * stepX
                    val y = size.height - ((net - minNet) / range * chartHeight).toFloat()
                    if (i == 0) moveTo(x, y) else lineTo(x, y)
                }
            }
            drawPath(linePath, NetColor, style = Stroke(width = 3f, cap = StrokeCap.Round, join = StrokeJoin.Round))

            // Dots
            nets.forEachIndexed { i, net ->
                val x = i * stepX
                val y = size.height - ((net - minNet) / range * chartHeight).toFloat()
                val dotColor = if (net >= 0) GreenColor else RedColor
                drawCircle(dotColor, radius = 5f, center = Offset(x, y))
                drawCircle(Color.White, radius = 2.5f, center = Offset(x, y))
            }
        }

        // Labels
        Row(
            modifier = Modifier.fillMaxWidth().padding(top = 4.dp),
            horizontalArrangement = Arrangement.SpaceBetween
        ) {
            reversed.take(6).forEachIndexed { i, month ->
                Text(
                    month.month.takeLast(2),
                    fontSize = 9.sp,
                    color = MaterialTheme.colorScheme.onSurfaceVariant
                )
            }
        }
    }
}
