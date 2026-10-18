package ai.nomi.designsystem.component

import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.drawOutline
import androidx.compose.ui.graphics.drawscope.translate
import androidx.compose.ui.unit.dp

private val FocusRingWidth = 2.dp
private val FocusRingGap = 2.dp
private const val BOTH_SIDES = 2

/**
 * A primary-coloured ring drawn just outside [shape] while this element (or a child) has focus: the shared focus
 * indicator for Nomi components whose M3 base draws none that stays visible (transparent gradient container, chips,
 * group segments). The ring sits outside the container so it shows on a primary fill too, and takes no layout space.
 */
@Composable
fun Modifier.nomiFocusRing(shape: Shape): Modifier {
    var focused by remember { mutableStateOf(false) }
    val color = MaterialTheme.colorScheme.primary
    return onFocusChanged { focused = it.hasFocus }.drawWithContent {
        drawContent()
        if (focused) {
            val outset = (FocusRingGap + FocusRingWidth / BOTH_SIDES).toPx()
            val ringSize = Size(size.width + outset * BOTH_SIDES, size.height + outset * BOTH_SIDES)
            translate(-outset, -outset) {
                drawOutline(shape.createOutline(ringSize, layoutDirection, this), color, style = Stroke(FocusRingWidth.toPx()))
            }
        }
    }
}
