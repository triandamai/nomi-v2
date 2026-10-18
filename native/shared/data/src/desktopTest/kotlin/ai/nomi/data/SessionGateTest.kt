package ai.nomi.data

import ai.nomi.database.NomiDatabase
import ai.nomi.database.clearAll
import ai.nomi.database.entity.SessionEntity
import ai.nomi.network.EmailCodeError
import ai.nomi.network.EmailCodeException
import ai.nomi.network.SignInResult
import ai.nomi.network.Tokens
import ai.nomi.network.SessionDto
import ai.nomi.network.VerificationInfo
import ai.nomi.realtime.RealtimeEvent
import ai.nomi.realtime.Topic
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.delay
import kotlinx.coroutines.withContext
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.yield
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.runTest
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
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

    /** When set, [signIn] suspends until it completes: a login POST still in flight. */
    var signInGate: CompletableDeferred<Unit>? = null
    var failRead = false

    override suspend fun hasSession(): Boolean {
        if (failRead) error("keychain unavailable")
        return session != null
    }

    override suspend fun signIn(email: String, password: String): SignInResult {
        signInGate?.await()
        return nextSignIn.also { if (it is SignInResult.SignedIn) session = it.tokens }
    }

    /** What [verifyCode] does: store and return these tokens, or throw this error. */
    var nextCode: Result<Tokens> = Result.success(Tokens("a-code", "r-code"))
    val verified = mutableListOf<Pair<String, String>>()

    override suspend fun verifyCode(challengeId: String, code: String): Tokens {
        verified += challengeId to code
        return nextCode.getOrThrow().also { session = it }
    }

    override suspend fun resendCode(challengeId: String): VerificationInfo =
        VerificationInfo(challengeId, "login", "a***@b.c", 600, 60, 2)

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
    private val sessionsApi = FakeSessionsApi()
    private lateinit var sessionRepository: SessionRepository

    @BeforeTest
    fun open() {
        db = inMemoryDb()
    }

    @AfterTest
    fun close() = db.close()

    private fun TestScope.gate(): SessionGate {
        val session = SessionScope(backgroundScope)
        sessionRepository = SessionRepository(sessionsApi, db, clock, session)
        val sync = RealtimeSync(
            hub,
            sessionRepository,
            NotificationRepository(FakeNotificationsApi(), db, clock, session),
            ActivityRepository(db, clock),
            ForegroundChat(),
            session,
        )
        return SessionGate(auth, hub, sync, session, clearLocalData = { db.clearAll() }, scope = backgroundScope)
    }

    /**
     * Room writes run on its own threads, outside the test scheduler: give a write that should never happen real
     * time to land before asserting it did not.
     */
    private suspend fun TestScope.settleRoom() {
        runCurrent()
        withContext(Dispatchers.Default) { delay(ROOM_SETTLE_MS) }
        runCurrent()
    }

    private companion object {
        const val ROOM_SETTLE_MS = 300L
    }

    private fun previousAccountsChat() =
        SessionDto(id = "a-chat", title = "A's chat", lastMessage = null, agentActive = false, updatedAt = "2026-10-18T10:00:00Z")

    @Test
    fun user_topic_work_queued_before_sign_out_never_writes_after_the_wipe() = runTest {
        val gate = gate()
        gate.signIn("a@b.c", "pw")
        sessionsApi.sessions = listOf(previousAccountsChat())
        val inFlight = CompletableDeferred<Unit>()
        sessionsApi.listGate = inFlight
        hub.emit(RealtimeEvent.Resynced(Topic.User)) // queues a sessions refresh, now waiting on the server
        runCurrent()

        gate.signOut()
        gate.signedIn.first { !it }
        runCurrent()
        inFlight.complete(Unit) // the old account's list arrives after the wipe
        settleRoom()

        assertEquals(emptyList(), db.sessionDao().observeAll().first().map { it.id })
    }

    @Test
    fun a_screen_refresh_in_flight_at_sign_out_never_writes_after_the_wipe() = runTest {
        val gate = gate()
        gate.signIn("a@b.c", "pw")
        sessionsApi.sessions = listOf(previousAccountsChat())
        val inFlight = CompletableDeferred<Unit>()
        sessionsApi.listGate = inFlight
        // As a ViewModel would: its own scope, not the session's.
        val screen = backgroundScope.async { runCatching { sessionRepository.refresh() } }
        runCurrent()

        gate.signOut()
        gate.signedIn.first { !it }
        runCurrent()
        inFlight.complete(Unit)
        settleRoom()

        assertTrue(screen.await().exceptionOrNull() is SessionEndedException)
        assertEquals(emptyList(), db.sessionDao().observeAll().first().map { it.id })
    }

    @Test
    fun a_sign_out_requested_for_an_earlier_session_does_not_end_the_next_one() = runTest {
        val gate = gate()
        auth.nextSignIn = SignInResult.SignedIn(Tokens("a1", "r1"))
        gate.signIn("a@b.c", "pw")
        gate.signOut()
        gate.signedIn.first { !it }

        // B signs in; while the login is in flight, a late 401 from A's session asks for a sign-out.
        auth.nextSignIn = SignInResult.SignedIn(Tokens("b1", "r2"))
        val loginInFlight = CompletableDeferred<Unit>()
        auth.signInGate = loginInFlight
        val signingIn = backgroundScope.launch { gate.signIn("b@b.c", "pw") }
        runCurrent()
        gate.signOut()
        runCurrent()
        loginInFlight.complete(Unit)
        signingIn.join()
        runCurrent()

        assertTrue(gate.signedIn.value)
        assertEquals(Tokens("b1", "r2"), auth.session)
        assertEquals(1, hub.stops)
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
    fun right_email_code_signs_in_and_starts_the_session() = runTest {
        val gate = gate()

        gate.verifyCode("c1", "123456")

        assertEquals(listOf("c1" to "123456"), auth.verified)
        assertTrue(gate.signedIn.value)
        assertEquals(1, hub.starts)
    }

    @Test
    fun wrong_email_code_starts_nothing() = runTest {
        val gate = gate()
        auth.nextCode = Result.failure(EmailCodeException(EmailCodeError.WrongCode(2)))

        assertFailsWith<EmailCodeException> { gate.verifyCode("c1", "000000") }

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
