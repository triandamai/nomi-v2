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

    /** App start: resumes a stored session. */
    suspend fun restore() {
        lock.withLock {
            if (!live && auth.hasSession()) startSession()
            mutableRestored.value = true
        }
    }

    suspend fun signIn(email: String, password: String): SignInResult {
        val result = auth.signIn(email, password)
        if (result is SignInResult.SignedIn) lock.withLock { startSession() }
        return result
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

    private suspend fun endSession() = lock.withLock {
        val wasLive = live
        live = false
        mutableSignedIn.value = false
        if (wasLive) {
            sync.stop()
            hub.stop()
        }
        attempt("forget tokens") { auth.signOut() }
        attempt("clear local data") { clearLocalData() }
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
