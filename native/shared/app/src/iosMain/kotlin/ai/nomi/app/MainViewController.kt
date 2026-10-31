package ai.nomi.app

import ai.nomi.app.di.initKoin
import ai.nomi.data.SdkSetup
import ai.nomi.network.ApiConfig
import ai.nomi.network.IosTokenStore
import ai.nomi.network.TokenStore
import ai.nomi.sdk.DeviceInfo
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
import platform.UIKit.UIDevice
import kotlinx.cinterop.alloc
import kotlinx.cinterop.memScoped
import kotlinx.cinterop.ptr
import kotlinx.cinterop.toKString
import platform.posix.uname
import platform.posix.utsname
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
    single { SdkSetup(get<ApiConfig>().baseUrl, applicationSupportPath() + "/" + SDK_DIR, iosDevice()) }
}

private const val PLATFORM_IOS = "ios"
private const val BRAND_APPLE = "Apple"
private const val VERSION_PLIST_KEY = "CFBundleShortVersionString"
private const val UNKNOWN_VERSION = "unknown"

/** This iPhone or iPad, as the person's device list shows it: "iPhone", the model id (iPhone16,1), iOS and its version. */
private fun iosDevice(): DeviceInfo {
    val device = UIDevice.currentDevice
    return DeviceInfo(
        platform = PLATFORM_IOS,
        name = device.model,
        brand = BRAND_APPLE,
        model = machineId() ?: device.model,
        osName = device.systemName,
        osVersion = device.systemVersion,
        appVersion = NSBundle.mainBundle.objectForInfoDictionaryKey(VERSION_PLIST_KEY) as? String ?: UNKNOWN_VERSION,
    )
}

/** The hardware id (`iPhone16,1`); the simulator reports the Mac's architecture. */
@OptIn(ExperimentalForeignApi::class)
private fun machineId(): String? = memScoped {
    val info = alloc<utsname>()
    if (uname(info.ptr) != 0) return@memScoped null
    info.machine.toKString().takeIf { it.isNotBlank() }
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
