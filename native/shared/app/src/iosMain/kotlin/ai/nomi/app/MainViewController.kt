package ai.nomi.app

import ai.nomi.app.di.initKoin
import ai.nomi.database.DatabaseFactory
import ai.nomi.database.IosDatabaseFactory
import ai.nomi.network.ApiConfig
import ai.nomi.network.IosTokenStore
import ai.nomi.network.TokenStore
import ai.nomi.network.defaultHttpEngine
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.window.ComposeUIViewController
import io.ktor.client.engine.HttpClientEngine
import org.koin.core.module.Module
import org.koin.dsl.module
import platform.Foundation.NSBundle
import platform.UIKit.UIAccessibilityIsReduceMotionEnabled
import platform.UIKit.UIViewController

/** Info.plist key holding the backend URL (Debug: the local backend; Release: required at build time, project.yml). */
private const val API_URL_PLIST_KEY = "NomiApiUrl"

private var koinStarted = false

@Composable
actual fun rememberReducedMotion(): Boolean = remember { UIAccessibilityIsReduceMotionEnabled() }

fun iosPlatformModule(): Module = module {
    val apiUrl = requireNotNull((NSBundle.mainBundle.objectForInfoDictionaryKey(API_URL_PLIST_KEY) as? String)?.takeIf { it.isNotBlank() }) {
        "Info.plist has no $API_URL_PLIST_KEY: set the NOMI_API_URL build setting"
    }
    single { ApiConfig(apiUrl) }
    single<TokenStore> { IosTokenStore() }
    single<DatabaseFactory> { IosDatabaseFactory() }
    single<HttpClientEngine> { defaultHttpEngine() }
}

/** Entry point for the SwiftUI host: `MainViewControllerKt.MainViewController()`. Starts Koin on first call. */
fun MainViewController(): UIViewController {
    if (!koinStarted) {
        initKoin(iosPlatformModule())
        koinStarted = true
    }
    return ComposeUIViewController { NomiApp() }
}
