package ai.nomi.network

import ai.nomi.network.dto.AccessTokenResponse
import ai.nomi.network.dto.RefreshRequest
import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.engine.HttpClientEngine
import io.ktor.client.plugins.auth.Auth
import io.ktor.client.plugins.auth.providers.BearerAuthProvider
import io.ktor.client.plugins.auth.providers.BearerTokens
import io.ktor.client.plugins.auth.providers.bearer
import io.ktor.client.plugins.auth.authProvider
import io.ktor.client.plugins.contentnegotiation.ContentNegotiation
import io.ktor.client.plugins.websocket.WebSockets
import io.ktor.client.plugins.websocket.pingInterval
import io.ktor.client.statement.request
import io.ktor.client.request.post
import io.ktor.client.request.setBody
import io.ktor.http.ContentType
import io.ktor.http.HttpStatusCode
import io.ktor.http.Url
import io.ktor.http.contentType
import io.ktor.http.isSuccess
import io.ktor.serialization.kotlinx.json.json
import kotlinx.serialization.json.Json
import kotlin.time.Duration.Companion.seconds

private val WEBSOCKET_PING_INTERVAL = 25.seconds

/** Auth endpoints whose own 401 means "bad credentials", not "access token expired". */
private val PUBLIC_AUTH_PATHS = listOf(LOGIN_PATH, REFRESH_PATH, LOGOUT_PATH)

private fun String.isPublicAuthPath() = PUBLIC_AUTH_PATHS.any { endsWith(it) }

internal val apiJson = Json {
    ignoreUnknownKeys = true
    explicitNulls = false
}

/**
 * The one HTTP client the app talks to Nomi with. Bearer tokens come from [tokens]; a 401 triggers a single
 * refresh and retry. A refresh the server rejects ends the session: store cleared, [onSignedOut] called.
 */
fun nomiHttpClient(
    engine: HttpClientEngine,
    config: ApiConfig,
    tokens: TokenStore,
    onSignedOut: () -> Unit,
): HttpClient {
    val apiUrl = Url(config.baseUrl)
    return HttpClient(engine) {
        install(ContentNegotiation) { json(apiJson) }
        install(WebSockets) { pingInterval = WEBSOCKET_PING_INTERVAL }
        install(Auth) {
            bearer {
                loadTokens { tokens.read()?.let { BearerTokens(it.access, it.refresh) } }
                refreshTokens {
                    val stale = oldTokens ?: return@refreshTokens null
                    val failed = response.request.url
                    if (failed.encodedPath.isPublicAuthPath() || failed.host != apiUrl.host || failed.port != apiUrl.port) {
                        return@refreshTokens null
                    }
                    val refreshed = client.post(config.url(REFRESH_PATH)) {
                        markAsRefreshTokenRequest()
                        contentType(ContentType.Application.Json)
                        setBody(RefreshRequest(stale.refreshToken.orEmpty()))
                    }
                    when {
                        refreshed.status == HttpStatusCode.Unauthorized -> {
                            tokens.clear()
                            onSignedOut()
                            null
                        }
                        !refreshed.status.isSuccess() -> null
                        else -> {
                            val access = refreshed.body<AccessTokenResponse>().accessToken
                            val next = Tokens(access, stale.refreshToken.orEmpty())
                            tokens.write(next)
                            BearerTokens(next.access, next.refresh)
                        }
                    }
                }
                sendWithoutRequest { request -> request.url.build().let { it.host == apiUrl.host && it.port == apiUrl.port } }
            }
        }
    }
}

/** Drops the Auth plugin's in-memory token so the next request reloads from the [TokenStore]. */
fun HttpClient.clearCachedTokens() {
    authProvider<BearerAuthProvider>()?.clearToken()
}
