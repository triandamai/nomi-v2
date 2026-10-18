package ai.nomi.network

import ai.nomi.network.dto.AccessTokenResponse
import ai.nomi.network.dto.LoginRequest
import ai.nomi.network.dto.RefreshRequest
import ai.nomi.network.dto.TokenPairResponse
import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.request.post
import io.ktor.client.request.setBody
import io.ktor.http.ContentType
import io.ktor.http.HttpStatusCode
import io.ktor.http.contentType
import io.ktor.http.isSuccess
import kotlinx.coroutines.CancellationException
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.decodeFromJsonElement

/** Where an emailed sign-in code went (`VerificationInfo` in routes/auth.rs). */
@Serializable
data class VerificationInfo(
    @SerialName("challenge_id") val challengeId: String,
    val purpose: String,
    val email: String,
    @SerialName("expires_in") val expiresIn: Long,
    @SerialName("resend_in") val resendIn: Long,
    @SerialName("sends_left") val sendsLeft: Int,
)

/** Mirrors the untagged `SignInResponse`: tokens, or `{ "verification": {...} }`. */
sealed interface SignInResult {
    data class SignedIn(val tokens: ai.nomi.network.Tokens) : SignInResult
    data class EmailCode(val verification: VerificationInfo) : SignInResult
}

class ApiException(val status: Int, message: String) : RuntimeException(message)

internal const val LOGIN_PATH = "/api/auth/login"
internal const val REFRESH_PATH = "/api/auth/refresh"
internal const val LOGOUT_PATH = "/api/auth/logout"
private const val VERIFICATION_FIELD = "verification"

class AuthApi(
    private val client: HttpClient,
    private val config: ApiConfig,
    private val tokens: TokenStore,
) {
    /** Persists the tokens when the server hands them over; an email-code challenge stores nothing. */
    suspend fun login(email: String, password: String): SignInResult {
        val response = client.post(config.url(LOGIN_PATH)) {
            contentType(ContentType.Application.Json)
            setBody(LoginRequest(email, password))
        }
        if (!response.status.isSuccess()) throw ApiException(response.status.value, "login failed")
        val body = response.body<JsonObject>()
        if (VERIFICATION_FIELD in body) {
            return SignInResult.EmailCode(
                apiJson.decodeFromJsonElement(VerificationInfo.serializer(), body.getValue(VERIFICATION_FIELD)),
            )
        }
        val pair = apiJson.decodeFromJsonElement<TokenPairResponse>(body)
        val session = Tokens(pair.accessToken, pair.refreshToken)
        tokens.write(session)
        client.clearCachedTokens()
        return SignInResult.SignedIn(session)
    }

    /**
     * Exchanges the stored refresh token for a new access token. False when there is no session or the server
     * rejected the refresh token (the session is then forgotten); throws [ApiException] for any other failure,
     * which says nothing about the session.
     */
    suspend fun refresh(): Boolean {
        val current = tokens.read() ?: return false
        val response = client.post(config.url(REFRESH_PATH)) {
            contentType(ContentType.Application.Json)
            setBody(RefreshRequest(current.refresh))
        }
        if (response.status == HttpStatusCode.Unauthorized) {
            tokens.clear()
            client.clearCachedTokens()
            return false
        }
        if (!response.status.isSuccess()) throw ApiException(response.status.value, "refresh failed")
        tokens.write(Tokens(response.body<AccessTokenResponse>().accessToken, current.refresh))
        client.clearCachedTokens()
        return true
    }

    /** Revokes the refresh token server-side (best effort) and always forgets the session locally. */
    suspend fun logout() {
        val current = tokens.read()
        try {
            if (current != null) {
                client.post(config.url(LOGOUT_PATH)) {
                    contentType(ContentType.Application.Json)
                    setBody(RefreshRequest(current.refresh))
                }
            }
        } catch (cancelled: CancellationException) {
            throw cancelled
        } catch (_: Exception) {
            // Best effort: the local session is dropped below whether or not the server heard us.
        } finally {
            tokens.clear()
            client.clearCachedTokens()
        }
    }
}
