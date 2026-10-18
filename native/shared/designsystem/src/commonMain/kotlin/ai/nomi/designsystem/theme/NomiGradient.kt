package ai.nomi.designsystem.theme

import ai.nomi.designsystem.generated.GradientSpec
import ai.nomi.designsystem.generated.NomiGradients
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.LinearGradientShader
import androidx.compose.ui.graphics.Shader
import androidx.compose.ui.graphics.ShaderBrush
import kotlin.math.abs
import kotlin.math.cos
import kotlin.math.sin

private const val DEGREES_PER_HALF_TURN = 180.0
private const val HALF = 0.5f

/**
 * Start and end of a CSS `linear-gradient(<angleDeg>deg, ...)` line in a box of [size]:
 * 0deg points up, angles run clockwise, and the line is |w·sin a| + |h·cos a| long,
 * centred on the box, so the corners get exactly the first and last stop.
 */
fun gradientEndpoints(angleDeg: Float, size: Size): Pair<Offset, Offset> {
    val radians = angleDeg * (kotlin.math.PI / DEGREES_PER_HALF_TURN)
    val dx = sin(radians).toFloat()
    val dy = -cos(radians).toFloat()
    val length = abs(size.width * dx) + abs(size.height * dy)
    val center = Offset(size.width * HALF, size.height * HALF)
    val half = Offset(dx * length * HALF, dy * length * HALF)
    return (center - half) to (center + half)
}

/** A [Brush] of this gradient fitted to a box of [size]. */
fun GradientSpec.toBrush(size: Size): Brush {
    val (start, end) = gradientEndpoints(angleDeg, size)
    return Brush.linearGradient(
        colorStops = stops.map { (offset, argb) -> offset to Color(argb) }.toTypedArray(),
        start = start,
        end = end,
    )
}

/** A size-agnostic brush that re-fits the gradient to whatever it is drawn into. */
fun GradientSpec.toFittedBrush(): Brush = FittedGradientBrush(this)

private class FittedGradientBrush(private val spec: GradientSpec) : ShaderBrush() {
    private val colors = spec.stops.map { Color(it.second) }
    private val offsets = spec.stops.map { it.first }

    override fun createShader(size: Size): Shader {
        val (start, end) = gradientEndpoints(spec.angleDeg, size)
        return LinearGradientShader(start, end, colors, offsets)
    }

    override fun equals(other: Any?) = other is FittedGradientBrush && other.spec == spec
    override fun hashCode() = spec.hashCode()
}

/** The named accent-independent gradients ("tones") agents are painted with. */
class NomiGradientSet(val all: Map<String, GradientSpec>) {
    val glow get() = all.getValue("glow")
    val ember get() = all.getValue("ember")
    val tide get() = all.getValue("tide")
    val sky get() = all.getValue("sky")
    val bloom get() = all.getValue("bloom")
    val dusk get() = all.getValue("dusk")
    val citrus get() = all.getValue("citrus")
    val slate get() = all.getValue("slate")

    companion object {
        val Default = NomiGradientSet(NomiGradients.all)
    }
}
