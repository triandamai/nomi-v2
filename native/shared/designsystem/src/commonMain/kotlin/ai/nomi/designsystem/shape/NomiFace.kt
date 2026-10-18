package ai.nomi.designsystem.shape

import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.drawscope.DrawScope

// Nomi's two-dot face in the 48-box (AgentShape.svelte).
private const val LEFT_EYE_X = 19f
private const val RIGHT_EYE_X = 29f
private const val EYE_Y = 22f
private const val EYE_RADIUS = 2.6f

/** How far the eyes slide toward the look target, in 48-box units, at a full (−1..1) look. */
const val EYE_MAX_OFFSET = 3.5f

private const val LOOK_MIN = -1f
private const val LOOK_MAX = 1f

/** [lookAt] clamped to −1..1 on both axes; null means looking straight ahead. */
fun clampLook(lookAt: Offset?): Offset =
    if (lookAt == null) Offset.Zero else Offset(lookAt.x.coerceIn(LOOK_MIN, LOOK_MAX), lookAt.y.coerceIn(LOOK_MIN, LOOK_MAX))

/** Draws the eyes in a box of the current draw size, shifted toward [look] (already clamped, −1..1). */
fun DrawScope.drawNomiFace(color: Color, look: Offset) {
    val scale = size.width / SHAPE_BOX
    val dx = look.x * EYE_MAX_OFFSET
    val dy = look.y * EYE_MAX_OFFSET
    listOf(LEFT_EYE_X, RIGHT_EYE_X).forEach { x ->
        drawCircle(color, radius = EYE_RADIUS * scale, center = Offset((x + dx) * scale, (EYE_Y + dy) * scale))
    }
}
