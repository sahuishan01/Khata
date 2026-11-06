package com.khata.app.ui.components.shared

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.*
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.khata.app.ui.theme.KhataColors
import com.khata.app.ui.theme.KhataRadius

/** DESIGN.md §4.3 — icon, one sentence of direction, and the primary action. */
@Composable
fun KhataEmptyState(
    emoji: String,
    title: String,
    description: String,
    actionLabel: String? = null,
    onAction: (() -> Unit)? = null,
) {
    Column(
        modifier = Modifier.fillMaxWidth().padding(vertical = 8.dp)
            .border(1.dp, KhataColors.rule, KhataRadius.lg)
            .padding(horizontal = 24.dp, vertical = 32.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        Box(
            Modifier.size(44.dp).clip(KhataRadius.md).background(KhataColors.surface2),
            contentAlignment = Alignment.Center,
        ) { Text(emoji, fontSize = 22.sp) }
        Spacer(Modifier.height(12.dp))
        Text(title, style = MaterialTheme.typography.titleMedium, color = KhataColors.text, textAlign = TextAlign.Center)
        Spacer(Modifier.height(4.dp))
        Text(description, style = MaterialTheme.typography.bodyMedium, color = KhataColors.text2, textAlign = TextAlign.Center)
        if (actionLabel != null && onAction != null) {
            Spacer(Modifier.height(16.dp))
            KhataButton(onClick = onAction) { Text(actionLabel) }
        }
    }
}
