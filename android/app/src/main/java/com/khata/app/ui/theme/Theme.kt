package com.khata.app.ui.theme

import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

// DESIGN.md §2.1 — authoritative tokens
object KhataColors {
    val bg = Color(0xFF0E0E13)
    val surface = Color(0xFF17171F)
    val surface2 = Color(0xFF1F1F2A)
    val surface3 = Color(0xFF28283A)
    val hairline = Color(0xFF2A2A38)
    val text = Color(0xFFF2F2F5)
    val text2 = Color(0xFF9A9AA8)
    val textMuted = Color(0xFF6B6B78)
    val brand = Color(0xFF8479F2)
    val brandPress = Color(0xFF6F62E6)
    val brandSoft = Color(0x298479F2)  // rgba(132,121,242,.16)
    val income = Color(0xFF2EC27E)
    val incomeSoft = Color(0x242EC27E)
    val expense = Color(0xFFEE6B4D)
    val expenseSoft = Color(0x24EE6B4D)
    val warn = Color(0xFFE0A33A)
    val warnSoft = Color(0x24E0A33A)
}

// DESIGN.md §2.2 — typography roles
val KhataTypography = Typography(
    displayLarge = TextStyle(       // Display: 26 / 700 / -0.02em
        fontFamily = FontFamily.Default,
        fontWeight = FontWeight.Bold,
        fontSize = 26.sp,
        letterSpacing = (-0.02).sp,
        fontFeatureSettings = "tnum"
    ),
    headlineMedium = TextStyle(     // Title: 20 / 700
        fontFamily = FontFamily.Default,
        fontWeight = FontWeight.Bold,
        fontSize = 20.sp,
        fontFeatureSettings = "tnum"
    ),
    titleMedium = TextStyle(        // Body-strong: 15 / 600
        fontFamily = FontFamily.Default,
        fontWeight = FontWeight.SemiBold,
        fontSize = 15.sp,
        fontFeatureSettings = "tnum"
    ),
    bodyLarge = TextStyle(          // Body: 15 / 500
        fontFamily = FontFamily.Default,
        fontWeight = FontWeight.Medium,
        fontSize = 15.sp,
        fontFeatureSettings = "tnum"
    ),
    bodyMedium = TextStyle(         // Body: 14 / 400 (secondary)
        fontFamily = FontFamily.Default,
        fontWeight = FontWeight.Normal,
        fontSize = 14.sp,
        fontFeatureSettings = "tnum"
    ),
    labelLarge = TextStyle(         // Label: 13 / 600
        fontFamily = FontFamily.Default,
        fontWeight = FontWeight.SemiBold,
        fontSize = 13.sp,
        letterSpacing = 0.08.sp,
        fontFeatureSettings = "tnum"
    ),
    labelSmall = TextStyle(         // Caption: 12 / 500
        fontFamily = FontFamily.Default,
        fontWeight = FontWeight.Medium,
        fontSize = 12.sp,
        fontFeatureSettings = "tnum"
    ),
)

// DESIGN.md §2.3 — radius scale
object KhataRadius {
    val sm = RoundedCornerShape(8.dp)
    val md = RoundedCornerShape(12.dp)
    val lg = RoundedCornerShape(16.dp)
    val pill = RoundedCornerShape(999.dp)
}

val KhataShapes = Shapes(
    small = KhataRadius.sm,
    medium = KhataRadius.md,
    large = KhataRadius.lg,
)

private val DarkColors = darkColorScheme(
    primary = KhataColors.brand,
    onPrimary = Color.White,
    primaryContainer = KhataColors.brandSoft,
    onPrimaryContainer = KhataColors.text,
    secondary = KhataColors.brandPress,
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
        shapes = KhataShapes,
        content = content
    )
}
