package com.khata.app.ui.charts

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.layout.*
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.nativeCanvas
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
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.spacedBy(4.dp)
        ) {
            reversed.forEach { month ->
                Column(
                    modifier = Modifier.weight(1f),
                    horizontalAlignment = androidx.compose.ui.Alignment.CenterHorizontally
                ) {
                    Box(
                        modifier = Modifier
                            .fillMaxWidth()
                            .height(120.dp)
                    ) {
                        Canvas(modifier = Modifier.fillMaxSize()) {
                            val barWidth = size.width * 0.3f
                            val spacing = (size.width - barWidth * 2) / 3
                            val spentHeight = (month.spent / maxVal * size.height * 0.9).toFloat()
                            val earnedHeight = (month.earned / maxVal * size.height * 0.9).toFloat()

                            // Spent bar
                            drawRect(
                                color = SpendColor,
                                topLeft = Offset(spacing, size.height - spentHeight),
                                size = Size(barWidth, spentHeight)
                            )
                            // Earned bar
                            drawRect(
                                color = EarnColor,
                                topLeft = Offset(spacing * 2 + barWidth, size.height - earnedHeight),
                                size = Size(barWidth, earnedHeight)
                            )
                        }
                    }
                    Text(
                        month.month.takeLast(2),
                        fontSize = 9.sp,
                        color = MaterialTheme.colorScheme.onSurfaceVariant
                    )
                }
            }
        }

        Row(
            modifier = Modifier.fillMaxWidth().padding(top = 4.dp),
            horizontalArrangement = Arrangement.Center,
            verticalAlignment = androidx.compose.ui.Alignment.CenterVertically
        ) {
            Box(
                modifier = Modifier
                    .size(10.dp)
                    .padding(0.dp)
            ) {
                Canvas(Modifier.fillMaxSize()) { drawRect(color = SpendColor) }
            }
            Text(" Spent  ", fontSize = 11.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
            Box(
                modifier = Modifier
                    .size(10.dp)
                    .padding(0.dp)
            ) {
                Canvas(Modifier.fillMaxSize()) { drawRect(color = EarnColor) }
            }
            Text(" Earned", fontSize = 11.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
        }
    }
}
