package ai.nomi.data

import ai.nomi.network.SignInResult
import ai.nomi.network.VerificationInfo
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
 * Whether the app is signed in, and everything that follows from it: signing in starts the Rust engine (its realtime
 * socket and sync) on the user's local data and starts [RealtimeSync]; signing out (by the person, or because the
 * engine lost the session) stops both, forgets the session and wipes the local data. The UI routes on [signedIn].
 *
 * [restore] and [signIn] run on the main thread (they start [RealtimeSync]); [signOut] may be called from anywhere,
 * it hops onto [scope] (main in the apps). Note that the engine revokes the session before the wipe: offline, that
 * attempt fails fast and the wipe still happens.
 *
 * Each session gets its own [SessionScope] and an epoch number. A sign-out is for the session that was current when
 * it was asked for: one still queued when a newer session has started (a late 401 from the old account) is dropped.
 */
class SessionGate(
    private val auth: AuthRepository,
    private val engine: SessionEngine,
    private val sync: RealtimeSync,
    private val session: SessionScope,
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

    init {
        // A session the engine ended (refresh token refused) ends here too, wiping like any sign-out.
        scope.launch {
            try {
                engine.ended.collect { signOut() }
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                log("session: can't follow the engine's session: $e")
            }
        }
    }

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

    /** The second step of a sign-in that asked for an emailed code; serialized with sign-out like [signIn]. */
    suspend fun verifyCode(challengeId: String, code: String) = lock.withLock {
        auth.verifyCode(challengeId, code)
        startSession()
    }

    /** Emails a fresh code for a waiting sign-in. */
    suspend fun resendCode(challengeId: String): VerificationInfo = auth.resendCode(challengeId)

    /** Idempotent. Ends the session current at the time of the call, never a later one. */
    fun signOut() = signOut(epoch)

    private fun signOut(requestedFor: Int) {
        scope.launch { endSession(requestedFor) }
    }

    private suspend fun startSession() {
        if (live) return
        live = true
        epoch++
        session.open()
        attempt("start the engine") { engine.begin() }
        mutableSignedIn.value = true
        sync.start()
    }

    /**
     * Everything happens under [lock] (session work cancelled and joined, session forgotten and revoked, local data
     * wiped) before a new sign-in can start.
     */
    private suspend fun endSession(requestedFor: Int) {
        lock.withLock {
            if (requestedFor != epoch) {
                log("session: dropping a sign-out meant for an earlier session")
                return
            }
            val wasLive = live
            live = false
            mutableSignedIn.value = false
            if (wasLive) sync.stop()
            // Before the wipe: nothing queued or in flight for this account may write after it.
            session.close()
            attempt("sign out") { auth.signOut() }
            attempt("clear local data") { engine.wipe() }
        }
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
