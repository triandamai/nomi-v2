package ai.nomi.data

import ai.nomi.database.NomiDatabase
import ai.nomi.database.clearAll
import ai.nomi.database.entity.SessionEntity
import ai.nomi.network.SignInResult
import ai.nomi.network.Tokens
import ai.nomi.network.VerificationInfo
import ai.nomi.realtime.Topic
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.runTest
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

/** Scripted [AuthRepository]: [session] is what a token store would hold. */
class FakeAuthRepository(var session: Tokens? = null) : AuthRepository {
    var nextSignIn: SignInResult = SignInResult.SignedIn(Tokens("a", "r"))
    var refreshResult = true
    var signOuts = 0

    override suspend fun hasSession(): Boolean = session != null

    override suspend fun signIn(email: String, password: String): SignInResult =
        nextSignIn.also { if (it is SignInResult.SignedIn) session = it.tokens }

    override suspend fun refresh(): Boolean = session != null && refreshResult

    override suspend fun signOut() {
        signOuts++
        session = null
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
}
