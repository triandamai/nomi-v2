package ai.nomi.data

import ai.nomi.network.EmailCodeError
import ai.nomi.network.EmailCodeException
import ai.nomi.network.SignInResult
import ai.nomi.network.Tokens
import ai.nomi.network.VerificationInfo
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.withContext
import kotlinx.coroutines.withTimeout
import kotlin.time.Duration.Companion.seconds
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class SessionGateTest {
    private val auth = FakeAuthRepository()
    private val engine = FakeSessionEngine()
    private lateinit var session: SessionScope

    private fun TestScope.gate(): SessionGate {
        session = SessionScope(backgroundScope)
        val nomi = backgroundScope.testEngine()
        val sync = RealtimeSync(nomi, ActivityRepository(nomi), ForegroundChat(), session)
        return SessionGate(auth, engine, sync, session, scope = backgroundScope)
    }

    /**
     * Sign-out first joins the session's work, here a real engine starting on its own threads, outside the test
     * scheduler: give it real time.
     */
    private suspend fun TestScope.awaitWipe() {
        withContext(Dispatchers.Default) {
            withTimeout(WIPE_TIMEOUT) {
                while (engine.wipes == 0) {
                    this@awaitWipe.testScheduler.runCurrent()
                    delay(POLL_MS)
                }
            }
        }
    }

    private companion object {
        val WIPE_TIMEOUT = 5.seconds
        const val POLL_MS = 10L
    }

    @Test
    fun sign_in_begins_the_engine_session() = runTest {
        val gate = gate()
        gate.restore()
        assertFalse(gate.signedIn.value)

        val result = gate.signIn("a@b.c", "pw")

        assertTrue(result is SignInResult.SignedIn)
        assertTrue(gate.signedIn.value)
        assertEquals(1, engine.begins)
    }

    @Test
    fun email_code_challenge_does_not_sign_in() = runTest {
        val gate = gate()
        auth.nextSignIn = SignInResult.EmailCode(VerificationInfo("c1", "login", "a***@b.c", 600, 60, 3))

        gate.signIn("a@b.c", "pw")

        assertFalse(gate.signedIn.value)
        assertEquals(0, engine.begins)
    }

    @Test
    fun right_email_code_signs_in() = runTest {
        val gate = gate()

        gate.verifyCode("c1", "123456")

        assertEquals(listOf("c1" to "123456"), auth.verified)
        assertTrue(gate.signedIn.value)
        assertEquals(1, engine.begins)
    }

    @Test
    fun wrong_email_code_starts_nothing() = runTest {
        val gate = gate()
        auth.nextCode = Result.failure(EmailCodeException(EmailCodeError.WrongCode(2)))

        assertFailsWith<EmailCodeException> { gate.verifyCode("c1", "000000") }

        assertFalse(gate.signedIn.value)
        assertEquals(0, engine.begins)
    }

    @Test
    fun restore_with_a_stored_session_begins_it() = runTest {
        auth.session = Tokens("a", "r")
        val gate = gate()

        gate.restore()

        assertTrue(gate.restored.value)
        assertTrue(gate.signedIn.value)
        assertEquals(1, engine.begins)
    }

    @Test
    fun an_unreadable_token_store_restores_to_signed_out() = runTest {
        auth.failRead = true
        val gate = gate()

        gate.restore()

        assertTrue(gate.restored.value)
        assertFalse(gate.signedIn.value)
        assertEquals(0, engine.begins)
    }

    @Test
    fun sign_out_ends_the_session_and_wipes_local_data() = runTest {
        val gate = gate()
        gate.signIn("a@b.c", "pw")

        gate.signOut()
        gate.signedIn.first { !it }
        awaitWipe()

        assertEquals(1, auth.signOuts)
        assertEquals(1, engine.wipes)
        assertTrue(session.scope.coroutineContext[kotlinx.coroutines.Job]!!.isCancelled)
    }

    @Test
    fun a_session_the_engine_ends_signs_out_here_too() = runTest {
        val gate = gate()
        gate.signIn("a@b.c", "pw")
        runCurrent()

        engine.end()
        gate.signedIn.first { !it }
        awaitWipe()

        assertEquals(1, engine.wipes)
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
        assertEquals(1, engine.wipes)
    }
}
