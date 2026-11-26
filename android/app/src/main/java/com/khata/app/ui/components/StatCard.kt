package com.khata.app.ui.components

import androidx.compose.foundation.border
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.khata.app.ui.components.shared.KhataCard
import com.khata.app.ui.theme.FigureStyle
import com.khata.app.ui.theme.KhataColors
import com.khata.app.ui.theme.KhataRadius

/** Secondary metric. [accentColor] colors the value — pass text for neutral totals. */
@Composable
fun StatCard(
    label: String,
    value: String,
    accentColor: Color = KhataColors.text,
    icon: @Composable (() -> Unit)? = null,
    subtitle: String? = null,
    modifier: Modifier = Modifier
) {
    KhataCard(modifier = modifier.border(1.dp, KhataColors.hairline, KhataRadius.lg)) {
        Column(modifier = Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                if (icon != null) { icon() }
                Text(label, style = MaterialTheme.typography.bodyMedium, color = KhataColors.text2)
            }
            Text(value, style = FigureStyle.copy(fontSize = 22.sp), color = if (accentColor == KhataColors.brand) KhataColors.text else accentColor, maxLines = 1)
            if (subtitle != null) Text(subtitle, style = MaterialTheme.typography.bodySmall, color = KhataColors.text2)
        }
    }
}
