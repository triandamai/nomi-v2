package ai.nomi.designsystem.theme

import ai.nomi.designsystem.generated.NomiPalettes
import ai.nomi.designsystem.generated.NomiPenColors
import ai.nomi.designsystem.generated.PenColors
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.ExperimentalMaterial3ExpressiveApi
import androidx.compose.material3.LocalContentColor
import androidx.compose.material3.MaterialExpressiveTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.MotionScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.Immutable
import androidx.compose.runtime.ReadOnlyComposable
import androidx.compose.runtime.remember
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.text.TextStyle

/** Nomi colours that have no Material 3 role: page ground, dark stage, and the accent "action" gradient. */
@Immutable
class NomiExtendedColors(
    val ground: Color,
    val stage: Color,
    val onStage: Color,
    val actionBrush: Brush,
    val onAction: Color,
)

private val LocalGradients = staticCompositionLocalOf { NomiGradientSet.Default }
private val LocalExtendedColors = staticCompositionLocalOf<NomiExtendedColors> {
    error("NomiTheme { } is missing from the composition")
}
private val LocalMetaStyle = staticCompositionLocalOf<TextStyle> {
    error("NomiTheme { } is missing from the composition")
}

private val LocalPenColors = staticCompositionLocalOf<PenColors> {
    error("NomiTheme { } is missing from the composition")
}
private val LocalPenDark = staticCompositionLocalOf { false }
private val LocalCoarsePointer = staticCompositionLocalOf { false }
private val LocalPenFamilies = staticCompositionLocalOf<PenFamilies> {
    error("NomiTheme { } is missing from the composition")
}

private val LocalReducedMotion = staticCompositionLocalOf { false }

private val LocalBubbleShapes = staticCompositionLocalOf<NomiBubbleShapes> {
    error("NomiTheme { } is missing from the composition")
}

object NomiTheme {
    /** The pen colours for the current scheme (frontend-new's tokens). */
    val pen: PenColors @Composable @ReadOnlyComposable get() = LocalPenColors.current

    /** True in the dark scheme. */
    val dark: Boolean @Composable @ReadOnlyComposable get() = LocalPenDark.current

    /** True where people point with a finger: targets grow to 44 (the web's `pointer-coarse`). */
    val coarsePointer: Boolean @Composable @ReadOnlyComposable get() = LocalCoarsePointer.current

    /** The pen's sans and mono families. */
    val families: PenFamilies @Composable @ReadOnlyComposable get() = LocalPenFamilies.current
    val bubbleShapes: NomiBubbleShapes @Composable @ReadOnlyComposable get() = LocalBubbleShapes.current
    val gradients: NomiGradientSet @Composable @ReadOnlyComposable get() = LocalGradients.current
    val extendedColors: NomiExtendedColors @Composable @ReadOnlyComposable get() = LocalExtendedColors.current
    val metaStyle: TextStyle @Composable @ReadOnlyComposable get() = LocalMetaStyle.current

    /** True when the person asked for less motion: decorative loops (agent working motions) stay still. */
    val reducedMotion: Boolean @Composable @ReadOnlyComposable get() = LocalReducedMotion.current
}

/**
 * Nomi's theme: frontend-new's pen colours, type and radii, light or dark. It still provides a Material theme built from
 * the pen colours, so screens not yet on the design system's components read in the same greys and green.
 *
 * [accent] is kept for callers from the Material look; the pen has one accent, so it no longer changes the colours.
 */
@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun NomiTheme(
    accent: String = NomiPalettes.DEFAULT_ACCENT,
    dark: Boolean = isSystemInDarkTheme(),
    reducedMotion: Boolean = false,
    coarsePointer: Boolean = platformCoarsePointer,
    content: @Composable () -> Unit,
) {
    val pen = if (dark) NomiPenColors.dark else NomiPenColors.light
    val families = nomiFontFamilies()
    val penFamilies = penFontFamilies()
    val typography = nomiTypography(families)
    val bubbles = remember { nomiBubbleShapes() }
    val extended = remember(pen) {
        NomiExtendedColors(
            ground = Color(pen.bg),
            stage = Color(pen.text),
            onStage = Color(pen.bg),
            actionBrush = SolidColor(Color(pen.accent)),
            onAction = Color(pen.accentInk),
        )
    }
    val colorScheme = remember(pen, dark) { pen.toColorScheme(dark) }
    val metaStyle = nomiMetaStyle(typography, families)
    CompositionLocalProvider(
        LocalPenColors provides pen,
        LocalPenDark provides dark,
        LocalCoarsePointer provides coarsePointer,
        LocalPenFamilies provides penFamilies,
        LocalExtendedColors provides extended,
        LocalMetaStyle provides metaStyle,
        LocalBubbleShapes provides bubbles,
        LocalReducedMotion provides reducedMotion,
    ) {
        MaterialExpressiveTheme(
            colorScheme = colorScheme,
            motionScheme = if (reducedMotion) MotionScheme.standard() else MotionScheme.expressive(),
            typography = typography,
            shapes = nomiShapes(),
        ) {
            // Text and icons outside a Surface (pages on the ground, the sidebar) read in the pen's text colour.
            CompositionLocalProvider(LocalContentColor provides Color(pen.text), content = content)
        }
    }
}

