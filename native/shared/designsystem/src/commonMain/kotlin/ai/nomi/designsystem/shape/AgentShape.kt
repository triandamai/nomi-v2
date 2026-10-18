package ai.nomi.designsystem.shape

import ai.nomi.designsystem.generated.NomiShapeSpecs
import ai.nomi.designsystem.generated.NomiMotionTokens
import ai.nomi.designsystem.generated.GradientSpec
import ai.nomi.designsystem.theme.NomiTheme
import ai.nomi.designsystem.theme.toFittedBrush
import ai.nomi.designsystem.theme.toTween
import androidx.compose.animation.core.animateOffsetAsState
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.size
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableLongStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.withFrameMillis
import androidx.compose.animation.core.snap
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.Dp


private const val DEFAULT_TONE = "glow"
private const val START_MS = 0L

/**
 * An agent's shape filled with its tone gradient. While [working] it runs the identity's motion
 * (not under reduced motion); [face] adds Nomi's eyes, which glance toward [lookAt]
 * (normalised −1..1 around the shape's centre, clamped).
 */
@Composable
fun AgentShape(
    identity: AgentIdentity,
    size: Dp,
    working: Boolean = false,
    face: Boolean = false,
    lookAt: Offset? = null,
    modifier: Modifier = Modifier,
) {
    val animate = working && !NomiTheme.reducedMotion
    var elapsedMs by remember { mutableLongStateOf(START_MS) }
    if (animate) {
        LaunchedEffect(identity.motion) {
            val start = withFrameMillis { it }
            while (true) withFrameMillis { elapsedMs = it - start }
        }
    }
    val frame = if (animate) motionFrame(identity.motion, elapsedMs) else MotionFrame.Rest
    val look by animateOffsetAsState(
        targetValue = clampLook(lookAt),
        animationSpec = if (NomiTheme.reducedMotion) snap() else NomiMotionTokens.spatialDefault.toTween(),
        label = "nomi-look",
    )
    AgentShapeFrame(identity, size, frame, face, look, modifier)
}

/** The stateless drawing of an [AgentShape] at one [frame]; also what screenshot tests render at a fixed time. */
@Composable
internal fun AgentShapeFrame(
    identity: AgentIdentity,
    size: Dp,
    frame: MotionFrame,
    face: Boolean,
    look: Offset,
    modifier: Modifier = Modifier,
) {
    val spec = NomiShapeSpecs.all[identity.shape] ?: NomiShapeSpecs.all.getValue(NomiIdentity.shape)
    val shape = remember(spec) { PolarShape(spec) }
    val gradient: GradientSpec = NomiTheme.gradients.all[identity.tone] ?: NomiTheme.gradients.all.getValue(DEFAULT_TONE)
    val brush = remember(gradient) { gradient.toFittedBrush() }
    val eyes = Color(gradient.onColor)
    val sizePx = with(LocalDensity.current) { size.toPx() }
    Box(
        modifier
            .size(size)
            .graphicsLayer {
                rotationZ = frame.rotationDeg
                scaleX = frame.scaleX
                scaleY = frame.scaleY
                translationX = frame.translateXFraction * sizePx
                translationY = frame.translateYFraction * sizePx
            }
            .background(brush, shape)
            .drawWithContent {
                drawContent()
                if (face) drawNomiFace(eyes, look)
            },
    )
}
