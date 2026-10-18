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
import kotlin.concurrent.Volatile

/**
 * Whether the app is signed in, and everything that follows from it: signing in opens the one realtime socket and
 * starts [RealtimeSync]; signing out (by the person, or because the server rejected the refresh token) stops both,
 * forgets the tokens and wipes the local cache. The UI routes on [signedIn].
 *
 * [restore] and [signIn] run on the main thread (they start [RealtimeSync]); [signOut] may be called from anywhere,
 * it hops onto [scope] (main in the apps).
 *
 * Each session gets its own [SessionScope] and an epoch number. A sign-out is for the session that was current when
 * it was asked for: one still queued when a newer session has started (a late 401 from the old account) is dropped.
 */
class SessionGate(
    private val auth: AuthRepository,
    private val hub: RealtimeHub,
    private val sync: RealtimeSync,
    private val session: SessionScope,
    private val clearLocalData: suspend () -> Unit,
    private val scope: CoroutineScope,
    private val log: (String) -> Unit = {},
) {
    private val mutableSignedIn = MutableStateFlow(false)
    private val mutableRestored = MutableStateFlow(false)
    private val lock = Mutex()
    private var live = false

    /** Bumped by every session start; read from any thread by [signOut]. */
    @Volatile
    private var epoch = 0

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

    /** Idempotent. Ends the session current at the time of the call, never a later one. */
    fun signOut() = signOut(epoch)

    private fun signOut(requestedFor: Int) {
        scope.launch { endSession(requestedFor) }
    }

    /** The realtime hub's refresh: a rejected refresh token ends the session it was made for. */
    suspend fun refreshSession(): Boolean {
        val requestedFor = epoch
        val ok = auth.refresh()
        if (!ok) signOut(requestedFor)
        return ok
    }

    private fun startSession() {
        if (live) return
        live = true
        epoch++
        mutableSignedIn.value = true
        session.open()
        sync.start()
        hub.start()
    }

    /**
     * Everything local happens under [lock] (session work cancelled and joined, tokens forgotten, cache wiped) before
     * a new sign-in can start; only the server-side revoke of the captured refresh token runs after it, and it
     * cannot touch newer tokens.
     */
    private suspend fun endSession(requestedFor: Int) {
        val toRevoke = lock.withLock {
            if (requestedFor != epoch) {
                log("session: dropping a sign-out meant for an earlier session")
                return
            }
            val wasLive = live
            live = false
            mutableSignedIn.value = false
            if (wasLive) {
                sync.stop()
                hub.stop()
            }
            // Before the wipe: nothing queued or in flight for this account may write after it.
            session.close()
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
