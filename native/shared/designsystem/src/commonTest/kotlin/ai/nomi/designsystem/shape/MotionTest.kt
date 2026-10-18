package ai.nomi.designsystem.shape

import androidx.compose.ui.geometry.Offset
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

private const val TOLERANCE = 1e-3f
private const val FULL_TURN_DEG = 360f
private const val HALF_SPIN_DEG = 180f
private const val WOBBLE_EXTREME_DEG = 14f

class MotionTest {
    @Test fun everyMotionIsPeriodic() {
        mapOf(
            ShapeMotion.Spin to SPIN_PERIOD_MS * SPIN_BREATHE_PERIOD_MS,
            ShapeMotion.Wobble to WOBBLE_PERIOD_MS,
            ShapeMotion.Bounce to BOUNCE_PERIOD_MS,
            ShapeMotion.Pulse to PULSE_PERIOD_MS,
            ShapeMotion.Orbit to ORBIT_SPIN_PERIOD_MS,
        ).forEach { (m, period) ->
            val a = motionFrame(m, 0)
            val b = motionFrame(m, period)
            assertEquals(a.scaleX, b.scaleX, TOLERANCE, "$m scale")
            assertEquals(a.rotationDeg % FULL_TURN_DEG, b.rotationDeg % FULL_TURN_DEG, TOLERANCE, "$m rotation")
            assertEquals(a.translateXFraction, b.translateXFraction, TOLERANCE, "$m translate")
        }
    }

    @Test fun spinHalfwayIsHalfTurn() = assertEquals(HALF_SPIN_DEG, motionFrame(ShapeMotion.Spin, SPIN_PERIOD_MS / 2).rotationDeg, TOLERANCE)

    @Test fun wobbleReachesItsExtremes() {
        assertEquals(-WOBBLE_EXTREME_DEG, motionFrame(ShapeMotion.Wobble, 0).rotationDeg, TOLERANCE)
        assertEquals(WOBBLE_EXTREME_DEG, motionFrame(ShapeMotion.Wobble, WOBBLE_PERIOD_MS / 2).rotationDeg, TOLERANCE)
    }

    @Test fun orbitStartsOffsetRight() = assertTrue(motionFrame(ShapeMotion.Orbit, 0).translateXFraction > 0f)

    @Test fun lookIsClamped() {
        assertEquals(Offset(1f, -1f), clampLook(Offset(5f, -9f)))
        assertEquals(Offset.Zero, clampLook(null))
    }
}
