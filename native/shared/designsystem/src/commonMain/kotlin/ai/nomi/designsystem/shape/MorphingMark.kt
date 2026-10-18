package ai.nomi.designsystem.shape

import ai.nomi.designsystem.generated.NomiShapeSpecs
import ai.nomi.designsystem.theme.NomiTheme
import ai.nomi.designsystem.theme.toFittedBrush
import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.tween
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.size
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.unit.Dp
import kotlinx.coroutines.delay

private const val FULL_TURN_DEG = 360f
private const val MORPH_START = 0f
private const val MORPH_END = 1f

/**
 * Nomi's mark morphing through [sequence] (shape names), one step every [stepMs], and turning once every
 * [turnMs] (0 = never) — the web's `MorphingShape` with a sequence. Filled with the [tone] gradient; [face] adds
 * Nomi's eyes. Under reduced motion it holds the first shape still.
 */
@Composable
fun MorphingMark(
    size: Dp,
    sequence: List<String>,
    stepMs: Long,
    modifier: Modifier = Modifier,
    tone: String = NomiIdentity.tone,
    face: Boolean = false,
    turnMs: Int = 0,
) {
    val specs = remember(sequence) { sequence.mapNotNull { NomiShapeSpecs.all[it] } }
    val gradient = NomiTheme.gradients.all[tone] ?: NomiTheme.gradients.all.getValue(NomiIdentity.tone)
    val brush = remember(gradient) { gradient.toFittedBrush() }
    val eyes = Color(gradient.onColor)
    val still = NomiTheme.reducedMotion || specs.size < 2

    var index by remember { mutableIntStateOf(0) }
    val progress = remember { Animatable(MORPH_START) }
    val morphSpec = MaterialTheme.motionScheme.slowSpatialSpec<Float>()
    if (!still) {
        LaunchedEffect(specs) {
            while (true) {
                delay(stepMs)
                progress.snapTo(MORPH_START)
                progress.animateTo(MORPH_END, morphSpec)
                index = (index + 1) % specs.size
                progress.snapTo(MORPH_START)
            }
        }
    }
    val rotation = if (!still && turnMs > 0) {
        val turn = rememberInfiniteTransition(label = "mark-turn")
        turn.animateFloat(0f, FULL_TURN_DEG, infiniteRepeatable(tween(turnMs, easing = LinearEasing), RepeatMode.Restart), label = "turn").value
    } else {
        0f
    }

    val from = specs.getOrNull(index) ?: return
    val to = specs[(index + 1) % specs.size]
    Box(
        modifier
            .size(size)
            .graphicsLayer { rotationZ = rotation }
            .drawWithContent {
                drawContent()
                if (face) drawNomiFace(eyes, Offset.Zero)
            },
    ) {
        MorphingShape(from, to, if (still) MORPH_START else progress.value, brush, Modifier.size(size))
    }
}
