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
    icon: @Composable () -> Unit,
    accentColor: Color = KhataColors.brand,
    subtitle: String? = null,
    modifier: Modifier = Modifier
) {
    KhataCard(modifier = modifier) {
        Row(verticalAlignment = Alignment.CenterVertically, modifier = Modifier.padding(start = 16.dp, end = 16.dp, top = 16.dp)) {
            Box(
                modifier = Modifier.size(36.dp).background(accentColor.copy(alpha = 0.1f), RoundedCornerShape(8.dp)),
                contentAlignment = Alignment.Center
            ) {
                icon()
            }
            Spacer(Modifier.width(10.dp))
            Column {
                Text(
                    text = label,
                    fontSize = 11.sp,
                    fontWeight = FontWeight.SemiBold,
                    color = KhataColors.textMuted,
                    letterSpacing = 0.6.sp
                )
                Text(
                    text = value,
                    fontSize = 20.sp,
                    fontWeight = FontWeight.Bold,
                    color = accentColor,
                    letterSpacing = (-0.5).sp,
                    style = LocalTextStyle.current.copy(fontFeatureSettings = "tnum")
                )
                if (subtitle != null) {
                    Text(text = subtitle, fontSize = 11.sp, color = KhataColors.textMuted)
                }
            }
        }
    }
}
