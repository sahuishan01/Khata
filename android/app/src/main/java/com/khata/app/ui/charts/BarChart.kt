package com.khata.app.ui.charts

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
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

private val SpendColor = Color(0xFFE17055)
private val EarnColor = Color(0xFF00B894)

@Composable
fun MonthlyBarChart(
    data: List<MonthBucket>,
    modifier: Modifier = Modifier
) {
    if (data.isEmpty()) return

    val maxVal = data.maxOf { maxOf(it.spent, it.earned) }
    val reversed = data.reversed()

    Column(modifier = modifier.fillMaxWidth()) {
        // Single Canvas for all bars
        Canvas(
            modifier = Modifier.fillMaxWidth().height(130.dp)
        ) {
            val stepX = size.width / reversed.size
            reversed.forEachIndexed { i, month ->
                val barWidth = stepX * 0.25f
                val spacing = (stepX - barWidth * 2) / 3
                val spentH = (month.spent / maxVal * size.height * 0.85f).toFloat()
                val earnedH = (month.earned / maxVal * size.height * 0.85f).toFloat()
                val x = i * stepX

                drawRect(SpendColor, Offset(x + spacing, size.height - spentH), Size(barWidth, spentH))
                drawRect(EarnColor, Offset(x + spacing * 2 + barWidth, size.height - earnedH), Size(barWidth, earnedH))
            }
        }

        // Labels
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceEvenly
        ) {
            reversed.forEach { month ->
                Text(month.month.takeLast(2), fontSize = 9.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
        }

        // Legend (Box instead of Canvas)
        Row(
            modifier = Modifier.fillMaxWidth().padding(top = 6.dp),
            horizontalArrangement = Arrangement.Center,
            verticalAlignment = Alignment.CenterVertically
        ) {
            Box(Modifier.size(10.dp).clip(RoundedCornerShape(2.dp)).background(SpendColor))
            Text(" Spent  ", fontSize = 11.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
            Box(Modifier.size(10.dp).clip(RoundedCornerShape(2.dp)).background(EarnColor))
            Text(" Earned", fontSize = 11.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
        }
    }
}
