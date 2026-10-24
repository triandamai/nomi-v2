package ai.nomi.app

import ai.nomi.database.DatabaseFactory
import ai.nomi.database.DesktopDatabaseFactory
import ai.nomi.network.ApiConfig
import ai.nomi.network.DesktopTokenStore
import ai.nomi.network.TokenStore
import ai.nomi.network.defaultHttpEngine
import ai.nomi.network.desktopDataDir
import androidx.compose.runtime.Composable
import io.ktor.client.engine.HttpClientEngine
import org.koin.core.module.Module
import org.koin.dsl.module

/**
 * `-Dnomi.apiUrl=…` wins over the `NOMI_API_URL` environment variable; both fall back to the hosted API. For local
 * development point either at your backend, e.g. `NOMI_API_URL=http://localhost:8080`.
 */
const val API_URL_PROPERTY = "nomi.apiUrl"
const val API_URL_ENV = "NOMI_API_URL"
const val DEFAULT_DESKTOP_API_URL = "https://api-nomi.trian.space"

fun desktopApiUrl(): String =
    System.getProperty(API_URL_PROPERTY)?.takeIf { it.isNotBlank() }
        ?: System.getenv(API_URL_ENV)?.takeIf { it.isNotBlank() }
        ?: DEFAULT_DESKTOP_API_URL

/** The desktop JVM exposes no reduce-motion setting cheaply. */
@Composable
actual fun rememberReducedMotion(): Boolean = false

fun desktopPlatformModule(apiUrl: String = desktopApiUrl()): Module = module {
    val dataDir = desktopDataDir()
    single { ApiConfig(apiUrl) }
    single<TokenStore> { DesktopTokenStore(dataDir) }
    single<DatabaseFactory> { DesktopDatabaseFactory(dataDir) }
    single<HttpClientEngine> { defaultHttpEngine() }
}
