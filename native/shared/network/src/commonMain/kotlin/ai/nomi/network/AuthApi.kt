package ai.nomi.network

import ai.nomi.network.dto.AccessTokenResponse
import ai.nomi.network.dto.ChallengeRequest
import ai.nomi.network.dto.LoginRequest
import ai.nomi.network.dto.RefreshRequest
import ai.nomi.network.dto.TokenPairResponse
import ai.nomi.network.dto.VerifyRequest
import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.request.HttpRequestBuilder
import io.ktor.client.request.post
import io.ktor.client.request.setBody
import io.ktor.client.statement.HttpResponse
import io.ktor.http.ContentType
import io.ktor.http.HttpStatusCode
import io.ktor.http.contentType
import io.ktor.http.isSuccess
import kotlinx.coroutines.CancellationException
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.decodeFromJsonElement
import kotlinx.serialization.json.intOrNull
import kotlinx.serialization.json.jsonPrimitive

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

/** Why an emailed sign-in code was not accepted or resent (`code_error` in routes/auth.rs). */
sealed interface EmailCodeError {
    /** The code is wrong; [attemptsLeft] more tries before the sign-in locks. */
    data class WrongCode(val attemptsLeft: Int) : EmailCodeError

    /** The sign-in expired: start over. */
    data object Expired : EmailCodeError

    /** Too many wrong codes: start over. */
    data object TooManyAttempts : EmailCodeError

    /** A new code can be sent in [seconds]. */
    data class Wait(val seconds: Int) : EmailCodeError

    /** No more codes for this sign-in. */
    data object TooManyCodes : EmailCodeError

    /** The server or the network failed; try again. */
    data object Failed : EmailCodeError
}

class EmailCodeException(val error: EmailCodeError) : RuntimeException(error.toString())

internal const val LOGIN_PATH = "/api/auth/login"
internal const val REFRESH_PATH = "/api/auth/refresh"
internal const val LOGOUT_PATH = "/api/auth/logout"
internal const val VERIFY_PATH = "/api/auth/verify"
internal const val RESEND_PATH = "/api/auth/resend"
private const val ERROR_FIELD = "error"
private const val ATTEMPTS_LEFT_FIELD = "attempts_left"
private const val SECONDS_FIELD = "seconds"
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

    /** Finishes a sign-in with the emailed code and stores the session; throws [EmailCodeException] otherwise. */
    suspend fun verifyCode(challengeId: String, code: String): Tokens {
        val response = postOrFail(VERIFY_PATH) { setBody(VerifyRequest(challengeId, code)) }
        val pair = response.body<TokenPairResponse>()
        val session = Tokens(pair.accessToken, pair.refreshToken)
        tokens.write(session)
        client.clearCachedTokens()
        return session
    }

    /** Emails a fresh code (the old one stops working); throws [EmailCodeException] when it can't. */
    suspend fun resendCode(challengeId: String): VerificationInfo =
        postOrFail(RESEND_PATH) { setBody(ChallengeRequest(challengeId)) }.body()

    private suspend fun postOrFail(path: String, body: HttpRequestBuilder.() -> Unit): HttpResponse {
        val response = try {
            client.post(config.url(path)) {
                contentType(ContentType.Application.Json)
                body()
            }
        } catch (cancelled: CancellationException) {
            throw cancelled
        } catch (_: Exception) {
            throw EmailCodeException(EmailCodeError.Failed)
        }
        if (!response.status.isSuccess()) throw EmailCodeException(codeError(response))
        return response
    }

    private suspend fun codeError(response: HttpResponse): EmailCodeError {
        val body = try {
            response.body<JsonObject>()
        } catch (cancelled: CancellationException) {
            throw cancelled
        } catch (_: Exception) {
            return EmailCodeError.Failed
        }
        val error = body[ERROR_FIELD]?.jsonPrimitive?.content
        return when (error) {
            "wrong_code" -> body[ATTEMPTS_LEFT_FIELD]?.jsonPrimitive?.intOrNull?.let { EmailCodeError.WrongCode(it) }
            "expired" -> EmailCodeError.Expired
            "too_many_attempts" -> EmailCodeError.TooManyAttempts
            "wait" -> body[SECONDS_FIELD]?.jsonPrimitive?.intOrNull?.let { EmailCodeError.Wait(it) }
            "too_many_codes" -> EmailCodeError.TooManyCodes
            else -> null
        } ?: EmailCodeError.Failed
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
        val current = forgetLocalSession()
        if (current != null) revoke(current.refresh)
    }

    /**
     * Drops the stored session (store and the client's cached bearer) and returns what it held, so the caller can
     * [revoke] it later without that call ever touching tokens a newer sign-in may have written meanwhile.
     */
    suspend fun forgetLocalSession(): Tokens? {
        val current = tokens.read()
        tokens.clear()
        client.clearCachedTokens()
        return current
    }

    /** Best-effort server-side revoke of [refreshToken]; never reads or changes the local token store. */
    suspend fun revoke(refreshToken: String) {
        try {
            client.post(config.url(LOGOUT_PATH)) {
                contentType(ContentType.Application.Json)
                setBody(RefreshRequest(refreshToken))
            }
        } catch (cancelled: CancellationException) {
            throw cancelled
        } catch (_: Exception) {
            // Best effort: the session is already gone locally whether or not the server heard us.
        }
    }
}
