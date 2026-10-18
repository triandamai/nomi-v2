package ai.nomi.designsystem.component

import ai.nomi.designsystem.generated.NomiCorners
import ai.nomi.designsystem.theme.NomiTheme
import androidx.compose.animation.core.FiniteAnimationSpec
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.core.snap
import androidx.compose.foundation.background
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsPressedAsState
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonColors
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.ElevatedButton
import androidx.compose.material3.ExperimentalMaterial3ExpressiveApi
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.ProvideTextStyle
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

enum class NomiButtonVariant { Filled, Gradient, Tonal, Outlined, Text, Elevated }

/** Web sizes xs/s/m/l/xl on M3's extra-small..extra-large button heights. */
enum class NomiButtonSize(internal val pressedCorner: Float) {
    ExtraSmall(NomiCorners.small),
    Small(NomiCorners.small),
    Medium(NomiCorners.medium),
    Large(NomiCorners.extraLarge),
    ExtraLarge(NomiCorners.extraExtraLarge),
}

private const val DISABLED_ALPHA = 0.38f
private const val OPAQUE = 1f
private const val HEIGHT_TO_RADIUS_DIVISOR = 2

@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
private fun NomiButtonSize.height(): Dp = when (this) {
    NomiButtonSize.ExtraSmall -> ButtonDefaults.ExtraSmallContainerHeight
    NomiButtonSize.Small -> ButtonDefaults.MinHeight
    NomiButtonSize.Medium -> ButtonDefaults.MediumContainerHeight
    NomiButtonSize.Large -> ButtonDefaults.LargeContainerHeight
    NomiButtonSize.ExtraLarge -> ButtonDefaults.ExtraLargeContainerHeight
}

/**
 * M3 Expressive button: fully round at rest (half its height, a real length so the radius animates), corners tighten
 * on press. [NomiButtonVariant.Gradient] is the hero action in the theme's action gradient: use at most one per screen.
 */
@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun NomiButton(
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    variant: NomiButtonVariant = NomiButtonVariant.Filled,
    size: NomiButtonSize = NomiButtonSize.Small,
    enabled: Boolean = true,
    interactionSource: MutableInteractionSource? = null,
    content: @Composable RowScope.() -> Unit,
) {
    val source = interactionSource ?: remember { MutableInteractionSource() }
    val height = size.height()
    val rest = height / HEIGHT_TO_RADIUS_DIVISOR
    val pressedCorner = size.pressedCorner.dp
    val shapes = ButtonDefaults.shapes(RoundedCornerShape(rest), RoundedCornerShape(pressedCorner))
    val padding = ButtonDefaults.contentPaddingFor(height)
    val textStyle = ButtonDefaults.textStyleFor(height)
    val common = modifier
    if (variant == NomiButtonVariant.Gradient) {
        val pressed by source.collectIsPressedAsState()
        val spatialSpec: FiniteAnimationSpec<Dp> = MaterialTheme.motionScheme.fastSpatialSpec()
        val corner by animateDpAsState(
            if (pressed) pressedCorner else rest,
            if (NomiTheme.reducedMotion) snap() else spatialSpec,
            label = "gradient-button-corner",
        )
        // The gradient is a background behind a transparent M3 button, so it follows the press morph itself.
        Button(
            onClick = onClick,
            shapes = shapes,
            modifier = common
                .background(NomiTheme.extendedColors.actionBrush, RoundedCornerShape(corner))
                .nomiFocusRing(RoundedCornerShape(corner))
                .alpha(if (enabled) OPAQUE else DISABLED_ALPHA),
            enabled = enabled,
            colors = ButtonDefaults.buttonColors(
                containerColor = Color.Transparent,
                contentColor = NomiTheme.extendedColors.onAction,
                disabledContainerColor = Color.Transparent,
                disabledContentColor = NomiTheme.extendedColors.onAction,
            ),
            elevation = null,
            contentPadding = padding,
            interactionSource = source,
        ) { ProvideTextStyle(textStyle) { content() } }
        return
    }
    val colors: ButtonColors = when (variant) {
        NomiButtonVariant.Filled, NomiButtonVariant.Gradient -> ButtonDefaults.buttonColors()
        NomiButtonVariant.Tonal -> ButtonDefaults.filledTonalButtonColors()
        NomiButtonVariant.Outlined -> ButtonDefaults.outlinedButtonColors()
        NomiButtonVariant.Text -> ButtonDefaults.textButtonColors()
        NomiButtonVariant.Elevated -> ButtonDefaults.elevatedButtonColors()
    }
    val styled: @Composable RowScope.() -> Unit = { ProvideTextStyle(textStyle) { content() } }
    when (variant) {
        NomiButtonVariant.Tonal -> FilledTonalButton(onClick, shapes, common, enabled, colors, contentPadding = padding, interactionSource = source, content = styled)
        NomiButtonVariant.Outlined -> OutlinedButton(onClick, shapes, common, enabled, colors, contentPadding = padding, interactionSource = source, content = styled)
        NomiButtonVariant.Text -> TextButton(onClick, shapes, common, enabled, colors, contentPadding = padding, interactionSource = source, content = styled)
        NomiButtonVariant.Elevated -> ElevatedButton(onClick, shapes, common, enabled, colors, contentPadding = padding, interactionSource = source, content = styled)
        else -> Button(onClick, shapes, common, enabled, colors, contentPadding = padding, interactionSource = source, content = styled)
    }
}
