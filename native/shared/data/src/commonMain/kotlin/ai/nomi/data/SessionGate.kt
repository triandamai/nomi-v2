package ai.nomi.data

import ai.nomi.network.SignInResult
import ai.nomi.realtime.RealtimeHub
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock

/**
 * Whether the app is signed in, and everything that follows from it: signing in opens the one realtime socket and
 * starts [RealtimeSync]; signing out (by the person, or because the server rejected the refresh token) stops both,
 * forgets the tokens and wipes the local cache. The UI routes on [signedIn].
 *
 * [restore] and [signIn] run on the main thread (they start [RealtimeSync]); [signOut] may be called from anywhere,
 * it hops onto [scope] (main in the apps).
 */
class SessionGate(
    private val auth: AuthRepository,
    private val hub: RealtimeHub,
    private val sync: RealtimeSync,
    private val clearLocalData: suspend () -> Unit,
    private val scope: CoroutineScope,
    private val log: (String) -> Unit = {},
) {
    private val mutableSignedIn = MutableStateFlow(false)
    private val mutableRestored = MutableStateFlow(false)
    private val lock = Mutex()
    private var live = false

    val signedIn: StateFlow<Boolean> = mutableSignedIn.asStateFlow()

    /** False until [restore] has looked for a stored session: the UI shows neither sign-in nor home before. */
    val restored: StateFlow<Boolean> = mutableRestored.asStateFlow()

    /** App start: resumes a stored session. A token store that cannot be read counts as signed out. */
    suspend fun restore() {
        lock.withLock {
            val stored = try {
                auth.hasSession()
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                log("session: reading the stored session failed: $e")
                false
            }
            if (!live && stored) startSession()
            mutableRestored.value = true
        }
    }

    /** Serialized with sign-out, so a sign-out still finishing can never clear the tokens this writes. */
    suspend fun signIn(email: String, password: String): SignInResult = lock.withLock {
        val result = auth.signIn(email, password)
        if (result is SignInResult.SignedIn) startSession()
        result
    }

    /** Idempotent. */
    fun signOut() {
        scope.launch { endSession() }
    }

    /** The realtime hub's refresh: a rejected refresh token ends the session. */
    suspend fun refreshSession(): Boolean {
        val ok = auth.refresh()
        if (!ok) signOut()
        return ok
    }

    private fun startSession() {
        if (live) return
        live = true
        mutableSignedIn.value = true
        sync.start()
        hub.start()
    }

    /**
     * Everything local happens under [lock] (tokens forgotten, cache wiped) before a new sign-in can start; only the
     * server-side revoke of the captured refresh token runs after it, and it cannot touch newer tokens.
     */
    private suspend fun endSession() {
        val toRevoke = lock.withLock {
            val wasLive = live
            live = false
            mutableSignedIn.value = false
            if (wasLive) {
                sync.stop()
                hub.stop()
            }
            var refreshToken: String? = null
            attempt("forget tokens") { refreshToken = auth.forgetSession() }
            attempt("clear local data") { clearLocalData() }
            refreshToken
        }
        if (toRevoke != null) attempt("revoke session") { auth.revoke(toRevoke) }
    }

    private suspend fun attempt(what: String, block: suspend () -> Unit) {
        try {
            block()
        } catch (e: CancellationException) {
            throw e
        } catch (e: Exception) {
            log("session: $what failed: $e")
        }
    }
}
