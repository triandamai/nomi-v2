package ai.nomi.app

import ai.nomi.app.di.initKoin
import ai.nomi.data.SdkSetup
import ai.nomi.network.ApiConfig
import ai.nomi.network.IosTokenStore
import ai.nomi.network.TokenStore
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.window.ComposeUIViewController
import org.koin.core.module.Module
import org.koin.dsl.module
import kotlinx.cinterop.ExperimentalForeignApi
import kotlin.experimental.ExperimentalNativeApi
import kotlin.native.setUnhandledExceptionHook
import kotlin.native.terminateWithUnhandledException
import platform.Foundation.NSApplicationSupportDirectory
import platform.Foundation.NSBundle
import platform.Foundation.NSFileManager
import platform.Foundation.NSLog
import platform.Foundation.NSUserDomainMask
import platform.UIKit.UIAccessibilityIsReduceMotionEnabled
import platform.UIKit.UIViewController

/** Info.plist key holding the backend URL (Debug: the local backend; Release: required at build time, project.yml). */
private const val API_URL_PLIST_KEY = "NomiApiUrl"

/** The Rust engine's data, in Application Support (backed up, not user-visible). */
private const val SDK_DIR = "nomi-sdk"

private var koinStarted = false

@Composable
actual fun rememberReducedMotion(): Boolean = remember { UIAccessibilityIsReduceMotionEnabled() }

/** Not wired to UIActivityViewController yet: a shared link is copied. */
@Composable
actual fun rememberShareText(): ((String) -> Unit)? = null

fun iosPlatformModule(): Module = module {
    val apiUrl = requireNotNull((NSBundle.mainBundle.objectForInfoDictionaryKey(API_URL_PLIST_KEY) as? String)?.takeIf { it.isNotBlank() }) {
        "Info.plist has no $API_URL_PLIST_KEY: set the NOMI_API_URL build setting"
    }
    single { ApiConfig(apiUrl) }
    single<TokenStore> { IosTokenStore() }
    single { SdkSetup(get<ApiConfig>().baseUrl, applicationSupportPath() + "/" + SDK_DIR) }
}

@OptIn(ExperimentalForeignApi::class)
private fun applicationSupportPath(): String = requireNotNull(
    NSFileManager.defaultManager.URLForDirectory(
        directory = NSApplicationSupportDirectory,
        inDomain = NSUserDomainMask,
        appropriateForURL = null,
        create = true,
        error = null,
    )?.path,
) { "Application Support directory unavailable" }

/**
 * A crash's cause goes to the system log before the app stops (Kotlin prints it only to stdout, which a release
 * build and a device away from Xcode don't keep): Xcode's console and Console.app show it.
 */
@OptIn(ExperimentalNativeApi::class)
private fun logCrashes() {
    setUnhandledExceptionHook { failure ->
        NSLog("Nomi crashed: %@", failure.stackTraceToString())
        terminateWithUnhandledException(failure)
    }
}

/** Entry point for the SwiftUI host: `MainViewControllerKt.MainViewController()`. Starts Koin on first call. */
fun MainViewController(): UIViewController {
    if (!koinStarted) {
        logCrashes()
        initKoin(iosPlatformModule())
        koinStarted = true
    }
    return ComposeUIViewController { NomiApp() }
}
