package ai.nomi.app

import ai.nomi.data.SdkSetup
import ai.nomi.network.AndroidTokenStore
import ai.nomi.network.ApiConfig
import ai.nomi.network.TokenStore
import ai.nomi.sdk.DeviceInfo
import android.content.Context
import android.os.Build
import android.content.Intent
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

/** Android's share sheet, for a shared reply's link. */
@Composable
actual fun rememberShareText(): ((String) -> Unit)? {
    val context = LocalContext.current
    return remember(context) {
        { text ->
            val send = Intent(Intent.ACTION_SEND).setType(TEXT_PLAIN).putExtra(Intent.EXTRA_TEXT, text)
            context.startActivity(Intent.createChooser(send, null).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK))
        }
    }
}

private const val TEXT_PLAIN = "text/plain"

/** Android's platform pieces; start Koin with `androidContext(application)`. [apiUrl] comes from BuildConfig. */
fun androidPlatformModule(apiUrl: String): Module = module {
    require(apiUrl.isNotBlank()) { "No backend URL: build release with -Pnomi.apiUrl.release=https://…" }
    single { ApiConfig(apiUrl) }
    single<TokenStore> { AndroidTokenStore(androidContext()) }
    single { SdkSetup(get<ApiConfig>().baseUrl, androidContext().filesDir.resolve(SDK_DIR).path, androidDevice(androidContext())) }
}

private const val PLATFORM_ANDROID = "android"
private const val OS_ANDROID = "Android"
private const val UNKNOWN_VERSION = "unknown"

/** This phone or tablet, as the person's device list shows it: "Pixel 9", google, Android 15. */
private fun androidDevice(context: Context): DeviceInfo {
    val version = runCatching { context.packageManager.getPackageInfo(context.packageName, 0).versionName }.getOrNull()
    val brand = Build.BRAND.takeIf { it.isNotBlank() }
    val model = Build.MODEL.takeIf { it.isNotBlank() }
    // A model that already names its maker ("Samsung SM-S918B" vs "Pixel 9") isn't prefixed again.
    val name = model?.let { if (brand == null || it.startsWith(brand, ignoreCase = true)) it else "${brand.replaceFirstChar(Char::uppercase)} $it" }
    return DeviceInfo(
        platform = PLATFORM_ANDROID,
        name = name,
        brand = brand,
        model = model,
        osName = OS_ANDROID,
        osVersion = Build.VERSION.RELEASE,
        appVersion = version ?: UNKNOWN_VERSION,
    )
}
