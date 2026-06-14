package com.khata.app.ui.components.shared

import androidx.compose.foundation.layout.*
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.khata.app.ui.theme.KhataColors

@Composable
fun KhataEmptyState(
    emoji: String,
    title: String,
    description: String,
    actionLabel: String? = null,
    onAction: (() -> Unit)? = null,
) {
    Box(modifier = Modifier.fillMaxWidth().padding(vertical = 24.dp), contentAlignment = Alignment.Center) {
        Column(horizontalAlignment = Alignment.CenterHorizontally) {
            Text(emoji, fontSize = 32.sp)
            Spacer(Modifier.height(8.dp))
            Text(title, fontSize = 16.sp, fontWeight = FontWeight.SemiBold, color = KhataColors.text)
            Spacer(Modifier.height(4.dp))
            Text(description, fontSize = 13.sp, color = KhataColors.text2, textAlign = TextAlign.Center, modifier = Modifier.padding(horizontal = 24.dp))
            if (actionLabel != null && onAction != null) {
                Spacer(Modifier.height(16.dp))
                Button(onClick = onAction, colors = ButtonDefaults.buttonColors(containerColor = KhataColors.brand)) { Text(actionLabel) }
            }
        }
    }
}
