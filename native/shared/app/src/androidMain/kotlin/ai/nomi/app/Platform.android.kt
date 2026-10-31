package ai.nomi.app

import ai.nomi.data.SdkSetup
import ai.nomi.network.AndroidTokenStore
import ai.nomi.network.ApiConfig
import ai.nomi.network.TokenStore
import android.provider.Settings
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.platform.LocalContext
import org.koin.android.ext.koin.androidContext
import org.koin.core.module.Module
import org.koin.dsl.module

/** The Rust engine's data, under the app's private files. */
private const val SDK_DIR = "nomi-sdk"

/** Animations are off when the developer/accessibility "animator duration scale" is 0. */
private const val ANIMATIONS_OFF = 0f
private const val DEFAULT_ANIMATOR_SCALE = 1f

@Composable
actual fun rememberReducedMotion(): Boolean {
    val resolver = LocalContext.current.contentResolver
    return remember(resolver) {
        Settings.Global.getFloat(resolver, Settings.Global.ANIMATOR_DURATION_SCALE, DEFAULT_ANIMATOR_SCALE) == ANIMATIONS_OFF
    }
}

/** Android's platform pieces; start Koin with `androidContext(application)`. [apiUrl] comes from BuildConfig. */
fun androidPlatformModule(apiUrl: String): Module = module {
    require(apiUrl.isNotBlank()) { "No backend URL: build release with -Pnomi.apiUrl.release=https://…" }
    single { ApiConfig(apiUrl) }
    single<TokenStore> { AndroidTokenStore(androidContext()) }
    single { SdkSetup(get<ApiConfig>().baseUrl, androidContext().filesDir.resolve(SDK_DIR).path) }
}
