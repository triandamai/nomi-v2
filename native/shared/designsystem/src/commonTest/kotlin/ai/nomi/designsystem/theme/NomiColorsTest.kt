package ai.nomi.designsystem.theme

import ai.nomi.designsystem.generated.NomiPalettes
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
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
}
