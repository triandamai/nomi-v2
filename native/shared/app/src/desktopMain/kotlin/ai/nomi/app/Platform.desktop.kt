package ai.nomi.app

import ai.nomi.data.SdkSetup
import ai.nomi.network.ApiConfig
import ai.nomi.network.DesktopTokenStore
import ai.nomi.network.TokenStore
import ai.nomi.network.desktopDataDir
import ai.nomi.sdk.DeviceInfo
import java.net.InetAddress
import androidx.compose.runtime.Composable
import org.koin.core.module.Module
import org.koin.dsl.module

/**
 * `-Dnomi.apiUrl=…` wins over the `NOMI_API_URL` environment variable; both fall back to the hosted API. For local
 * development point either at your backend, e.g. `NOMI_API_URL=http://localhost:8080`.
 */
const val API_URL_PROPERTY = "nomi.apiUrl"
const val API_URL_ENV = "NOMI_API_URL"
const val DEFAULT_DESKTOP_API_URL = "https://api-nomi.trian.space"

/** The Rust engine's data, under the app's data directory. */
private const val SDK_DIR = "sdk"

fun desktopApiUrl(): String =
    System.getProperty(API_URL_PROPERTY)?.takeIf { it.isNotBlank() }
        ?: System.getenv(API_URL_ENV)?.takeIf { it.isNotBlank() }
        ?: DEFAULT_DESKTOP_API_URL

/** The desktop JVM exposes no reduce-motion setting cheaply. */
@Composable
actual fun rememberReducedMotion(): Boolean = false

/** Desktop has no share sheet: a shared link is copied. */
@Composable
actual fun rememberShareText(): ((String) -> Unit)? = null

fun desktopPlatformModule(apiUrl: String = desktopApiUrl()): Module = module {
    val dataDir = desktopDataDir()
    single { ApiConfig(apiUrl) }
    single<TokenStore> { DesktopTokenStore(dataDir) }
    single { SdkSetup(get<ApiConfig>().baseUrl, dataDir.resolve(SDK_DIR).path, desktopDevice()) }
}

/** The version a packaged app was built as (jpackage), or the one Gradle passes when run from source. */
private val VERSION_PROPERTIES = listOf("jpackage.app-version", "nomi.version")
private const val DEV_VERSION = "dev"
private const val LOCAL_SUFFIX = ".local"

/** This computer, as the person's device list shows it: its name, its maker, the OS and version. */
private fun desktopDevice(): DeviceInfo {
    val os = System.getProperty("os.name").orEmpty()
    val platform = when {
        os.startsWith("Mac", ignoreCase = true) -> "macos"
        os.startsWith("Windows", ignoreCase = true) -> "windows"
        else -> "linux"
    }
    val host = runCatching { InetAddress.getLocalHost().hostName.removeSuffix(LOCAL_SUFFIX) }.getOrNull()?.takeIf { it.isNotBlank() }
    return DeviceInfo(
        platform = platform,
        name = host ?: os,
        brand = if (platform == "macos") "Apple" else null,
        model = System.getProperty("os.arch"),
        osName = os.takeIf { it.isNotBlank() },
        osVersion = System.getProperty("os.version"),
        appVersion = VERSION_PROPERTIES.firstNotNullOfOrNull { System.getProperty(it)?.takeIf(String::isNotBlank) } ?: DEV_VERSION,
    )
}
