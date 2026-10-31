package ai.nomi.network

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

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
