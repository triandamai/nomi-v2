package ai.nomi.app.screens

import ai.nomi.designsystem.theme.NomiTheme
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.size
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.test.junit4.ComposeContentTestRule
import androidx.compose.ui.test.onRoot
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import io.github.takahirom.roborazzi.captureRoboImage

/** A phone (the frontend rule: every screen checked at 390 wide) and a laptop window. */
enum class Device(val width: Dp, val height: Dp) {
    Phone(390.dp, 844.dp),
    Wide(1280.dp, 820.dp),
}

/** Time stepped before the capture, so springs settle and looping motion sits at a fixed frame. */
private const val SETTLE_MS = 900L

/** Renders [screen] at [device] size in light or [dark] theme and saves it under the app's snapshots. */
fun ComposeContentTestRule.captureScreen(name: String, device: Device, dark: Boolean, screen: @Composable () -> Unit) {
    mainClock.autoAdvance = false
    setContent {
        NomiTheme(accent = "green", dark = dark, reducedMotion = false) {
            Box(Modifier.size(device.width, device.height)) { screen() }
        }
    }
    mainClock.advanceTimeBy(SETTLE_MS)
    val theme = if (dark) "dark" else "light"
    onRoot().captureRoboImage("src/desktopTest/snapshots/${name}_${device.name.lowercase()}_$theme.png")
}
