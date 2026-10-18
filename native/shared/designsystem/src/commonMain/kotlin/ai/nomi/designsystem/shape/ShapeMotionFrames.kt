package ai.nomi.designsystem.shape

import ai.nomi.designsystem.generated.NomiMotionTokens
import ai.nomi.designsystem.theme.easing
import androidx.compose.animation.core.CubicBezierEasing
import androidx.compose.animation.core.Easing
import androidx.compose.animation.core.LinearEasing

/** One instant of a working motion: rotation, scale and translation (as a fraction of the shape's size). */
data class MotionFrame(
    val rotationDeg: Float = 0f,
    val scaleX: Float = 1f,
    val scaleY: Float = 1f,
    val translateXFraction: Float = 0f,
    val translateYFraction: Float = 0f,
) {
    companion object {
        val Rest = MotionFrame()
    }
}

// Durations mirror the keyframes in AgentShape.svelte.
internal const val SPIN_PERIOD_MS = 2600L
internal const val SPIN_BREATHE_PERIOD_MS = 1700L
internal const val WOBBLE_PERIOD_MS = 1100L
internal const val BOUNCE_PERIOD_MS = 900L
internal const val PULSE_PERIOD_MS = 1300L
internal const val ORBIT_SPIN_PERIOD_MS = 4200L
internal const val ORBIT_PERIOD_MS = 2100L

private const val FULL_TURN_DEG = 360f
private const val WOBBLE_DEG = 14f
private const val BREATHE_SCALE = 0.86f
private const val BOUNCE_RISE = -0.14f
private const val BOUNCE_UP_AT = 0.35f
private const val BOUNCE_UP_SCALE_X = 0.94f
private const val BOUNCE_UP_SCALE_Y = 1.06f
private const val BOUNCE_LAND_AT = 0.7f
private const val BOUNCE_LAND_SCALE_X = 1.08f
private const val BOUNCE_LAND_SCALE_Y = 0.9f
private const val PULSE_SQUEEZE_AT = 0.4f
private const val PULSE_SQUEEZE_SCALE = 0.78f
private const val PULSE_SWELL_AT = 0.6f
private const val PULSE_SWELL_SCALE = 1.06f
private const val ORBIT_RADIUS = 0.06f
private const val QUARTER = 0.25f
private const val HALF = 0.5f
private const val THREE_QUARTERS = 0.75f

// CSS ease-in-out, the orbit's translate timing.
private const val EASE_IN_OUT_X1 = 0.42f
private const val EASE_IN_OUT_Y1 = 0f
private const val EASE_IN_OUT_X2 = 0.58f
private const val EASE_IN_OUT_Y2 = 1f
private val EaseInOut: Easing = CubicBezierEasing(EASE_IN_OUT_X1, EASE_IN_OUT_Y1, EASE_IN_OUT_X2, EASE_IN_OUT_Y2)

/** (fraction of the period, value) stops; each segment eases with the easing of the stop it starts from. */
private class Keyframes(val stops: List<Pair<Float, Float>>, val easing: Easing) {
    fun at(fraction: Float): Float {
        val i = stops.indexOfLast { it.first <= fraction }.coerceIn(0, stops.size - 2)
        val (f0, v0) = stops[i]
        val (f1, v1) = stops[i + 1]
        val local = ((fraction - f0) / (f1 - f0)).coerceIn(0f, 1f)
        return v0 + (v1 - v0) * easing.transform(local)
    }
}

private fun fractionOf(elapsedMs: Long, periodMs: Long): Float = (elapsedMs % periodMs).toFloat() / periodMs

private val spring: Easing by lazy { NomiMotionTokens.spatialDefault.easing() }

private val breathe by lazy { Keyframes(listOf(0f to 1f, HALF to BREATHE_SCALE, 1f to 1f), spring) }
private val wobble by lazy { Keyframes(listOf(0f to -WOBBLE_DEG, HALF to WOBBLE_DEG, 1f to -WOBBLE_DEG), spring) }
private val bounceY by lazy { Keyframes(listOf(0f to 0f, BOUNCE_UP_AT to BOUNCE_RISE, BOUNCE_LAND_AT to 0f, 1f to 0f), spring) }
private val bounceSx by lazy { Keyframes(listOf(0f to 1f, BOUNCE_UP_AT to BOUNCE_UP_SCALE_X, BOUNCE_LAND_AT to BOUNCE_LAND_SCALE_X, 1f to 1f), spring) }
private val bounceSy by lazy { Keyframes(listOf(0f to 1f, BOUNCE_UP_AT to BOUNCE_UP_SCALE_Y, BOUNCE_LAND_AT to BOUNCE_LAND_SCALE_Y, 1f to 1f), spring) }
private val pulse by lazy {
    Keyframes(listOf(0f to 1f, PULSE_SQUEEZE_AT to PULSE_SQUEEZE_SCALE, PULSE_SWELL_AT to PULSE_SWELL_SCALE, 1f to 1f), spring)
}
private val orbitX by lazy {
    Keyframes(listOf(0f to ORBIT_RADIUS, QUARTER to 0f, HALF to -ORBIT_RADIUS, THREE_QUARTERS to 0f, 1f to ORBIT_RADIUS), EaseInOut)
}
private val orbitY by lazy {
    Keyframes(listOf(0f to 0f, QUARTER to ORBIT_RADIUS, HALF to 0f, THREE_QUARTERS to -ORBIT_RADIUS, 1f to 0f), EaseInOut)
}

private fun spinDegrees(elapsedMs: Long, periodMs: Long) = FULL_TURN_DEG * LinearEasing.transform(fractionOf(elapsedMs, periodMs))

/** The pose of [motion] [elapsedMs] after it started, the same keyframes as the web's AgentShape. */
fun motionFrame(motion: ShapeMotion, elapsedMs: Long): MotionFrame = when (motion) {
    ShapeMotion.Spin -> {
        val s = breathe.at(fractionOf(elapsedMs, SPIN_BREATHE_PERIOD_MS))
        MotionFrame(rotationDeg = spinDegrees(elapsedMs, SPIN_PERIOD_MS), scaleX = s, scaleY = s)
    }
    ShapeMotion.Wobble -> MotionFrame(rotationDeg = wobble.at(fractionOf(elapsedMs, WOBBLE_PERIOD_MS)))
    ShapeMotion.Bounce -> {
        val f = fractionOf(elapsedMs, BOUNCE_PERIOD_MS)
        MotionFrame(scaleX = bounceSx.at(f), scaleY = bounceSy.at(f), translateYFraction = bounceY.at(f))
    }
    ShapeMotion.Pulse -> {
        val s = pulse.at(fractionOf(elapsedMs, PULSE_PERIOD_MS))
        MotionFrame(scaleX = s, scaleY = s)
    }
    ShapeMotion.Orbit -> {
        val f = fractionOf(elapsedMs, ORBIT_PERIOD_MS)
        MotionFrame(
            rotationDeg = spinDegrees(elapsedMs, ORBIT_SPIN_PERIOD_MS),
            translateXFraction = orbitX.at(f),
            translateYFraction = orbitY.at(f),
        )
    }
}
