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

    private fun capture(dark: Boolean, file: String, sheet: @Composable () -> Unit) {
        // The loaders animate forever, so time is stepped by hand for a deterministic frame.
        rule.mainClock.autoAdvance = false
        rule.setContent { NomiTheme(accent = "green", dark = dark, reducedMotion = false) { sheet() } }
        rule.mainClock.advanceTimeBy(SETTLE_MS)
        rule.onRoot().captureRoboImage("src/desktopTest/snapshots/$file")
    }

    @Test fun actionsLight() = capture(false, "components_actions_light.png") { ActionsSheet() }
    @Test fun actionsDark() = capture(true, "components_actions_dark.png") { ActionsSheet() }
    @Test fun surfacesLight() = capture(false, "components_surfaces_light.png") { SurfacesSheet() }
    @Test fun composerLight() = capture(false, "components_composer_light.png") { ComposerSheet() }
    @Test fun composerDark() = capture(true, "components_composer_dark.png") { ComposerSheet() }
    @Test fun surfacesDark() = capture(true, "components_surfaces_dark.png") { SurfacesSheet() }
}
