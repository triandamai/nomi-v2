package ai.nomi.desktop

import ai.nomi.app.NomiApp
import androidx.compose.ui.window.Window
import androidx.compose.ui.window.application

private const val WINDOW_TITLE = "Nomi"

fun main() = application {
    Window(onCloseRequest = ::exitApplication, title = WINDOW_TITLE) {
        NomiApp()
    }
}
