package ai.nomi.designsystem.component

import ai.nomi.designsystem.generated.NomiMotionTokens
import ai.nomi.designsystem.generated.NomiShapeSpecs
import ai.nomi.designsystem.shape.PolarShape
import ai.nomi.designsystem.theme.NomiTheme
import ai.nomi.designsystem.theme.toTween
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.snap
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsFocusedAsState
import androidx.compose.foundation.interaction.collectIsHoveredAsState
import androidx.compose.foundation.interaction.collectIsPressedAsState
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

/** What the send button shows and does. */
enum class SendButtonState { Disabled, Ready, Sending }

/** Sending wins over everything; otherwise the button is ready once there is something besides whitespace to send. */
fun sendButtonState(text: String, sending: Boolean): SendButtonState = when {
    sending -> SendButtonState.Sending
    text.isBlank() -> SendButtonState.Disabled
    else -> SendButtonState.Ready
}

val SendButtonSize: Dp = 56.dp
private val ArrowSize = 22.dp
private val FocusRingWidth = 2.dp
private val StopSize = 16.dp
private val LoaderInset = 8.dp
private const val HOVER_SCALE = 1.06f
private const val PRESS_SCALE = 0.92f
private const val HOVER_ROTATION_DEG = 40f
private const val DISABLED_ALPHA = 0.38f
private const val REST = 1f
private const val NO_ROTATION = 0f
private const val COOKIE = "cookie9"

/**
 * Nomi's send action: the brand cookie in the theme's action gradient with an arrow on top. While a turn is in
 * flight it becomes the morphing loader; give it [onStop] to make that state a Stop button with a square in its middle.
 */
@Composable
fun SendButton(
    state: SendButtonState,
    onSend: () -> Unit,
    modifier: Modifier = Modifier,
    onStop: (() -> Unit)? = null,
    sendLabel: String,
    sendingLabel: String = sendLabel,
    stopLabel: String = sendingLabel,
    size: Dp = SendButtonSize,
) {
    val stoppable = state == SendButtonState.Sending && onStop != null
    val enabled = state == SendButtonState.Ready || stoppable
    val interaction = remember { MutableInteractionSource() }
    val hovered by interaction.collectIsHoveredAsState()
    val pressed by interaction.collectIsPressedAsState()
    val focused by interaction.collectIsFocusedAsState()
    val spec = if (NomiTheme.reducedMotion) snap<Float>() else NomiMotionTokens.spatialFast.toTween()
    val scale by animateFloatAsState(
        when { !enabled -> REST; pressed -> PRESS_SCALE; hovered -> HOVER_SCALE; else -> REST }, spec, label = "send-scale",
    )
    val rotation by animateFloatAsState(
        if (enabled && hovered && state == SendButtonState.Ready) HOVER_ROTATION_DEG else NO_ROTATION,
        if (NomiTheme.reducedMotion) snap() else NomiMotionTokens.spatialSlow.toTween(), label = "send-rotation",
    )
    val shape = remember { PolarShape(NomiShapeSpecs.all.getValue(COOKIE)) }
    val brush = NomiTheme.extendedColors.actionBrush
    val label = when {
        stoppable -> stopLabel
        state == SendButtonState.Sending -> sendingLabel
        else -> sendLabel
    }
    Box(
        modifier
            .size(size)
            .graphicsLayer { scaleX = scale; scaleY = scale }
            .alpha(if (state == SendButtonState.Disabled) DISABLED_ALPHA else REST)
            .semantics { contentDescription = label }
            .then(if (focused) Modifier.border(FocusRingWidth, MaterialTheme.colorScheme.primary, CircleShape) else Modifier)
            .clip(CircleShape)
            .clickable(
                interactionSource = interaction,
                indication = null,
                enabled = enabled,
                role = Role.Button,
                onClick = if (stoppable) onStop!! else onSend,
            ),
        contentAlignment = Alignment.Center,
    ) {
        if (state == SendButtonState.Sending) {
            NomiLoadingIndicator(size = size - LoaderInset)
            if (stoppable) StopGlyph(MaterialTheme.colorScheme.onSurface, StopSize)
        } else {
            Box(Modifier.size(size).graphicsLayer { rotationZ = rotation }.background(brush, shape))
            ArrowUpGlyph(NomiTheme.extendedColors.onAction, ArrowSize)
        }
    }
}
