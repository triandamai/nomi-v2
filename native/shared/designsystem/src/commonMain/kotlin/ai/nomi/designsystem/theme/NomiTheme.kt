package ai.nomi.designsystem.theme

import ai.nomi.designsystem.generated.NomiPalettes
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.ExperimentalMaterial3ExpressiveApi
import androidx.compose.material3.MaterialExpressiveTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.MotionScheme
import androidx.compose.material3.Shapes
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

object NomiTheme {
    val gradients: NomiGradientSet @Composable @ReadOnlyComposable get() = LocalGradients.current
    val extendedColors: NomiExtendedColors @Composable @ReadOnlyComposable get() = LocalExtendedColors.current
    val metaStyle: TextStyle @Composable @ReadOnlyComposable get() = LocalMetaStyle.current
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
    val typography = nomiTypography()
    val extended = remember(roles) {
        NomiExtendedColors(
            ground = Color(roles.ground),
            stage = Color(roles.stage),
            onStage = Color(roles.onStage),
            actionBrush = ai.nomi.designsystem.generated.GradientSpec(
                angleDeg = ACTION_GRADIENT_ANGLE_DEG,
                stops = listOf(
                    ACTION_STOP_START to roles.action1,
                    ACTION_STOP_MID to roles.action2,
                    ACTION_STOP_END to roles.action3,
                ),
                onColor = roles.onAction,
            ).toFittedBrush(),
            onAction = Color(roles.onAction),
        )
    }
    val metaStyle = nomiMetaStyle(typography)
    CompositionLocalProvider(
        LocalExtendedColors provides extended,
        LocalMetaStyle provides metaStyle,
    ) {
        MaterialExpressiveTheme(
            colorScheme = roles.toColorScheme(),
            motionScheme = if (reducedMotion) MotionScheme.standard() else MotionScheme.expressive(),
            typography = typography,
            shapes = Shapes(),
            content = content,
        )
    }
}

// Same stops as the web's --nomi-gradient-action.
private const val ACTION_GRADIENT_ANGLE_DEG = 135f
private const val ACTION_STOP_START = 0f
private const val ACTION_STOP_MID = 0.55f
private const val ACTION_STOP_END = 1f
