package ai.nomi.data

import ai.nomi.network.ApiException
import ai.nomi.network.EmailCodeError
import ai.nomi.network.EmailCodeException
import ai.nomi.network.VerificationInfo
import ai.nomi.sdk.NomiException
import ai.nomi.sdk.EmailCodeError as SdkEmailCodeError
import ai.nomi.sdk.VerificationInfo as SdkVerificationInfo

internal const val ROLE_ASSISTANT = "assistant"
private const val HTTP_UNAUTHORIZED = 401

// The engine's models as the app's: the app's types predate the engine and screens use them.

internal fun ai.nomi.sdk.Session.toDomain() = Session(id, title, lastMessagePreview, updatedAt, unread)

internal fun ai.nomi.sdk.Message.toDomain() =
    Message(id, sessionId, role, agentType, agentDisplayName, content, contentJson, createdAt, status, myFeedback, stepsJson)

internal fun ai.nomi.sdk.Draft.toDomain() = Draft(turnJobId, text, phase, updatedAt)

internal fun ai.nomi.sdk.AgentActivity.toDomain() =
    AgentActivity(agentSessionId, sessionId, agentType, phase, detail, updatedAt)

internal fun ai.nomi.sdk.NotificationItem.toDomain() = NotificationItem(id, kind, title, body, link, createdAt, readAt)

internal fun SdkVerificationInfo.toDomain() =
    VerificationInfo(challengeId, purpose, email, expiresIn, resendIn, sendsLeft)

/**
 * The engine's errors as the ones screens already handle: HTTP refusals as [ApiException], code problems as
 * [EmailCodeException], a closed account as [SessionEndedException]. Anything else stays as it is.
 */
internal suspend fun <T> translated(block: suspend () -> T): T = try {
    block()
} catch (e: NomiException) {
    throw when (e) {
        is NomiException.Unauthorized -> ApiException(HTTP_UNAUTHORIZED, "unauthorized")
        is NomiException.Http -> ApiException(e.status.toInt(), e.reason)
        is NomiException.EmailCode -> EmailCodeException(e.error.toDomain())
        is NomiException.AccountClosed, is NomiException.NotSignedIn -> SessionEndedException()
        else -> e
    }
}

private fun SdkEmailCodeError.toDomain(): EmailCodeError = when (this) {
    is SdkEmailCodeError.WrongCode -> EmailCodeError.WrongCode(attemptsLeft.toInt())
    SdkEmailCodeError.Expired -> EmailCodeError.Expired
    SdkEmailCodeError.TooManyAttempts -> EmailCodeError.TooManyAttempts
    is SdkEmailCodeError.Wait -> EmailCodeError.Wait(seconds.toInt())
    SdkEmailCodeError.TooManyCodes -> EmailCodeError.TooManyCodes
}
