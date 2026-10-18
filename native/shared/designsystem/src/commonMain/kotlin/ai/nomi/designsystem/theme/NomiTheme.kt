package ai.nomi.designsystem.theme

import ai.nomi.designsystem.generated.GradientSpec
import ai.nomi.designsystem.generated.NomiGradients
import ai.nomi.designsystem.generated.NomiPalettes
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

private val LocalReducedMotion = staticCompositionLocalOf { false }

private val LocalBubbleShapes = staticCompositionLocalOf<NomiBubbleShapes> {
    error("NomiTheme { } is missing from the composition")
}

object NomiTheme {
    val bubbleShapes: NomiBubbleShapes @Composable @ReadOnlyComposable get() = LocalBubbleShapes.current
    val gradients: NomiGradientSet @Composable @ReadOnlyComposable get() = LocalGradients.current
    val extendedColors: NomiExtendedColors @Composable @ReadOnlyComposable get() = LocalExtendedColors.current
    val metaStyle: TextStyle @Composable @ReadOnlyComposable get() = LocalMetaStyle.current

    /** True when the person asked for less motion: decorative loops (agent working motions) stay still. */
    val reducedMotion: Boolean @Composable @ReadOnlyComposable get() = LocalReducedMotion.current
}

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun NomiTheme(
    accent: String = NomiPalettes.DEFAULT_ACCENT,
    dark: Boolean = isSystemInDarkTheme(),
    reducedMotion: Boolean = false,
    content: @Composable () -> Unit,
) {
    val palette = NomiPalettes.all[accent] ?: NomiPalettes.all.getValue(NomiPalettes.DEFAULT_ACCENT)
    val roles = if (dark) palette.dark else palette.light
    val families = nomiFontFamilies()
    val typography = nomiTypography(families)
    val bubbles = remember { nomiBubbleShapes() }
    // --nomi-gradient-action shares the tone gradients' angle and stop positions; only its colours follow the accent.
    val actionStops = remember { NomiGradients.glow.stops.map { it.first } }
    val extended = remember(roles) {
        NomiExtendedColors(
            ground = Color(roles.ground),
            stage = Color(roles.stage),
            onStage = Color(roles.onStage),
            actionBrush = GradientSpec(
                angleDeg = NomiGradients.glow.angleDeg,
                stops = actionStops.zip(listOf(roles.action1, roles.action2, roles.action3)) { offset, argb -> offset to argb },
                onColor = roles.onAction,
            ).toFittedBrush(),
            onAction = Color(roles.onAction),
        )
    }
    val metaStyle = nomiMetaStyle(typography, families)
    CompositionLocalProvider(
        LocalExtendedColors provides extended,
        LocalMetaStyle provides metaStyle,
        LocalBubbleShapes provides bubbles,
        LocalReducedMotion provides reducedMotion,
    ) {
        MaterialExpressiveTheme(
            colorScheme = roles.toColorScheme(),
            motionScheme = if (reducedMotion) MotionScheme.standard() else MotionScheme.expressive(),
            typography = typography,
            shapes = nomiShapes(),
        ) {
            // Text and icons outside a Surface (pages on the ground, the sidebar) read in on-surface, in both themes.
            CompositionLocalProvider(LocalContentColor provides MaterialTheme.colorScheme.onSurface, content = content)
        }
    }
}

