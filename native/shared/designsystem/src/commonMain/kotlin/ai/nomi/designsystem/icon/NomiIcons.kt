package ai.nomi.designsystem.icon

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.layout.size
import androidx.compose.material3.LocalContentColor
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.PathEffect
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.StrokeJoin
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.drawscope.scale
import androidx.compose.ui.graphics.vector.PathParser
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

/** The web icons' 24-unit viewBox (frontend/src/lib/components/icons). */
private const val GRID = 24f
private const val DEFAULT_STROKE = 2f
val DefaultIconSize = 20.dp

/** One stroked outline of a glyph, in SVG path syntax on the 24-unit grid; [dash] is on/off lengths in grid units. */
class GlyphStroke(val d: String, val dash: FloatArray? = null)

/** A line icon: the web's inline SVG, stroked with round caps and joins. */
class NomiGlyph(val strokes: List<GlyphStroke>, val strokeWidth: Float = DEFAULT_STROKE)

private fun circle(cx: Float, cy: Float, r: Float) = "M${cx - r} ${cy}a$r $r 0 1 0 ${2 * r} 0a$r $r 0 1 0 ${-2 * r} 0"
private fun line(x1: Float, y1: Float, x2: Float, y2: Float) = "M$x1 ${y1}L$x2 $y2"
private fun polyline(vararg points: Float) =
    points.toList().chunked(2).mapIndexed { i, (x, y) -> (if (i == 0) "M" else "L") + "$x $y" }.joinToString("")
private fun rect(x: Float, y: Float, w: Float, h: Float, r: Float) =
    "M${x + r} ${y}h${w - 2 * r}a$r $r 0 0 1 $r ${r}v${h - 2 * r}a$r $r 0 0 1 ${-r} ${r}h${-(w - 2 * r)}a$r $r 0 0 1 ${-r} ${-r}v${-(h - 2 * r)}a$r $r 0 0 1 $r ${-r}z"

private fun glyph(vararg d: String) = NomiGlyph(d.map { GlyphStroke(it) })

/** The app's line icons, copied from the web's SVGs so both look the same. */
object NomiIcons {
    val Home = glyph("M3 11l9-7 9 7v9a1 1 0 0 1-1 1h-5v-6h-6v6H4a1 1 0 0 1-1-1z")
    val Chat = glyph("M4 5h16v11H9l-4 4V5z")
    val Bell = glyph("M6 8a6 6 0 0 1 12 0c0 7 3 9 3 9H3s3-2 3-9", "M10.3 21a1.94 1.94 0 0 0 3.4 0")
    val Inbox = glyph(
        "M22 12h-6l-2 3h-4l-2-3H2",
        "M5.45 5.11 2 12v6a2 2 0 0 0 2 2h16a2 2 0 0 0 2-2v-6l-3.45-6.89A2 2 0 0 0 16.76 4H7.24a2 2 0 0 0-1.79 1.11z",
    )
    val Settings = NomiGlyph(listOf(GlyphStroke(circle(12f, 12f, 3f)), GlyphStroke(circle(12f, 12f, 8f), dash = floatArrayOf(2f, 3.2f))))
    val Plus = glyph(line(12f, 5f, 12f, 19f), line(5f, 12f, 19f, 12f))
    val ChevronLeft = glyph(polyline(15f, 18f, 9f, 12f, 15f, 6f))
    val ChevronRight = glyph(polyline(9f, 18f, 15f, 12f, 9f, 6f))
    val ArrowBack = glyph("M19 12H5", "M11 6l-6 6 6 6")
    val ArrowForward = glyph("M5 12h14M13 6l6 6-6 6")
    val Person = glyph(circle(12f, 8f, 4f), "M4 20c0-4.4 3.6-8 8-8s8 3.6 8 8")
    val Logout = glyph("M15 4h3a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2h-3", line(10f, 12f, 3f, 12f), polyline(6.5f, 7.5f, 3f, 12f, 6.5f, 16.5f))
    val Close = glyph(line(6f, 6f, 18f, 18f), line(18f, 6f, 6f, 18f))
    val Menu = glyph(line(3f, 6f, 21f, 6f), line(3f, 12f, 21f, 12f), line(3f, 18f, 21f, 18f))
    val Apps = glyph(rect(4f, 4f, 6f, 6f, 1.5f), rect(14f, 4f, 6f, 6f, 1.5f), rect(4f, 14f, 6f, 6f, 1.5f), rect(14f, 14f, 6f, 6f, 1.5f))
    val Memory = glyph(circle(6f, 7f, 2.5f), circle(18f, 9f, 2.5f), circle(10f, 18f, 2.5f), "M8.3 8 15.6 8.6M7 9.3l2.2 6.4M16.4 11l-4.4 5.4")
    val Search = glyph(circle(10.5f, 10.5f, 6.5f), line(20f, 20f, 15.3f, 15.3f))
    val Check = glyph(polyline(5f, 12.5f, 10f, 17.5f, 19f, 6.5f))
    val Refresh = glyph("M20 11a8 8 0 1 0-2.3 5.7", polyline(20f, 4f, 20f, 11f, 13f, 11f))
    val Pencil = glyph("M12 20h9", "M16.5 3.5a2.1 2.1 0 0 1 3 3L7 19l-4 1 1-4Z")
}

/**
 * Draws [glyph] in [tint] (the surrounding content colour by default). Decorative unless [contentDescription] is
 * given; icon-only buttons pass their label there.
 */
@Composable
fun NomiIcon(
    glyph: NomiGlyph,
    contentDescription: String?,
    modifier: Modifier = Modifier,
    tint: Color = LocalContentColor.current,
    size: Dp = DefaultIconSize,
) {
    val paths = remember(glyph) { glyph.strokes.map { PathParser().parsePathString(it.d).toPath() to it.dash } }
    val described = if (contentDescription != null) Modifier.semantics { this.contentDescription = contentDescription } else Modifier
    Canvas(modifier.size(size).then(described)) {
        val unit = this.size.minDimension / GRID
        scale(unit, unit, pivot = Offset.Zero) {
            paths.forEach { (path, dash) ->
                drawPath(
                    path,
                    tint,
                    style = Stroke(
                        width = glyph.strokeWidth,
                        cap = StrokeCap.Round,
                        join = StrokeJoin.Round,
                        pathEffect = dash?.let { PathEffect.dashPathEffect(it) },
                    ),
                )
            }
        }
    }
}

/** Parses once per glyph; kept separate so a test can check every path string is valid. */
internal fun NomiGlyph.parsedPaths(): List<Path> = strokes.map { PathParser().parsePathString(it.d).toPath() }
