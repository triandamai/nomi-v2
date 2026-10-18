package ai.nomi.designsystem.component

import ai.nomi.designsystem.theme.NomiTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onRoot
import io.github.takahirom.roborazzi.captureRoboImage
import org.junit.Rule
import kotlin.test.Test

private const val SETTLE_MS = 600L

@OptIn(ExperimentalTestApi::class)
class ComponentScreenshotTest {
    @get:Rule val rule = createComposeRule()

    /** [animated] sheets hold loaders that run forever, so their time is stepped by hand for a deterministic frame. */
    private fun capture(dark: Boolean, file: String, animated: Boolean = true, sheet: @Composable () -> Unit) {
        if (animated) rule.mainClock.autoAdvance = false
        rule.setContent { NomiTheme(accent = "green", dark = dark, reducedMotion = false) { sheet() } }
        if (animated) rule.mainClock.advanceTimeBy(SETTLE_MS) else rule.waitForIdle()
        rule.onRoot().captureRoboImage("src/desktopTest/snapshots/$file")
    }

    @Test fun actionsLight() = capture(false, "components_actions_light.png") { ActionsSheet() }
    @Test fun actionsDark() = capture(true, "components_actions_dark.png") { ActionsSheet() }
    @Test fun surfacesLight() = capture(false, "components_surfaces_light.png") { SurfacesSheet() }
    @Test fun composerLight() = capture(false, "components_composer_light.png") { ComposerSheet() }
    @Test fun composerDark() = capture(true, "components_composer_dark.png") { ComposerSheet() }
    @Test fun focusGradientLight() = capture(false, "components_focus_gradient_light.png", animated = false) { FocusSheet(FocusTarget.Gradient) }
    @Test fun focusGradientDark() = capture(true, "components_focus_gradient_dark.png", animated = false) { FocusSheet(FocusTarget.Gradient) }
    @Test fun focusChipLight() = capture(false, "components_focus_chip_light.png", animated = false) { FocusSheet(FocusTarget.Chip) }
    @Test fun focusSegmentLight() = capture(false, "components_focus_segment_light.png", animated = false) { FocusSheet(FocusTarget.Segment) }
    @Test fun surfacesDark() = capture(true, "components_surfaces_dark.png") { SurfacesSheet() }
}
