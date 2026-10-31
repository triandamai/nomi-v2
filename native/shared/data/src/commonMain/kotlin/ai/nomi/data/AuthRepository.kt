package ai.nomi.data

import ai.nomi.network.SignInResult
import ai.nomi.network.TokenStore
import ai.nomi.network.VerificationInfo
import ai.nomi.sdk.SdkEvent
import ai.nomi.sdk.SignOutReason
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.emitAll
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.map
import ai.nomi.sdk.SignInResult as SdkSignInResult

/** The signed-in session's credentials: sign-in, token refresh and sign-out. */
interface AuthRepository {
    /** Whether a session from an earlier sign-in is stored. */
    suspend fun hasSession(): Boolean

    /** The session is stored on [SignInResult.SignedIn]; an email-code challenge stores nothing. */
    suspend fun signIn(email: String, password: String): SignInResult

    /** Finishes an email-code sign-in and stores the session; throws `EmailCodeException` when the code isn't taken. */
    suspend fun verifyCode(challengeId: String, code: String)

    /** Emails a fresh code for a waiting sign-in; throws `EmailCodeException` when it can't. */
    suspend fun resendCode(challengeId: String): VerificationInfo

    /** Forgets the stored session right away and revokes it on the backend, best effort. */
    suspend fun signOut()
}

/** Auth through the Rust engine, which keeps the session in [tokens]. */
class SdkAuthRepository(private val engine: NomiEngine, private val tokens: TokenStore) : AuthRepository {
    override suspend fun hasSession(): Boolean = engine.get().user.value != null

    override suspend fun signIn(email: String, password: String): SignInResult = translated {
        when (val result = engine.get().signIn(email, password)) {
            // The screens' SignedIn carries the tokens; the engine has just stored them.
            is SdkSignInResult.SignedIn -> SignInResult.SignedIn(checkNotNull(tokens.read()) { "the session wasn't stored" })
            is SdkSignInResult.EmailCodeRequired -> SignInResult.EmailCode(result.verification.toDomain())
        }
    }

    override suspend fun verifyCode(challengeId: String, code: String) {
        translated { engine.get().verifyEmailCode(challengeId, code) }
    }

    override suspend fun resendCode(challengeId: String): VerificationInfo =
        translated { engine.get().resendEmailCode(challengeId).toDomain() }

    override suspend fun signOut() = engine.get().signOut()
}

/** The engine's side of a session: [SessionGate] begins it at sign-in and wipes it at sign-out. */
interface SessionEngine {
    /** Starts the engine and opens the signed-in user's local data. */
    suspend fun begin()

    /** Deletes every account's local data. */
    suspend fun wipe()

    /** The engine ended the session on its own (the backend refused the refresh token, or the socket's access). */
    val ended: Flow<Unit>
}

class EngineSession(private val engine: NomiEngine) : SessionEngine {
    override suspend fun begin() {
        val client = engine.get()
        client.start()
        val user = client.user.value ?: throw SessionEndedException()
        client.openAccount(user.userId)
    }

    override suspend fun wipe() = engine.get().resetStorage()

    override val ended: Flow<Unit> = flow {
        emitAll(
            engine.get().events
                .filter { it is SdkEvent.SignedOut && it.reason != SignOutReason.REQUESTED }
                .map { },
        )
    }
}
