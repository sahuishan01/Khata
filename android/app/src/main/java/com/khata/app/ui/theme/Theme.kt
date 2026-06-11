package com.khata.app.ui.theme

import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color

private val LightColors = lightColorScheme(
    primary = Color(0xFF6C5CE7),
    onPrimary = Color.White,
    primaryContainer = Color(0xFFEDE6FF),
    onPrimaryContainer = Color(0xFF3B2A8A),
    secondary = Color(0xFF00B894),
    onSecondary = Color.White,
    secondaryContainer = Color(0xFFE0FFF5),
    error = Color(0xFFE17055),
    onError = Color.White,
    errorContainer = Color(0xFFFFE8E0),
    background = Color(0xFFF5F3F0),
    onBackground = Color(0xFF1A1628),
    surface = Color.White,
    onSurface = Color(0xFF1A1628),
    surfaceVariant = Color(0xFFF8F7F4),
    onSurfaceVariant = Color(0xFF4A4658),
    outline = Color(0xFFECE9E3),
    outlineVariant = Color(0xFFD6D2C8),
)

private val DarkColors = darkColorScheme(
    primary = Color(0xFF8B7CF2),
    onPrimary = Color.White,
    primaryContainer = Color(0xFF3B2A8A),
    onPrimaryContainer = Color(0xFFEDE6FF),
    secondary = Color(0xFF00D2A0),
    onSecondary = Color.White,
    secondaryContainer = Color(0xFF004D3D),
    error = Color(0xFFE17055),
    onError = Color.White,
    errorContainer = Color(0xFF5C1A06),
    background = Color(0xFF121016),
    onBackground = Color(0xFFEAE6F0),
    surface = Color(0xFF1B1823),
    onSurface = Color(0xFFEAE6F0),
    surfaceVariant = Color(0xFF221F2D),
    onSurfaceVariant = Color(0xFF9D97AD),
    outline = Color(0xFF2A2638),
    outlineVariant = Color(0xFF3A3550),
)

@Composable
fun KhataTheme(
    darkTheme: Boolean = false,
    content: @Composable () -> Unit
) {
    val colorScheme = if (darkTheme) DarkColors else LightColors
    MaterialTheme(
        colorScheme = colorScheme,
        content = content
    )
}
