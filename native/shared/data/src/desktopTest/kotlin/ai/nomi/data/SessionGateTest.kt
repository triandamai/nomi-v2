package ai.nomi.data

import ai.nomi.database.NomiDatabase
import ai.nomi.database.clearAll
import ai.nomi.database.entity.SessionEntity
import ai.nomi.network.SignInResult
import ai.nomi.network.Tokens
import ai.nomi.network.VerificationInfo
import ai.nomi.realtime.Topic
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.yield
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.runTest
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertTrue

/** Scripted [AuthRepository]: [session] is what a token store would hold. */
class FakeAuthRepository(var session: Tokens? = null) : AuthRepository {
    var nextSignIn: SignInResult = SignInResult.SignedIn(Tokens("a", "r"))
    var refreshResult = true
    var signOuts = 0
    val revoked = mutableListOf<String>()

    /** When set, [revoke] suspends until it completes: a logout POST still in flight. */
    var revokeGate: CompletableDeferred<Unit>? = null
    var failRead = false

    override suspend fun hasSession(): Boolean {
        if (failRead) error("keychain unavailable")
        return session != null
    }

    override suspend fun signIn(email: String, password: String): SignInResult =
        nextSignIn.also { if (it is SignInResult.SignedIn) session = it.tokens }

    override suspend fun refresh(): Boolean = session != null && refreshResult

    override suspend fun forgetSession(): String? {
        signOuts++
        return session?.refresh.also { session = null }
    }

    override suspend fun revoke(refreshToken: String) {
        revokeGate?.await()
        revoked += refreshToken
    }
}

class SessionGateTest {
    private lateinit var db: NomiDatabase
    private val hub = FakeRealtimeHub()
    private val clock = FakeClock()
    private val auth = FakeAuthRepository()

    @BeforeTest
    fun open() {
        db = inMemoryDb()
    }

    @AfterTest
    fun close() = db.close()

    private fun TestScope.gate(): SessionGate {
        val sync = RealtimeSync(
            hub,
            SessionRepository(FakeSessionsApi(), db, clock),
            NotificationRepository(FakeNotificationsApi(), db, clock),
            ActivityRepository(db, clock),
            ForegroundChat(),
            backgroundScope,
        )
        return SessionGate(auth, hub, sync, clearLocalData = { db.clearAll() }, scope = backgroundScope)
    }

    @Test
    fun sign_in_opens_the_socket_and_listens_on_the_user_topic() = runTest {
        val gate = gate()
        gate.restore()
        assertFalse(gate.signedIn.value)

        val result = gate.signIn("a@b.c", "pw")

        assertTrue(result is SignInResult.SignedIn)
        assertTrue(gate.signedIn.value)
        assertEquals(1, hub.starts)
        assertEquals(1, hub.listenerCount(Topic.User))
    }

    @Test
    fun email_code_challenge_does_not_sign_in() = runTest {
        val gate = gate()
        auth.nextSignIn = SignInResult.EmailCode(VerificationInfo("c1", "login", "a***@b.c", 600, 60, 3))

        gate.signIn("a@b.c", "pw")

        assertFalse(gate.signedIn.value)
        assertEquals(0, hub.starts)
    }

    @Test
    fun restore_with_stored_tokens_starts_the_session() = runTest {
        auth.session = Tokens("a", "r")
        val gate = gate()

        gate.restore()

        assertTrue(gate.restored.value)
        assertTrue(gate.signedIn.value)
        assertEquals(1, hub.starts)
    }

    @Test
    fun sign_out_stops_realtime_and_forgets_tokens_and_cache() = runTest {
        val gate = gate()
        gate.signIn("a@b.c", "pw")
        db.sessionDao().upsert(SessionEntity("s1", "t", "p", 1, unread = false))

        gate.signOut()

        // Sign-out hops onto the gate's scope; the cache wipe goes through Room's own dispatcher.
        gate.signedIn.first { !it }
        db.sessionDao().observeAll().first { it.isEmpty() }
        assertEquals(1, hub.stops)
        assertEquals(0, hub.listenerCount(Topic.User))
        assertEquals(1, auth.signOuts)
    }

    @Test
    fun rejected_refresh_ends_the_session() = runTest {
        val gate = gate()
        gate.signIn("a@b.c", "pw")
        auth.refreshResult = false

        assertFalse(gate.refreshSession())

        gate.signedIn.first { !it }
        assertEquals(1, hub.stops)
    }

    @Test
    fun refresh_without_a_session_is_false() = runTest {
        val gate = gate()
        assertFalse(gate.refreshSession())
    }

    @Test
    fun signing_in_again_while_the_logout_post_is_in_flight_keeps_the_new_session() = runTest {
        val gate = gate()
        auth.nextSignIn = SignInResult.SignedIn(Tokens("a1", "r1"))
        gate.signIn("a@b.c", "pw")
        val inFlight = CompletableDeferred<Unit>()
        auth.revokeGate = inFlight

        gate.signOut()
        gate.signedIn.first { !it }
        auth.nextSignIn = SignInResult.SignedIn(Tokens("a2", "r2"))
        gate.signIn("a@b.c", "pw")
        inFlight.complete(Unit)
        // Let the revoke finish before checking it touched nothing but the old token.
        auth.revokeGate = null
        while (auth.revoked.isEmpty()) yield()

        assertEquals(listOf("r1"), auth.revoked)
        assertEquals(Tokens("a2", "r2"), auth.session)
        assertTrue(gate.signedIn.value)
        assertEquals(2, hub.starts)
    }

    @Test
    fun unreadable_token_store_restores_to_signed_out() = runTest {
        auth.failRead = true
        val gate = gate()

        gate.restore()

        assertTrue(gate.restored.value)
        assertFalse(gate.signedIn.value)
        assertEquals(0, hub.starts)
        assertNull(auth.session)
    }
}
