package com.khata.app.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.khata.app.ui.components.shared.KhataCard
import com.khata.app.ui.theme.KhataColors

@Composable
fun StatCard(
    label: String,
    value: String,
    accentColor: Color = KhataColors.brand,
    icon: @Composable (() -> Unit)? = null,
    subtitle: String? = null,
    modifier: Modifier = Modifier
) {
    com.khata.app.ui.components.shared.KhataCard(modifier = modifier) {
        Column(modifier = Modifier.padding(11.dp)) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                if (icon != null) { icon(); Spacer(Modifier.width(8.dp)) }
                Text(label, fontSize = 9.5.sp, color = KhataColors.text2)
            }
            Text(value, fontSize = 16.sp, fontWeight = FontWeight.Bold, color = accentColor,
                style = LocalTextStyle.current.copy(fontFeatureSettings = "tnum"))
            if (subtitle != null) { Text(subtitle, fontSize = 9.5.sp, color = KhataColors.textMuted) }
        }
    }
}
