package com.khata.app.ui.theme

import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.sp

object KhataColors {
    val bg = Color(0xFF0C0C11)
    val surface = Color(0xFF14141C)
    val surface2 = Color(0xFF1A1A24)
    val surface3 = Color(0xFF21212E)
    val hairline = Color(0x12FFFFFF)
    val text = Color(0xFFF2F2F5)
    val text2 = Color(0xFF9A9AA8)
    val textMuted = Color(0xFF6B6B78)
    val brand = Color(0xFF8479F2)
    val brandPress = Color(0xFF6F62E6)
    val brandSoft = Color(0x288479F2)
    val income = Color(0xFF2EC27E)
    val incomeSoft = Color(0x242EC27E)
    val expense = Color(0xFFEE6B4D)
    val expenseSoft = Color(0x24EE6B4D)
    val warn = Color(0xFFE0A33A)
}

val KhataTypography = Typography(
    bodyMedium = TextStyle(
        fontFamily = FontFamily.Default,
        fontWeight = FontWeight.Normal,
        fontSize = 14.sp,
        fontFeatureSettings = "tnum"
    ),
    titleMedium = TextStyle(
        fontFamily = FontFamily.Default,
        fontWeight = FontWeight.Medium,
        fontSize = 16.sp,
        fontFeatureSettings = "tnum"
    ),
    labelSmall = TextStyle(
        fontFamily = FontFamily.Default,
        fontWeight = FontWeight.Medium,
        fontSize = 11.sp,
        fontFeatureSettings = "tnum"
    ),
)

private val DarkColors = darkColorScheme(
    primary = KhataColors.brand,
    onPrimary = Color.White,
    primaryContainer = KhataColors.brandSoft,
    onPrimaryContainer = KhataColors.text,
    secondary = Color(0xFF6F62E6),
    onSecondary = Color.White,
    secondaryContainer = Color(0x286F62E6),
    error = KhataColors.expense,
    onError = Color.White,
    errorContainer = KhataColors.expenseSoft,
    background = KhataColors.bg,
    onBackground = KhataColors.text,
    surface = KhataColors.surface,
    onSurface = KhataColors.text,
    surfaceVariant = KhataColors.surface2,
    onSurfaceVariant = KhataColors.text2,
    surfaceTint = KhataColors.surface3,
    outline = KhataColors.hairline,
    outlineVariant = KhataColors.hairline,
)

@Composable
fun KhataTheme(
    darkTheme: Boolean = false,
    content: @Composable () -> Unit
) {
    MaterialTheme(
        colorScheme = DarkColors,
        typography = KhataTypography,
        content = content
    )
}
