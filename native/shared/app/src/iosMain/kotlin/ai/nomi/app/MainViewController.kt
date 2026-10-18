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

/** Info.plist key holding the backend URL (set per configuration in project.yml). */
private const val API_URL_PLIST_KEY = "NomiApiUrl"
private const val DEFAULT_IOS_API_URL = "http://localhost:8080"

private var koinStarted = false

@Composable
actual fun rememberReducedMotion(): Boolean = remember { UIAccessibilityIsReduceMotionEnabled() }

fun iosPlatformModule(): Module = module {
    val apiUrl = (NSBundle.mainBundle.objectForInfoDictionaryKey(API_URL_PLIST_KEY) as? String)
        ?.takeIf { it.isNotBlank() } ?: DEFAULT_IOS_API_URL
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
