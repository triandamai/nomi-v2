package ai.nomi.desktop

import ai.nomi.app.NomiApp
import ai.nomi.app.desktopPlatformModule
import ai.nomi.app.di.initKoin
import ai.nomi.designsystem.component.ComposerPlatform
import ai.nomi.designsystem.component.LocalComposerPlatform
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.ui.window.Window
import androidx.compose.ui.window.application

private const val WINDOW_TITLE = "Nomi"

fun main() {
    initKoin(desktopPlatformModule())
    application {
        // Closing the window quits for now; staying in the background (tray, island) comes with the companion.
        Window(onCloseRequest = ::exitApplication, title = WINDOW_TITLE) {
            CompositionLocalProvider(LocalComposerPlatform provides ComposerPlatform.Desktop) {
                NomiApp()
            }
        }
    }
}
