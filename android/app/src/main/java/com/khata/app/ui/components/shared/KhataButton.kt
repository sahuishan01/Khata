package com.khata.app.ui.components.shared

import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.unit.dp
import com.khata.app.ui.theme.KhataColors

enum class ButtonVariant { Primary, Secondary, Danger, Ghost }

@Composable
fun KhataButton(
    onClick: () -> Unit,
    variant: ButtonVariant = ButtonVariant.Primary,
    modifier: Modifier = Modifier,
    enabled: Boolean = true,
    content: @Composable () -> Unit,
) {
    when (variant) {
        ButtonVariant.Primary -> Button(
            onClick = onClick, enabled = enabled, modifier = modifier.height(44.dp),
            shape = RoundedCornerShape(999.dp), colors = ButtonDefaults.buttonColors(containerColor = KhataColors.brand)
        ) { content() }
        ButtonVariant.Secondary -> OutlinedButton(
            onClick = onClick, enabled = enabled, modifier = modifier.height(44.dp),
            shape = RoundedCornerShape(999.dp), colors = ButtonDefaults.outlinedButtonColors(contentColor = KhataColors.text),
            border = ButtonDefaults.outlinedButtonBorder(enabled = enabled)
        ) { content() }
        ButtonVariant.Danger -> OutlinedButton(
            onClick = onClick, enabled = enabled, modifier = modifier.height(44.dp),
            shape = RoundedCornerShape(999.dp), colors = ButtonDefaults.outlinedButtonColors(contentColor = KhataColors.expense),
            border = ButtonDefaults.outlinedButtonBorder(enabled = false).copy(brush = SolidColor(KhataColors.expense))
        ) { content() }
        ButtonVariant.Ghost -> TextButton(
            onClick = onClick, enabled = enabled, modifier = modifier.height(44.dp),
            colors = ButtonDefaults.textButtonColors(contentColor = KhataColors.text)
        ) { content() }
    }
}

@Composable
fun KhataSmallButton(onClick: () -> Unit, variant: ButtonVariant = ButtonVariant.Ghost, modifier: Modifier = Modifier, content: @Composable () -> Unit) {
    KhataButton(onClick = onClick, variant = variant, modifier = modifier.size(36.dp), content = content)
}
