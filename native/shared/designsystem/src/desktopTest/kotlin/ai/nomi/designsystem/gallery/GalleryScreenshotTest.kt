package ai.nomi.designsystem.gallery

import ai.nomi.designsystem.ds.DsToasts
import ai.nomi.designsystem.theme.NomiTheme
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.onRoot
import androidx.compose.ui.test.runDesktopComposeUiTest
import androidx.compose.ui.unit.dp
import io.github.takahirom.roborazzi.captureRoboImage
import kotlin.test.Test
import kotlin.test.assertTrue

private const val SETTLE_MS = 800L
private const val WINDOW_WIDTH = 390
private const val WINDOW_HEIGHT = 3200
private val PhoneWidth = WINDOW_WIDTH.dp

/**
 * Every section of the three design sheets, drawn at phone width in both themes, one image each, so a change to any
 * component shows up against its web twin. Loaders run forever, so time is stepped by hand.
 */
@OptIn(ExperimentalTestApi::class)
class GalleryScreenshotTest {
    private fun capture(sheet: GallerySheet, dark: Boolean) = runDesktopComposeUiTest(WINDOW_WIDTH, WINDOW_HEIGHT) {
        mainClock.autoAdvance = false
        var at by mutableIntStateOf(0)
        val toasts = DsToasts()
        val sections = gallerySections(sheet, toasts)
        setContent {
            NomiTheme(accent = "green", dark = dark, reducedMotion = false) {
                val current = remember(at) { sections[at] }
                Box(Modifier.width(PhoneWidth).background(NomiTheme.extendedColors.ground).padding(16.dp)) { GallerySectionView(current) }
            }
        }
        sections.forEachIndexed { index, section ->
            at = index
            mainClock.advanceTimeBy(SETTLE_MS)
            val name = section.label.lowercase().replace(' ', '_')
            onRoot().captureRoboImage("src/desktopTest/snapshots/ds_${sheet.name.lowercase()}_${name}_${if (dark) "dark" else "light"}.png")
        }
    }

    @Test fun primitivesLight() = capture(GallerySheet.Primitives, dark = false)
    @Test fun primitivesDark() = capture(GallerySheet.Primitives, dark = true)
    @Test fun chatLight() = capture(GallerySheet.Chat, dark = false)
    @Test fun chatDark() = capture(GallerySheet.Chat, dark = true)
    @Test fun buildLight() = capture(GallerySheet.Build, dark = false)
    @Test fun buildDark() = capture(GallerySheet.Build, dark = true)

    @Test
    fun everySheetHasItsSections() {
        val toasts = DsToasts()
        val labels = GallerySheet.entries.associateWith { sheet -> gallerySections(sheet, toasts).map { it.label } }
        assertTrue(labels.getValue(GallerySheet.Primitives).containsAll(listOf("Actions", "Crew", "Status", "Inputs", "Overlays", "App chrome", "Lists", "Conversation")))
        assertTrue(labels.getValue(GallerySheet.Chat).containsAll(listOf("Thinking", "Rich", "Actions", "Feedback", "States", "Controls", "Threads", "Crew", "Mobile")))
        assertTrue(labels.getValue(GallerySheet.Build).containsAll(listOf("Estimate", "Building", "Share", "First run", "Workspace", "Code", "Together", "Mobile build")))
    }
}
