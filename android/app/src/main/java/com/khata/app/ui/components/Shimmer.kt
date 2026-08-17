package com.khata.app.ui.components

import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.unit.dp
import com.khata.app.ui.theme.KhataColors

@Composable
fun ShimmerBrush(): Brush {
    val transition = rememberInfiniteTransition(label = "shimmer")
    val translateX by transition.animateFloat(
        initialValue = -400f,
        targetValue = 800f,
        animationSpec = infiniteRepeatable(
            animation = tween(1200, easing = LinearEasing),
            repeatMode = RepeatMode.Restart
        ),
        label = "shimmer_x"
    )
    return Brush.linearGradient(
        colors = listOf(
            KhataColors.surface2,
            KhataColors.surface3,
            KhataColors.surface2,
        ),
        start = Offset(translateX, 0f),
        end = Offset(translateX + 400f, 0f)
    )
}

@Composable
fun ShimmerBox(
    modifier: Modifier = Modifier,
    brush: Brush = ShimmerBrush()
) {
    Box(
        modifier = modifier
            .clip(RoundedCornerShape(8.dp))
            .background(brush)
    )
}

@Composable
fun ShimmerCard(
    modifier: Modifier = Modifier,
    lines: Int = 3
) {
    val brush = ShimmerBrush()
    androidx.compose.material3.Card(
        modifier = modifier.fillMaxWidth(),
        shape = RoundedCornerShape(12.dp),
        colors = androidx.compose.material3.CardDefaults.cardColors(
            containerColor = KhataColors.surface
        )
    ) {
        Column(
            modifier = Modifier.padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            repeat(lines) { i ->
                ShimmerBox(
                    brush = brush,
                    modifier = Modifier
                        .fillMaxWidth(if (i == lines - 1) 0.6f else 1f)
                        .height(14.dp)
                )
            }
        }
    }
}

@Composable
fun ShimmerStatCard(modifier: Modifier = Modifier) {
    val brush = ShimmerBrush()
    androidx.compose.material3.Card(
        modifier = modifier,
        shape = RoundedCornerShape(12.dp),
        colors = androidx.compose.material3.CardDefaults.cardColors(
            containerColor = KhataColors.surface
        )
    ) {
        Column(
            modifier = Modifier.padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp)
        ) {
            Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                ShimmerBox(brush = brush, modifier = Modifier.size(36.dp))
                ShimmerBox(brush = brush, modifier = Modifier.width(60.dp).height(12.dp))
            }
            ShimmerBox(brush = brush, modifier = Modifier.width(100.dp).height(22.dp))
        }
    }
}

@Composable
fun ShimmerTransactionRow(modifier: Modifier = Modifier) {
    val brush = ShimmerBrush()
    Row(
        modifier = modifier
            .fillMaxWidth()
            .padding(horizontal = 16.dp, vertical = 12.dp),
        horizontalArrangement = Arrangement.spacedBy(12.dp)
    ) {
        ShimmerBox(brush = brush, modifier = Modifier.size(36.dp))
        Column(
            modifier = Modifier.weight(1f),
            verticalArrangement = Arrangement.spacedBy(8.dp)
        ) {
            ShimmerBox(brush = brush, modifier = Modifier.fillMaxWidth(0.7f).height(14.dp))
            ShimmerBox(brush = brush, modifier = Modifier.fillMaxWidth(0.3f).height(10.dp))
        }
        ShimmerBox(brush = brush, modifier = Modifier.width(72.dp).height(14.dp))
    }
}
