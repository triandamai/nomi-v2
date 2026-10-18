package ai.nomi.desktop

import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.unit.IntRect
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import java.awt.MouseInfo
import kotlin.math.hypot
import kotlin.math.max

const val CURSOR_POLL_NEAR_MS = 16L
const val CURSOR_POLL_FAR_MS = 100L
const val NEAR_DISTANCE_DP = 160f
private const val HALF_F = 2f
private const val LOOK_RANGE_MIN = -1f
private const val LOOK_RANGE_MAX = 1f

data class CursorSample(
    val xDp: Int,
    val yDp: Int,
    val insideIsland: Boolean,
    val distanceDp: Float,
    val lookAt: Offset?,
)

/**
 * Pure calculation of [CursorSample] from a cursor point `(cursorXDp, cursorYDp)` and [pillBounds].
 * When the cursor is within [NEAR_DISTANCE_DP] of the island, [CursorSample.lookAt] is a normalized `-1..1` vector
 * so Nomi's eyes glance toward the pointer.
 */
fun evaluateCursorSample(cursorXDp: Int, cursorYDp: Int, pillBounds: IntRect): CursorSample {
    val inside = cursorXDp in pillBounds.left..pillBounds.right && cursorYDp in pillBounds.top..pillBounds.bottom
    val nearestX = cursorXDp.coerceIn(pillBounds.left, pillBounds.right)
    val nearestY = cursorYDp.coerceIn(pillBounds.top, pillBounds.bottom)
    val dxRect = (cursorXDp - nearestX).toFloat()
    val dyRect = (cursorYDp - nearestY).toFloat()
    val distance = hypot(dxRect, dyRect)

    val centerX = (pillBounds.left + pillBounds.right) / HALF_F
    val centerY = (pillBounds.top + pillBounds.bottom) / HALF_F
    val look = if (distance <= NEAR_DISTANCE_DP) {
        val normRadius = max(NEAR_DISTANCE_DP, pillBounds.width / HALF_F)
        Offset(
            x = ((cursorXDp - centerX) / normRadius).coerceIn(LOOK_RANGE_MIN, LOOK_RANGE_MAX),
            y = ((cursorYDp - centerY) / normRadius).coerceIn(LOOK_RANGE_MIN, LOOK_RANGE_MAX),
        )
    } else {
        null
    }

    return CursorSample(
        xDp = cursorXDp,
        yDp = cursorYDp,
        insideIsland = inside,
        distanceDp = distance,
        lookAt = look,
    )
}

/**
 * Polls `MouseInfo.getPointerInfo()` at 60fps when the cursor is near the Dynamic Island and 10fps when far.
 */
class CursorTracker(
    private val scope: CoroutineScope,
    private val pillBoundsProvider: () -> IntRect,
    private val pointerProvider: () -> Pair<Int, Int>? = {
        runCatching { MouseInfo.getPointerInfo()?.location?.let { it.x to it.y } }.getOrNull()
    },
) {
    private val mutableSample = MutableStateFlow<CursorSample?>(null)
    val sample: StateFlow<CursorSample?> = mutableSample.asStateFlow()

    private var pollJob: Job? = null

    fun start() {
        if (pollJob != null) return
        pollJob = scope.launch {
            while (isActive) {
                val point = pointerProvider()
                if (point == null) {
                    mutableSample.value = null
                    delay(CURSOR_POLL_FAR_MS)
                } else {
                    val evaluated = evaluateCursorSample(point.first, point.second, pillBoundsProvider())
                    mutableSample.value = evaluated
                    val interval = if (evaluated.distanceDp <= NEAR_DISTANCE_DP) CURSOR_POLL_NEAR_MS else CURSOR_POLL_FAR_MS
                    delay(interval)
                }
            }
        }
    }

    fun stop() {
        pollJob?.cancel()
        pollJob = null
        mutableSample.value = null
    }
}
