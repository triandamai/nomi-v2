package ai.nomi.designsystem.component

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.layout.size
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.CornerRadius
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.StrokeJoin
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.unit.Dp

/** Side of the 24-unit grid the web's inline SVG glyphs are drawn on. */
private const val GRID = 24f

private const val ARROW_TIP_Y = 5f
private const val ARROW_TAIL_Y = 19f
private const val ARROW_MID = 12f
private const val ARROW_WING_LEFT = 5f
private const val ARROW_WING_RIGHT = 19f
private const val ARROW_STROKE = 2.4f

private const val STOP_INSET = 5f
private const val STOP_SIDE = 14f
private const val STOP_RADIUS = 3.5f

private const val CHECK_STROKE = 2.6f
private const val CHECK_X1 = 5f
private const val CHECK_Y1 = 12f
private const val CHECK_X2 = 10f
private const val CHECK_Y2 = 17f
private const val CHECK_X3 = 19f
private const val CHECK_Y3 = 7f

private const val CHEVRON_STROKE = 2.2f
private const val CHEVRON_X1 = 6f
private const val CHEVRON_X2 = 12f
private const val CHEVRON_X3 = 18f
private const val CHEVRON_Y_TOP = 9f
private const val CHEVRON_Y_BOTTOM = 15f

private fun DrawScope.strokeOf(width: Float) =
    Stroke(width = width * size.minDimension / GRID, cap = StrokeCap.Round, join = StrokeJoin.Round)

private fun DrawScope.at(x: Float, y: Float) = Offset(x * size.width / GRID, y * size.height / GRID)

private fun Path.moveTo(o: Offset) = moveTo(o.x, o.y)
private fun Path.lineTo(o: Offset) = lineTo(o.x, o.y)

/** The send arrow (the web SendButton's `M12 19V5M5 12l7-7 7 7`). */
@Composable
internal fun ArrowUpGlyph(color: Color, size: Dp, modifier: Modifier = Modifier) {
    Canvas(modifier.size(size)) {
        val path = Path().apply {
            moveTo(at(ARROW_MID, ARROW_TAIL_Y)); lineTo(at(ARROW_MID, ARROW_TIP_Y))
            moveTo(at(ARROW_WING_LEFT, ARROW_MID)); lineTo(at(ARROW_MID, ARROW_TIP_Y)); lineTo(at(ARROW_WING_RIGHT, ARROW_MID))
        }
        drawPath(path, color, style = strokeOf(ARROW_STROKE))
    }
}

/** The rounded stop square shown inside the in-flight send button. */
@Composable
internal fun StopGlyph(color: Color, size: Dp, modifier: Modifier = Modifier) {
    Canvas(modifier.size(size)) {
        val unit = this.size.minDimension / GRID
        drawRoundRect(
            color,
            topLeft = at(STOP_INSET, STOP_INSET),
            size = Size(STOP_SIDE * unit, STOP_SIDE * unit),
            cornerRadius = CornerRadius(STOP_RADIUS * unit),
        )
    }
}

/** The filter chip's selected check. */
@Composable
internal fun CheckGlyph(color: Color, size: Dp, modifier: Modifier = Modifier) {
    Canvas(modifier.size(size)) {
        val path = Path().apply {
            moveTo(at(CHECK_X1, CHECK_Y1)); lineTo(at(CHECK_X2, CHECK_Y2)); lineTo(at(CHECK_X3, CHECK_Y3))
        }
        drawPath(path, color, style = strokeOf(CHECK_STROKE))
    }
}

/** The split button's menu chevron. */
@Composable
internal fun ChevronDownGlyph(color: Color, size: Dp, modifier: Modifier = Modifier) {
    Canvas(modifier.size(size)) {
        val path = Path().apply {
            moveTo(at(CHEVRON_X1, CHEVRON_Y_TOP)); lineTo(at(CHEVRON_X2, CHEVRON_Y_BOTTOM)); lineTo(at(CHEVRON_X3, CHEVRON_Y_TOP))
        }
        drawPath(path, color, style = strokeOf(CHEVRON_STROKE))
    }
}
