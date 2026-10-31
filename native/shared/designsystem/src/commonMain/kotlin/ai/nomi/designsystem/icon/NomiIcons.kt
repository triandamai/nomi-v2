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
    val Wallet = glyph(
        "M19 7V5a1 1 0 0 0-1-1H5a2 2 0 0 0 0 4h14a1 1 0 0 1 1 1v3",
        "M3 6v12a2 2 0 0 0 2 2h14a1 1 0 0 0 1-1v-3",
        "M21 12h-4a2 2 0 0 0 0 4h4z",
    )
    val Link = glyph(
        "M10 14a4 4 0 0 0 5.66 0l3-3a4 4 0 0 0-5.66-5.66l-1 1",
        "M14 10a4 4 0 0 0-5.66 0l-3 3a4 4 0 0 0 5.66 5.66l1-1",
    )
    val Copy = glyph(
        "M9 10.5a1.5 1.5 0 0 1 1.5-1.5h8a1.5 1.5 0 0 1 1.5 1.5v8a1.5 1.5 0 0 1-1.5 1.5h-8a1.5 1.5 0 0 1-1.5-1.5z",
        "M5 15V6a2 2 0 0 1 2-2h9",
    )
    val Delete = glyph("M4 7h16M10 11v6M14 11v6M6 7l1 12a2 2 0 0 0 2 2h6a2 2 0 0 0 2-2l1-12M9 7V4h6v3")
    val ChevronDown = glyph("M6 9l6 6 6-6")
    val ChevronUp = glyph("M6 15l6-6 6 6")
    val Sparkle = glyph("M12 3l1.9 5.8a2 2 0 0 0 1.3 1.3L21 12l-5.8 1.9a2 2 0 0 0-1.3 1.3L12 21l-1.9-5.8a2 2 0 0 0-1.3-1.3L3 12l5.8-1.9a2 2 0 0 0 1.3-1.3z")
    val Clock = glyph("M3 12a9 9 0 1 0 18 0a9 9 0 1 0-18 0", "M12 7v5l3 3")
    val Folder = glyph("M3 6a2 2 0 0 1 2-2h4l2 3h8a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V6z")
    val Lock = glyph(rect(5f, 11f, 14f, 10f, 2f), "M8 11V8a4 4 0 0 1 8 0v3")
    val Pin = glyph("M9 4h6l-1 5 3 3v2H7v-2l3-3z", "M12 14v6")
    val DragHandle = glyph(
        line(8f, 6f, 16f, 6f),
        line(8f, 12f, 16f, 12f),
        line(8f, 18f, 16f, 18f),
    )
    val Card = glyph(rect(3f, 5f, 18f, 14f, 2f), "M3 10h18", "M7 15h4")

    // A project's tools (the web's project toolbar).
    val Bug = glyph("M8 9a4 4 0 0 1 8 0v5a4 4 0 0 1-8 0z", "M4 11h4M16 11h4M4 17h4M16 17h4M12 9v9", "M9 5L7.5 3.5M15 5l1.5-1.5")
    val Rocket = glyph("M9 15l-3-3 6-7c2-2 5-2 7-2 0 2 0 5-2 7l-7 6z", "M5 15c-1.5 1.5-2 4-2 4s2.5-.5 4-2", circle(15f, 9f, 1.5f))
    val Pulse = glyph("M3 12h4l3-8 4 16 3-8h4")
    val Database = glyph(
        "M4 6c0 1.7 3.6 3 8 3s8-1.3 8-3-3.6-3-8-3-8 1.3-8 3z",
        "M4 6v12c0 1.7 3.6 3 8 3s8-1.3 8-3V6",
        "M4 12c0 1.7 3.6 3 8 3s8-1.3 8-3",
    )
    val Globe = glyph(circle(12f, 12f, 9f), "M3 12h18", "M12 3c2.5 2.5 3.5 5.5 3.5 9s-1 6.5-3.5 9c-2.5-2.5-3.5-5.5-3.5-9s1-6.5 3.5-9z")

    // Attached files, one per kind (`AttachmentIcon.svelte`).
    val File = glyph("M14 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8z", "M14 3v5h5")
    val Pdf = glyph("M14 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8z", "M14 3v5h5", "M9 13h6M9 17h4")
    val Image = glyph(rect(3f, 3f, 18f, 18f, 3f), circle(9f, 9f, 2f), "m21 15-4.5-4.5L6 21")
    val Video = glyph(rect(2f, 5f, 14f, 14f, 3f), "m22 8-6 4 6 4z")
    val Audio = glyph("M9 18V5l12-2v13", circle(6f, 18f, 3f), circle(18f, 16f, 3f))
    val Voice = glyph(rect(9f, 3f, 6f, 11f, 3f), "M5 11a7 7 0 0 0 14 0M12 18v3")
    val Spreadsheet = glyph(rect(3f, 3f, 18f, 18f, 2f), "M3 9h18M3 15h18M9 3v18")
    val Presentation = glyph(rect(2f, 3f, 20f, 13f, 2f), "M12 16v5M8 21h8")
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
