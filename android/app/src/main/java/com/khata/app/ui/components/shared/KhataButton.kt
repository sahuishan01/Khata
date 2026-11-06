package com.khata.app.ui.components.shared

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.size
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.khata.app.ui.theme.KhataColors
import com.khata.app.ui.theme.KhataRadius

enum class ButtonVariant { Primary, Secondary, Danger, Ghost }

/** DESIGN.md §4.4 — one primary per screen; destructive = expense outline. */
@Composable
fun KhataButton(
    onClick: () -> Unit,
    variant: ButtonVariant = ButtonVariant.Primary,
    modifier: Modifier = Modifier,
    enabled: Boolean = true,
    content: @Composable () -> Unit,
) {
    val m = modifier.height(48.dp)
    val pad = PaddingValues(horizontal = 20.dp)
    when (variant) {
        ButtonVariant.Primary -> Button(
            onClick = onClick, enabled = enabled, modifier = m, shape = KhataRadius.md, contentPadding = pad,
            colors = ButtonDefaults.buttonColors(
                containerColor = KhataColors.brand, contentColor = KhataColors.onBrand,
                disabledContainerColor = KhataColors.surface3, disabledContentColor = KhataColors.textMuted,
            )
        ) { content() }
        ButtonVariant.Secondary -> OutlinedButton(
            onClick = onClick, enabled = enabled, modifier = m, shape = KhataRadius.md, contentPadding = pad,
            colors = ButtonDefaults.outlinedButtonColors(contentColor = KhataColors.text),
            border = BorderStroke(1.dp, KhataColors.rule)
        ) { content() }
        ButtonVariant.Danger -> OutlinedButton(
            onClick = onClick, enabled = enabled, modifier = m, shape = KhataRadius.md, contentPadding = pad,
            colors = ButtonDefaults.outlinedButtonColors(contentColor = KhataColors.expense),
            border = BorderStroke(1.dp, KhataColors.expense)
        ) { content() }
        ButtonVariant.Ghost -> TextButton(
            onClick = onClick, enabled = enabled, modifier = m, shape = KhataRadius.md, contentPadding = pad,
            colors = ButtonDefaults.textButtonColors(contentColor = KhataColors.text2)
        ) { content() }
    }
}

/** Icon-sized button; still a 48dp touch target (DESIGN.md §2.3). */
@Composable
fun KhataSmallButton(onClick: () -> Unit, variant: ButtonVariant = ButtonVariant.Ghost, modifier: Modifier = Modifier, content: @Composable () -> Unit) {
    IconButton(onClick = onClick, modifier = modifier.size(48.dp)) { content() }
}
