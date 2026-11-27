package ai.nomi.designsystem.theme

import ai.nomi.designsystem.generated.PenColors
import androidx.compose.material3.ColorScheme
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.compositeOver

/**
 * A Material 3 [ColorScheme] built from the pen colours, so screens and widgets still drawn with Material read in the
 * pen's greys and green until they move onto the design system's own components.
 */
fun PenColors.toColorScheme(dark: Boolean): ColorScheme = ColorScheme(
    primary = Color(accent),
    onPrimary = Color(accentInk),
    primaryContainer = Color(accentBg).compositeOn(surface),
    onPrimaryContainer = Color(text),
    inversePrimary = Color(accent),
    secondary = Color(text2),
    onSecondary = Color(bg),
    secondaryContainer = Color(surface2),
    onSecondaryContainer = Color(text),
    tertiary = Color(warning),
    onTertiary = Color(bg),
    tertiaryContainer = Color(warningBg).compositeOn(surface),
    onTertiaryContainer = Color(text),
    background = Color(bg),
    onBackground = Color(text),
    surface = Color(surface),
    onSurface = Color(text),
    surfaceVariant = Color(surface2),
    onSurfaceVariant = Color(text2),
    surfaceTint = Color.Transparent,
    inverseSurface = Color(text),
    inverseOnSurface = Color(bg),
    error = Color(danger),
    onError = if (dark) Color(bg) else Color(accentInk),
    errorContainer = Color(dangerBg).compositeOn(surface),
    onErrorContainer = Color(danger),
    outline = Color(borderStrong),
    outlineVariant = Color(border),
    scrim = Color(overlay),
    surfaceBright = Color(surface),
    surfaceDim = Color(bg),
    surfaceContainer = Color(surface2),
    surfaceContainerHigh = Color(surface2),
    surfaceContainerHighest = Color(border),
    surfaceContainerLow = Color(surface),
    surfaceContainerLowest = Color(surface),
)

/** A translucent pen colour flattened onto an opaque one, for Material roles that expect opaque fills. */
private fun Color.compositeOn(ground: Long): Color = compositeOver(Color(ground))
