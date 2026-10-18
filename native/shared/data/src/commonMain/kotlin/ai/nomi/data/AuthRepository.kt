package ai.nomi.data

import ai.nomi.network.AuthApi
import ai.nomi.network.SignInResult
import ai.nomi.network.TokenStore
import ai.nomi.network.Tokens
import ai.nomi.network.VerificationInfo
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock

/** The signed-in session's credentials: sign-in, token refresh and sign-out. */
interface AuthRepository {
    /** Whether tokens from an earlier sign-in are stored. */
    suspend fun hasSession(): Boolean

    /** Tokens are stored on [SignInResult.SignedIn]; an email-code challenge stores nothing. */
    suspend fun signIn(email: String, password: String): SignInResult

    /** Finishes an email-code sign-in and stores the session; throws `EmailCodeException` when the code isn't taken. */
    suspend fun verifyCode(challengeId: String, code: String): Tokens

    /** Emails a fresh code for a waiting sign-in; throws `EmailCodeException` when it can't. */
    suspend fun resendCode(challengeId: String): VerificationInfo

    /**
     * Gets a new access token. False when signed out or the server rejected the refresh token (tokens forgotten);
     * throws when the attempt itself failed (offline, 5xx), which is no verdict on the session.
     */
    suspend fun refresh(): Boolean

    /** Forgets the stored session right away; returns its refresh token for a later [revoke] (null if none). */
    suspend fun forgetSession(): String?

    /** Best-effort server-side revoke of [refreshToken]; never touches the stored session (a newer one may exist). */
    suspend fun revoke(refreshToken: String)
}

class DefaultAuthRepository(private val api: AuthApi, private val tokens: TokenStore) : AuthRepository {
    private val refreshing = Mutex()

    override suspend fun hasSession(): Boolean = tokens.read() != null

    override suspend fun signIn(email: String, password: String): SignInResult = api.login(email, password)

    override suspend fun verifyCode(challengeId: String, code: String): Tokens = api.verifyCode(challengeId, code)

    override suspend fun resendCode(challengeId: String): VerificationInfo = api.resendCode(challengeId)

    /**
     * Concurrent callers share one refresh: whoever waited on [refreshing] and finds a different access token than
     * the one it failed with knows someone (another caller, or the HTTP client's own 401 refresh) already renewed it.
     */
    override suspend fun refresh(): Boolean {
        val stale = tokens.read() ?: return false
        return refreshing.withLock {
            val current = tokens.read() ?: return@withLock false
            if (current.access != stale.access) true else api.refresh()
        }
    }

    override suspend fun forgetSession(): String? = api.forgetLocalSession()?.refresh

    override suspend fun revoke(refreshToken: String) = api.revoke(refreshToken)
}
