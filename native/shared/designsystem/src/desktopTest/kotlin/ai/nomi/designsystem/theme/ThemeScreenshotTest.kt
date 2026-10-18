package ai.nomi.designsystem.theme

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onRoot
import io.github.takahirom.roborazzi.captureRoboImage
import org.junit.Rule
import kotlin.test.Test

@OptIn(ExperimentalTestApi::class)
class ThemeScreenshotTest {
    @get:Rule val rule = createComposeRule()

    private fun capture(dark: Boolean, file: String) {
        rule.setContent { NomiTheme(accent = "green", dark = dark) { ThemePreview() } }
        rule.waitForIdle()
        rule.onRoot().captureRoboImage("src/desktopTest/snapshots/$file")
    }

    @Test fun greenLight() = capture(dark = false, file = "theme_light.png")
    @Test fun greenDark() = capture(dark = true, file = "theme_dark.png")
}
