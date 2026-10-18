package ai.nomi.designsystem.theme

import ai.nomi.designsystem.generated.NomiPalettes
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import kotlin.math.roundToInt
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotEquals
import kotlin.test.assertTrue

class NomiColorsTest {
    @Test fun defaultLightPrimaryMatchesWeb() {
        val scheme = NomiPalettes.all.getValue("green").light.toColorScheme()
        assertEquals(Color(0xFF0B6B4A), scheme.primary)
    }
    @Test fun everyAccentHasLightAndDark() {
        NomiPalettes.all.forEach { (name, p) -> assertNotEquals(p.light.primary, p.dark.primary, name) }
    }
    @Test fun gradient135degRunsTopLeftToBottomRight() {
        val (start, end) = gradientEndpoints(angleDeg = 135f, size = Size(100f, 100f))
        assertTrue(start.x < end.x && start.y < end.y)
    }
    @Test fun gradient90degRunsLeftToRightAcrossMiddle() {
        val (start, end) = gradientEndpoints(angleDeg = 90f, size = Size(200f, 100f))
        assertEquals(Offset(0f, 50f), start.rounded())
        assertEquals(Offset(200f, 50f), end.rounded())
    }
    @Test fun gradient0degRunsBottomToTopThroughCentre() {
        val (start, end) = gradientEndpoints(angleDeg = 0f, size = Size(200f, 100f))
        assertEquals(Offset(100f, 100f), start.rounded())
        assertEquals(Offset(100f, 0f), end.rounded())
    }

    // sin/cos of the right angles leave ~1e-6 noise; snap it away before comparing.
    private fun Offset.rounded() = Offset(x.roundToInt().toFloat(), y.roundToInt().toFloat())
}
