package com.khata.app.ui.charts

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
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
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.khata.app.api.CategoryBucket
import com.khata.app.ui.theme.FigureStyle
import com.khata.app.ui.theme.KhataColors
import com.khata.app.ui.theme.KhataRadius
import com.khata.app.ui.theme.MonoStyle
import com.khata.app.ui.theme.chartOther
import com.khata.app.ui.theme.chartPalette
import com.khata.app.util.formatINR

/** Fixed-order palette; a 7th+ category folds into "Other" (never cycled). */
private fun fold(data: List<CategoryBucket>): List<Pair<CategoryBucket, Color>> {
    val head = if (data.size <= chartPalette.size) data else data.take(chartPalette.size - 1)
    val out = head.mapIndexed { i, b -> b to chartPalette[i] }.toMutableList()
    if (data.size > chartPalette.size) {
        val tail = data.drop(chartPalette.size - 1)
        out += CategoryBucket("Other", tail.sumOf { it.amount }, tail.sumOf { it.txnCount }, tail.sumOf { it.pct }) to chartOther
    }
    return out
}

@Composable
fun CategoryPieChart(
    data: List<CategoryBucket>,
    modifier: Modifier = Modifier,
    onCategoryClick: ((String) -> Unit)? = null
) {
    if (data.isEmpty()) return
    val total = data.sumOf { it.amount }
    val slices = fold(data)

    Column(modifier = modifier.fillMaxWidth(), horizontalAlignment = Alignment.CenterHorizontally) {
        Box(Modifier.size(176.dp), contentAlignment = Alignment.Center) {
            Canvas(Modifier.matchParentSize()) {
                val stroke = 26.dp.toPx()
                val gap = 1.2f // degrees of surface between slices
                var start = -90f
                slices.forEach { (b, color) ->
                    val sweep = (b.amount / total * 360).toFloat()
                    drawArc(
                        color = color,
                        startAngle = start + gap / 2,
                        sweepAngle = (sweep - gap).coerceAtLeast(0.5f),
                        useCenter = false,
                        style = Stroke(width = stroke),
                        size = Size(size.width - stroke, size.height - stroke),
                        topLeft = Offset(stroke / 2, stroke / 2),
                    )
                    start += sweep
                }
            }
            Column(horizontalAlignment = Alignment.CenterHorizontally) {
                Text("Spent", style = MaterialTheme.typography.bodySmall, color = KhataColors.text2)
                Text(formatINR(total), style = FigureStyle.copy(fontSize = 15.sp), color = KhataColors.text)
            }
        }
        Spacer(Modifier.height(12.dp))
        // Legend doubles as the table view: name, amount, share.
        slices.forEach { (b, color) ->
            val clickable = onCategoryClick != null && b.category != "Other"
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .heightIn(min = 44.dp)
                    .clip(KhataRadius.sm)
                    .then(if (clickable) Modifier.clickable { onCategoryClick!!(b.category) } else Modifier)
                    .padding(horizontal = 4.dp),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(10.dp)
            ) {
                Box(Modifier.size(10.dp).clip(KhataRadius.sm).background(color))
                Text(
                    b.category, style = MaterialTheme.typography.bodyMedium, color = KhataColors.text,
                    modifier = Modifier.weight(1f), maxLines = 1, overflow = TextOverflow.Ellipsis,
                )
                Text(formatINR(b.amount), style = FigureStyle.copy(fontSize = 13.sp), color = KhataColors.text)
                Text(
                    "${"%.1f".format(b.pct)}%", style = MonoStyle, color = KhataColors.text2,
                    modifier = Modifier.widthIn(min = 48.dp), textAlign = androidx.compose.ui.text.style.TextAlign.End,
                )
            }
        }
    }
}
