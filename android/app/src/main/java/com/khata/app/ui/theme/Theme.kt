package com.khata.app.ui.theme

import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.ExperimentalTextApi
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontVariation
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.khata.app.R

// DESIGN.md §2.1 — "Bahi by lamplight". Same hex values as frontend/src/index.css.
object KhataColors {
    val bg = Color(0xFF12110F)
    val surface = Color(0xFF1B1916)
    val surface2 = Color(0xFF24211D)
    val surface3 = Color(0xFF2E2A25)
    val hairline = Color(0xFF34302A)
    val rule = Color(0xFF4D463D)
    val text = Color(0xFFF1ECE3)
    val text2 = Color(0xFFB7AE9F)
    val textMuted = Color(0xFF968C7D)
    val brand = Color(0xFF6F8CFF)        // actions only
    val brandPress = Color(0xFF5A78F0)
    val brandSoft = Color(0x296F8CFF)    // rgba(111,140,255,.16)
    val onBrand = Color(0xFF0E1226)
    val income = Color(0xFF52C78D)       // money in only
    val incomeSoft = Color(0x2452C78D)
    val expense = Color(0xFFF26B4F)      // money out + destructive only
    val expenseSoft = Color(0x24F26B4F)
    val warn = Color(0xFFE8A33D)         // needs attention
    val warnSoft = Color(0x24E8A33D)
}

/**
 * Categorical chart palette for the dark surface — identical to
 * frontend/src/components/charts/palette.ts (validated for CVD separation).
 * No green/red: those mean money in/out. 7th+ category folds into [chartOther].
 */
val chartPalette = listOf(
    Color(0xFF3987E5), Color(0xFFD95926), Color(0xFF199E70),
    Color(0xFFC98500), Color(0xFFD55181), Color(0xFF9085E9),
)
val chartOther = Color(0xFF6E675E)

// DESIGN.md §2.2 — Schibsted Grotesk for UI, Space Mono for money and dates.
@OptIn(ExperimentalTextApi::class)
private fun schibsted(weight: FontWeight) = Font(
    R.font.schibsted_grotesk,
    weight = weight,
    variationSettings = FontVariation.Settings(FontVariation.weight(weight.weight)),
)

val SchibstedGrotesk = FontFamily(
    schibsted(FontWeight.Normal),
    schibsted(FontWeight.Medium),
    schibsted(FontWeight.SemiBold),
    schibsted(FontWeight.Bold),
)

val SpaceMono = FontFamily(
    Font(R.font.space_mono_regular, FontWeight.Normal),
    Font(R.font.space_mono_bold, FontWeight.Bold),
)

/** Monetary figures: Space Mono bold, digits line up in columns. */
val FigureStyle = TextStyle(fontFamily = SpaceMono, fontWeight = FontWeight.Bold, letterSpacing = (-0.02).sp)

/** Dates and other short data labels. */
val MonoStyle = TextStyle(fontFamily = SpaceMono, fontWeight = FontWeight.Normal, fontSize = 12.sp)

private fun ui(size: Int, weight: FontWeight, tracking: Double = 0.0, lineHeight: Int? = null) = TextStyle(
    fontFamily = SchibstedGrotesk,
    fontWeight = weight,
    fontSize = size.sp,
    letterSpacing = tracking.sp,
    lineHeight = (lineHeight ?: (size * 1.4).toInt()).sp,
)

val KhataTypography = Typography(
    displayLarge = ui(44, FontWeight.Bold, -0.6),
    displayMedium = ui(36, FontWeight.Bold, -0.5),
    displaySmall = ui(28, FontWeight.Bold, -0.4),      // screen titles
    headlineLarge = ui(28, FontWeight.Bold, -0.4),
    headlineMedium = ui(24, FontWeight.Bold, -0.3),
    headlineSmall = ui(21, FontWeight.SemiBold, -0.2),
    titleLarge = ui(21, FontWeight.SemiBold, -0.2),    // top app bar
    titleMedium = ui(18, FontWeight.SemiBold, -0.1),   // card titles
    titleSmall = ui(15, FontWeight.SemiBold),
    bodyLarge = ui(15, FontWeight.Normal, lineHeight = 22),
    bodyMedium = ui(14, FontWeight.Normal, lineHeight = 20),
    bodySmall = ui(12, FontWeight.Normal, lineHeight = 16),
    labelLarge = ui(14, FontWeight.SemiBold),          // buttons
    labelMedium = ui(12, FontWeight.Medium),
    labelSmall = ui(12, FontWeight.Medium),
)

// DESIGN.md §2.3 — radius follows hierarchy: panels lg, controls md, chips pill.
object KhataRadius {
    val sm = RoundedCornerShape(6.dp)
    val md = RoundedCornerShape(10.dp)
    val lg = RoundedCornerShape(14.dp)
    val pill = RoundedCornerShape(999.dp)
}

val KhataShapes = Shapes(
    extraSmall = KhataRadius.sm,
    small = KhataRadius.sm,
    medium = KhataRadius.md,
    large = KhataRadius.lg,
    extraLarge = KhataRadius.lg,
)

private val DarkColors = darkColorScheme(
    primary = KhataColors.brand,
    onPrimary = KhataColors.onBrand,
    primaryContainer = KhataColors.brandSoft,
    onPrimaryContainer = KhataColors.text,
    inversePrimary = KhataColors.brandPress,
    secondary = KhataColors.text2,
    onSecondary = KhataColors.bg,
    secondaryContainer = KhataColors.brandSoft,
    onSecondaryContainer = KhataColors.text,
    tertiary = KhataColors.warn,
    onTertiary = KhataColors.bg,
    tertiaryContainer = KhataColors.surface3,
    onTertiaryContainer = KhataColors.text,
    error = KhataColors.expense,
    onError = KhataColors.bg,
    errorContainer = KhataColors.expenseSoft,
    onErrorContainer = KhataColors.expense,
    background = KhataColors.bg,
    onBackground = KhataColors.text,
    surface = KhataColors.surface,
    onSurface = KhataColors.text,
    surfaceVariant = KhataColors.surface2,
    onSurfaceVariant = KhataColors.text2,
    surfaceTint = Color.Transparent,
    surfaceContainerLowest = KhataColors.bg,
    surfaceContainerLow = KhataColors.surface,
    surfaceContainer = KhataColors.surface,
    surfaceContainerHigh = KhataColors.surface2,
    surfaceContainerHighest = KhataColors.surface3,
    inverseSurface = KhataColors.text,
    inverseOnSurface = KhataColors.bg,
    outline = KhataColors.rule,
    outlineVariant = KhataColors.hairline,
    scrim = Color(0xB3080706),
)

@Composable
fun KhataTheme(
    darkTheme: Boolean = false,
    content: @Composable () -> Unit
) {
    // Dark only until light reaches parity on both platforms (DESIGN.md §2.1).
    MaterialTheme(
        colorScheme = DarkColors,
        typography = KhataTypography,
        shapes = KhataShapes,
        content = content
    )
}
