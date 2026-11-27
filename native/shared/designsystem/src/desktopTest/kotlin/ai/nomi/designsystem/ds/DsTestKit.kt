package ai.nomi.designsystem.ds

import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.toPixelMap
import androidx.compose.ui.test.SemanticsNodeInteraction
import androidx.compose.ui.test.captureToImage
import androidx.compose.ui.test.junit4.ComposeContentTestRule
import androidx.compose.ui.test.onRoot
import androidx.compose.ui.unit.Dp
import kotlin.math.abs
import kotlin.test.assertTrue

/** How far a drawn channel may be from the expected one (anti-aliasing, colour rounding): 3 of 255. */
private const val CHANNEL_TOLERANCE = 3f / 255f

/** The colour drawn at ([x], [y]) inside this node, measured from its top-left. */
fun SemanticsNodeInteraction.pixelAt(rule: ComposeContentTestRule, x: Dp, y: Dp): Color {
    val bounds = fetchSemanticsNode().boundsInRoot
    val map = rule.onRoot().captureToImage().toPixelMap()
    val px = with(rule.density) { (bounds.left + x.toPx()).toInt() to (bounds.top + y.toPx()).toInt() }
    return map[px.first, px.second]
}

/** The colour drawn at ([x], [y]) of this node's vertical middle. */
fun SemanticsNodeInteraction.pixelAtMiddle(rule: ComposeContentTestRule, x: Dp): Color {
    val height = with(rule.density) { fetchSemanticsNode().boundsInRoot.height.toDp() }
    return pixelAt(rule, x, height / 2)
}

/** The colour drawn just outside this node's leading edge, [outside] to its left, at its vertical middle. */
fun SemanticsNodeInteraction.pixelLeftOf(rule: ComposeContentTestRule, outside: Dp): Color = pixelAtMiddle(rule, -outside)

fun assertColor(expected: Color, actual: Color, what: String) {
    val close = abs(expected.red - actual.red) <= CHANNEL_TOLERANCE &&
        abs(expected.green - actual.green) <= CHANNEL_TOLERANCE &&
        abs(expected.blue - actual.blue) <= CHANNEL_TOLERANCE
    assertTrue(close, "$what: expected $expected, drew $actual")
}

/** The style a text node was laid out with. */
fun SemanticsNodeInteraction.textStyle(): androidx.compose.ui.text.TextStyle {
    val results = mutableListOf<androidx.compose.ui.text.TextLayoutResult>()
    fetchSemanticsNode().config[androidx.compose.ui.semantics.SemanticsActions.GetTextLayoutResult].action?.invoke(results)
    return results.single().layoutInput.style
}
