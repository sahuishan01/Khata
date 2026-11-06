package com.khata.app.ui.components.shared

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import com.khata.app.ui.theme.KhataColors
import com.khata.app.ui.theme.KhataRadius

@Composable
fun KhataField(
    value: String,
    onValueChange: (String) -> Unit,
    placeholder: String = "",
    modifier: Modifier = Modifier,
    singleLine: Boolean = true,
) {
    OutlinedTextField(
        value = value, onValueChange = onValueChange,
        placeholder = { if (placeholder.isNotEmpty()) Text(placeholder, style = MaterialTheme.typography.bodyMedium) },
        singleLine = singleLine, modifier = modifier.fillMaxWidth(),
        shape = KhataRadius.md,
        textStyle = MaterialTheme.typography.bodyLarge,
        colors = OutlinedTextFieldDefaults.colors(
            focusedBorderColor = KhataColors.brand,
            unfocusedBorderColor = KhataColors.hairline,
            focusedContainerColor = KhataColors.surface2,
            unfocusedContainerColor = KhataColors.surface2,
            cursorColor = KhataColors.brand,
            focusedPlaceholderColor = KhataColors.textMuted,
            unfocusedPlaceholderColor = KhataColors.textMuted,
        )
    )
}
