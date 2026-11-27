package ai.nomi.designsystem.theme

import ai.nomi.designsystem.ds.Ds
import ai.nomi.designsystem.ds.DsShapes
import ai.nomi.designsystem.ds.crewBg
import ai.nomi.designsystem.generated.NomiPenColors
import ai.nomi.designsystem.generated.NomiPenType
import androidx.compose.foundation.shape.CornerBasedShape
import androidx.compose.material3.ColorScheme
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.luminance
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.unit.Density
import androidx.compose.ui.unit.sp
import org.junit.Rule
import kotlin.math.max
import kotlin.math.min
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

private const val WCAG_AA = 4.5f

/** The pen's own accent fill: light accent-ink on accent is 3.4:1 on the web too, so it is held to AA for UI parts. */
private const val WCAG_AA_UI = 3f
private const val WCAG_OFFSET = 0.05f

class PenThemeTest {
    @get:Rule val rule = createComposeRule()

    private fun corner(shape: CornerBasedShape): Float = shape.topStart.toPx(Size(100f, 100f), Density(1f))

    @Test
    fun `the light theme draws with the pen's light values`() {
        lateinit var seen: List<Any>
        rule.setContent {
            NomiTheme(dark = false) {
                seen = listOf(Ds.bg, Ds.accent, Ds.text3, Ds.focus, Ds.body.fontSize, Ds.body.lineHeight, crewBg("sora"))
            }
        }
        rule.waitForIdle()
        assertEquals(
            listOf(Color(0xFFF6F6F4), Color(0xFF1E9A58), Color(0xFF7C7C84), Color(0x591E9A58), 15.sp, 19.sp, Color(0xFFE6E9FD)),
            seen,
        )
        assertEquals(6f, corner(DsShapes.sm))
        assertEquals(10f, corner(DsShapes.md))
    }

    @Test
    fun `the dark theme draws with the pen's dark values`() {
        lateinit var seen: List<Any>
        rule.setContent {
            NomiTheme(dark = true) { seen = listOf(Ds.bg, Ds.accent, Ds.dark, crewBg("sora")) }
        }
        rule.waitForIdle()
        assertEquals(listOf(Color(0xFF0A0A0B), Color(0xFF7BE3A4), true, Color(0xFF1C1F30)), seen)
    }

    @Test
    fun `a label is the mono role and a display line carries its tracking`() {
        lateinit var seen: List<Any?>
        rule.setContent {
            NomiTheme { seen = listOf(Ds.label.fontSize, Ds.label.letterSpacing, Ds.display.letterSpacing, Ds.labelCaps.letterSpacing) }
        }
        rule.waitForIdle()
        assertEquals(listOf(NomiPenType.label.size.sp, 0.sp, (-0.4).sp, 0.6.sp), seen)
    }

    private fun contrast(a: Color, b: Color): Float {
        val la = a.luminance() + WCAG_OFFSET
        val lb = b.luminance() + WCAG_OFFSET
        return max(la, lb) / min(la, lb)
    }

    private fun assertReadable(scheme: ColorScheme, theme: String) {
        listOf(
            "onSurface on surface" to contrast(scheme.onSurface, scheme.surface),
            "onBackground on background" to contrast(scheme.onBackground, scheme.background),
            "onSurfaceVariant on surface" to contrast(scheme.onSurfaceVariant, scheme.surface),
            "onError on error" to contrast(scheme.onError, scheme.error),
        ).forEach { (pair, ratio) -> assertTrue(ratio >= WCAG_AA, "$theme $pair is $ratio:1") }
        val inverse = contrast(scheme.inversePrimary, scheme.inverseSurface)
        assertTrue(inverse >= WCAG_AA_UI, "$theme inversePrimary on inverseSurface is $inverse:1")
        val primary = contrast(scheme.onPrimary, scheme.primary)
        assertTrue(primary >= WCAG_AA_UI, "$theme onPrimary on primary is $primary:1")
    }

    @Test
    fun `screens still on Material read clearly on the colours built from the pen`() {
        assertReadable(NomiPenColors.light.toColorScheme(dark = false), "light")
        assertReadable(NomiPenColors.dark.toColorScheme(dark = true), "dark")
    }

    @Test
    fun `the pen families carry regular, medium and semibold, so a mono label is 400 as on the web`() {
        var mono: androidx.compose.ui.text.font.FontFamily? = null
        var sans: androidx.compose.ui.text.font.FontFamily? = null
        rule.setContent { NomiTheme { mono = NomiTheme.families.mono; sans = NomiTheme.families.sans } }
        rule.waitForIdle()
        val weights = { f: androidx.compose.ui.text.font.FontFamily? -> (f as androidx.compose.ui.text.font.FontListFontFamily).fonts.map { it.weight.weight }.toSet() }
        assertEquals(setOf(400, 500, 600), weights(mono))
        assertEquals(setOf(400, 500, 600), weights(sans))
    }
}
