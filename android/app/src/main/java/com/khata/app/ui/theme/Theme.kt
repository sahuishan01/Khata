package com.khata.app.ui.theme

import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
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

// DESIGN.md §2.1 — "Bahi by lamplight" (dark) & "Bahi by daylight" (light).
object KhataColors {
    var bg by mutableStateOf(Color(0xFF12110F))
    var surface by mutableStateOf(Color(0xFF1B1916))
    var surface2 by mutableStateOf(Color(0xFF24211D))
    var surface3 by mutableStateOf(Color(0xFF2E2A25))
    var hairline by mutableStateOf(Color(0xFF34302A))
    var rule by mutableStateOf(Color(0xFF4D463D))
    var text by mutableStateOf(Color(0xFFF1ECE3))
    var text2 by mutableStateOf(Color(0xFFB7AE9F))
    var textMuted by mutableStateOf(Color(0xFF968C7D))
    var brand by mutableStateOf(Color(0xFF6F8CFF))        // actions only
    var brandPress by mutableStateOf(Color(0xFF5A78F0))
    var brandSoft by mutableStateOf(Color(0x296F8CFF))    // rgba(111,140,255,.16)
    var onBrand by mutableStateOf(Color(0xFF0E1226))
    var income by mutableStateOf(Color(0xFF52C78D))       // money in only
    var incomeSoft by mutableStateOf(Color(0x2452C78D))
    var expense by mutableStateOf(Color(0xFFF26B4F))      // money out + destructive only
    var expenseSoft by mutableStateOf(Color(0x24F26B4F))
    var warn by mutableStateOf(Color(0xFFE8A33D))         // needs attention
    var warnSoft by mutableStateOf(Color(0x24E8A33D))

    internal fun apply(dark: Boolean) {
        if (dark) {
            bg = Color(0xFF12110F)
            surface = Color(0xFF1B1916)
            surface2 = Color(0xFF24211D)
            surface3 = Color(0xFF2E2A25)
            hairline = Color(0xFF34302A)
            rule = Color(0xFF4D463D)
            text = Color(0xFFF1ECE3)
            text2 = Color(0xFFB7AE9F)
            textMuted = Color(0xFF968C7D)
            brand = Color(0xFF6F8CFF)
            brandPress = Color(0xFF5A78F0)
            brandSoft = Color(0x296F8CFF)
            onBrand = Color(0xFF0E1226)
            income = Color(0xFF52C78D)
            incomeSoft = Color(0x2452C78D)
            expense = Color(0xFFF26B4F)
            expenseSoft = Color(0x24F26B4F)
            warn = Color(0xFFE8A33D)
            warnSoft = Color(0x24E8A33D)
        } else {
            bg = Color(0xFFFAF7F2)
            surface = Color(0xFFFFFFFF)
            surface2 = Color(0xFFF2EDE4)
            surface3 = Color(0xFFE6DFD3)
            hairline = Color(0xFFDDD5C7)
            rule = Color(0xFFBFB5A4)
            text = Color(0xFF1A1714)
            text2 = Color(0xFF5C544B)
            textMuted = Color(0xFF7E7468)
            brand = Color(0xFF4361EE)
            brandPress = Color(0xFF3451D1)
            brandSoft = Color(0x1F4361EE)
            onBrand = Color(0xFFFFFFFF)
            income = Color(0xFF1E8E5A)
            incomeSoft = Color(0x1F1E8E5A)
            expense = Color(0xFFD94426)
            expenseSoft = Color(0x1FD94426)
            warn = Color(0xFFC97A10)
            warnSoft = Color(0x1FC97A10)
        }
    }
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
val FigureStyle = TextStyle(
    fontFamily = SpaceMono,
    fontWeight = FontWeight.Bold,
    letterSpacing = (-0.02).sp,
    fontFeatureSettings = "tnum",
)

/** Dates and other short data labels. */
val MonoStyle = TextStyle(
    fontFamily = SpaceMono,
    fontWeight = FontWeight.Normal,
    fontSize = 12.sp,
    fontFeatureSettings = "tnum",
)

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
    KhataColors.apply(darkTheme)
    val scheme = if (darkTheme) {
        darkColorScheme(
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
    } else {
        lightColorScheme(
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
            scrim = Color(0x66000000),
        )
    }
    MaterialTheme(
        colorScheme = scheme,
        typography = KhataTypography,
        shapes = KhataShapes,
        content = content
    )
}
