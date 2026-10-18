package ai.nomi.designsystem.shape

import ai.nomi.designsystem.generated.PolarSpec
import ai.nomi.designsystem.generated.SHAPE_CENTER
import ai.nomi.designsystem.generated.SHAPE_POINTS
import ai.nomi.designsystem.generated.SHAPE_RADIUS
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Outline
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.unit.Density
import androidx.compose.ui.unit.LayoutDirection
import kotlin.math.PI
import kotlin.math.cos
import kotlin.math.sin

/** Side of the square box every outline is drawn in (the web's 48×48 viewBox). */
const val SHAPE_BOX = SHAPE_CENTER * 2f

private const val FLOATS_PER_POINT = 2
private const val FULL_TURN = 2.0 * PI

/**
 * Interleaved x,y of the outline in the 48×48 box, exactly the web's `shapePoints`
 * (frontend/src/lib/components/m3/shapes.ts): r(θ) = R·(1 + a·cos(lobes·(θ − phase))).
 */
fun polarPoints(spec: PolarSpec, points: Int = SHAPE_POINTS): FloatArray {
    val out = FloatArray(points * FLOATS_PER_POINT)
    for (i in 0 until points) {
        val theta = FULL_TURN * i / points
        val r = SHAPE_RADIUS * (1.0 + spec.amplitude * cos(spec.lobes * (theta - spec.phase)))
        out[i * FLOATS_PER_POINT] = (SHAPE_CENTER + r * cos(theta)).toFloat()
        out[i * FLOATS_PER_POINT + 1] = (SHAPE_CENTER + r * sin(theta)).toFloat()
    }
    return out
}

/** Per-point linear interpolation; both arrays must hold the same number of points. */
fun lerpPoints(a: FloatArray, b: FloatArray, t: Float): FloatArray {
    require(a.size == b.size) { "outlines differ in point count: ${a.size} vs ${b.size}" }
    return FloatArray(a.size) { a[it] + (b[it] - a[it]) * t }
}

/** A closed path through [points], scaled from the 48-box into [size]. */
fun pointsToPath(points: FloatArray, size: Size): Path {
    val sx = size.width / SHAPE_BOX
    val sy = size.height / SHAPE_BOX
    return Path().apply {
        for (i in points.indices step FLOATS_PER_POINT) {
            val x = points[i] * sx
            val y = points[i + 1] * sy
            if (i == 0) moveTo(x, y) else lineTo(x, y)
        }
        close()
    }
}

/** A Nomi agent shape as a Compose [Shape]: the 48-box outline scaled into whatever size it clips or fills. */
class PolarShape(val spec: PolarSpec) : Shape {
    private val points = polarPoints(spec)

    override fun createOutline(size: Size, layoutDirection: LayoutDirection, density: Density): Outline =
        Outline.Generic(pointsToPath(points, size))

    override fun equals(other: Any?) = other is PolarShape && other.spec == spec
    override fun hashCode() = spec.hashCode()
}

/** A shape part-way between two outlines; [progress] 0 is [from], 1 is [to]. */
internal class MorphShape(private val from: FloatArray, private val to: FloatArray, private val progress: Float) : Shape {
    override fun createOutline(size: Size, layoutDirection: LayoutDirection, density: Density): Outline =
        Outline.Generic(pointsToPath(lerpPoints(from, to, progress), size))

    override fun equals(other: Any?) =
        other is MorphShape && other.progress == progress && other.from.contentEquals(from) && other.to.contentEquals(to)

    override fun hashCode() = progress.hashCode() * HASH_MIX + from.contentHashCode() + to.contentHashCode()

    private companion object {
        const val HASH_MIX = 31
    }
}
