package ai.nomi.app

import androidx.compose.ui.window.ComposeUIViewController
import platform.UIKit.UIViewController

/** Entry point for the SwiftUI host: `MainViewControllerKt.MainViewController()`. */
fun MainViewController(): UIViewController = ComposeUIViewController { NomiApp() }
