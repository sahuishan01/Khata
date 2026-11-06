package com.khata.app.ui.components.shared

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.khata.app.ui.theme.KhataColors
import com.khata.app.ui.theme.KhataRadius

@Composable
fun KhataCard(modifier: Modifier = Modifier, content: @Composable ColumnScope.() -> Unit) {
    Surface(
        modifier = modifier,
        shape = KhataRadius.lg,
        color = KhataColors.surface,
        border = BorderStroke(1.dp, KhataColors.hairline),
    ) {
        Column(content = content)
    }
}

@Composable
fun KhataCardHeader(
    title: String,
    modifier: Modifier = Modifier,
    subtitle: String? = null,
    action: @Composable () -> Unit = {},
) {
    Row(
        modifier = modifier.fillMaxWidth().padding(start = 16.dp, end = 12.dp, top = 16.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Column(Modifier.weight(1f)) {
            Text(title, style = MaterialTheme.typography.titleMedium, color = KhataColors.text)
            if (subtitle != null) {
                Text(subtitle, style = MaterialTheme.typography.bodySmall, color = KhataColors.text2)
            }
        }
        action()
    }
}

@Composable
fun KhataCardBody(modifier: Modifier = Modifier, content: @Composable ColumnScope.() -> Unit) {
    Column(modifier = modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) { content() }
}
