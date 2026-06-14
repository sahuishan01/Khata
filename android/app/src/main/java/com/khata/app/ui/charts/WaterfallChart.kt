package com.khata.app.ui.charts

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.khata.app.api.MonthBucket
import com.khata.app.ui.theme.KhataColors

private val NetPosColor = KhataColors.brand
private val NetNegColor = KhataColors.expense

@Composable
fun WaterfallChart(
    data: List<MonthBucket>,
    modifier: Modifier = Modifier
) {
    if (data.isEmpty()) return

    val reversed = data.reversed()
    val nets = reversed.map { it.earned - it.spent }
    val maxVal = maxOf(data.maxOf { it.spent }, data.maxOf { it.earned }, nets.maxOf { kotlin.math.abs(it) })

    Column(modifier = modifier.fillMaxWidth()) {
        Canvas(
            modifier = Modifier.fillMaxWidth().height(160.dp)
        ) {
            val barWidth = size.width / (nets.size * 2f)
            val chartH = size.height * 0.9f
            val zeroY = size.height

            nets.forEachIndexed { i, net ->
                val x = i * barWidth * 2f + barWidth / 2
                val barH = (kotlin.math.abs(net) / maxVal * chartH).toFloat().coerceAtMost(chartH)
                val y = zeroY - (if (net >= 0) barH else 0f)

                drawRect(
                    color = if (net >= 0) NetPosColor else NetNegColor,
                    topLeft = Offset(x, y),
                    size = Size(barWidth, barH.coerceAtLeast(1f))
                )
            }
        }

        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween
        ) {
            reversed.forEach { month ->
                Text(
                    month.month.takeLast(2),
                    fontSize = 9.sp,
                    color = KhataColors.text2,
                    modifier = Modifier.padding(horizontal = 4.dp)
                )
            }
        }

        Row(
            modifier = Modifier.fillMaxWidth().padding(top = 4.dp),
            horizontalArrangement = Arrangement.Center
        ) {
            Box(modifier = Modifier.size(10.dp, 10.dp).clip(RoundedCornerShape(2.dp)).background(NetPosColor))
            Text(" Surplus  ", fontSize = 11.sp, color = KhataColors.text2)
            Box(modifier = Modifier.size(10.dp, 10.dp).clip(RoundedCornerShape(2.dp)).background(NetNegColor))
            Text(" Deficit", fontSize = 11.sp, color = KhataColors.text2)
        }
    }
}
