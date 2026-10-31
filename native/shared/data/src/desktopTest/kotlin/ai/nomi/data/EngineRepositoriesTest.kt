package ai.nomi.data

import ai.nomi.network.Tokens
import ai.nomi.sdk.Credentials
import ai.nomi.sdk.NomiException
import ai.nomi.sdk.NotificationItem as SdkNotificationItem
import ai.nomi.sdk.Session as SdkSession
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.withContext
import kotlinx.coroutines.withTimeout
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertNull
import kotlin.test.assertTrue
import kotlin.time.Duration.Companion.seconds

private const val USER_ID = "8c3e7a51-user"
private const val CHAT = "c1"
private val TEST_TIMEOUT = 10.seconds

/**
 * The repositories over a real engine (the native library), against a backend that can't be reached: what they read
 * from the engine's store, what they write locally, and how the engine's errors reach the screens.
 */
class EngineRepositoriesTest {
    private fun engineTest(body: suspend CoroutineScope.(NomiEngine, SessionScope) -> Unit) = runTest {
        withContext(Dispatchers.Default) {
            withTimeout(TEST_TIMEOUT) {
                val scope = CoroutineScope(coroutineContext + kotlinx.coroutines.SupervisorJob())
                val signedIn = MemoryCredentials(Credentials(accessToken(USER_ID), "refresh"))
                val engine = scope.testEngine(signedIn)
                engine.get().openAccount(USER_ID)
                try {
                    body(engine, scope.openSession())
                } finally {
                    engine.get().close()
                    scope.cancel()
                }
            }
        }
    }

    @Test
    fun the_chat_list_and_inbox_read_the_engines_store() = engineTest { engine, _ ->
        val store = engine.account()
        store.upsertSessions(listOf(SdkSession(CHAT, "Plans", "See you", updatedAt = 5, unread = true)))
        store.upsertNotifications(listOf(SdkNotificationItem("n1", "deploy", "Live", "B", null, createdAt = 1, readAt = null)))

        val sessions = SessionRepository(engine)
        assertEquals(listOf(Session(CHAT, "Plans", "See you", 5, true)), sessions.observeAll().first { it.isNotEmpty() })
        sessions.markRead(CHAT)
        assertEquals(false, sessions.observeAll().first { !it.single().unread }.single().unread)

        val notifications = NotificationRepository(engine)
        assertEquals(1, notifications.observeUnreadCount().first { it == 1 })
        notifications.markRead("n1")
        assertEquals(0, notifications.observeUnreadCount().first { it == 0 })
        assertTrue(notifications.observeAll().first().single().readAt != null)
    }

    @Test
    fun a_sent_message_shows_at_once_while_the_engine_delivers_it() = engineTest { engine, session ->
        val chats = ChatRepository(engine, ActivityRepository(engine), session)

        chats.send(CHAT, "Hello")

        val state = chats.observe(CHAT).first { it.messages.isNotEmpty() }
        val shown = state.messages.single()
        assertEquals("Hello", shown.content)
        assertEquals("sending", shown.status)
        assertNull(state.draft)
        assertNull(state.lastError)
    }

    @Test
    fun a_queued_message_can_be_discarded_and_only_a_failed_one_retried() = engineTest { engine, session ->
        val chats = ChatRepository(engine, ActivityRepository(engine), session)
        chats.send(CHAT, "Never mind")
        val queued = chats.observe(CHAT).first { it.messages.isNotEmpty() }.messages.single()

        // Still waiting to go out (the backend can't be reached), not failed: nothing to retry.
        assertEquals(false, chats.retrySend(queued.id))
        assertTrue(chats.discardSend(queued.id))

        chats.observe(CHAT).first { it.messages.isEmpty() }
        assertTrue(engine.account().outboxEntries().isEmpty())
        assertEquals(false, chats.discardSend(queued.id))
        assertEquals(false, chats.retrySend("m-not-local"))
    }

    @Test
    fun the_engines_errors_reach_screens_as_they_expect() = engineTest { engine, session ->
        val chats = ChatRepository(engine, ActivityRepository(engine), session)
        assertFailsWith<NomiException.Network> { SessionRepository(engine).refresh() }
        assertFailsWith<NomiException.Network> { chats.resolveApproval(CHAT, "m1", approve = true, remember = false) }
        assertFailsWith<NomiException.InvalidArgument> { chats.send(CHAT, "  ") }

        engine.get().closeAccount()
        assertFailsWith<SessionEndedException> { SessionRepository(engine).markRead(CHAT) }
        assertFailsWith<SessionEndedException> { SessionRepository(engine).refresh() }
    }

    @Test
    fun flows_follow_the_open_account() = engineTest { engine, _ ->
        engine.account().upsertSessions(listOf(SdkSession(CHAT, "", "", updatedAt = 1, unread = false)))
        val sessions = SessionRepository(engine)
        sessions.observeAll().first { it.isNotEmpty() }

        engine.get().closeAccount()

        assertEquals(emptyList(), sessions.observeAll().first())
    }

    @Test
    fun binding_a_chat_survives_an_unreachable_backend() = engineTest { engine, session ->
        val chats = ChatRepository(engine, ActivityRepository(engine), session)
        val binding = chats.bind(CHAT)
        chats.observe(CHAT).first()
        binding.remove()
        binding.remove()
    }

    @Test
    fun the_engine_shares_the_apps_token_store() = runTest {
        val tokens = FakeTokenStore(Tokens("access", "refresh"))
        val credentials = TokenStoreCredentials(tokens)

        assertEquals(Credentials("access", "refresh"), credentials.load())
        credentials.save(Credentials("access-2", "refresh"))
        assertEquals(Tokens("access-2", "refresh"), tokens.tokens)
        credentials.clear()
        assertNull(tokens.tokens)
        assertNull(credentials.load())
    }

    @Test
    fun an_unreadable_token_store_is_signed_out_not_a_crash() = runTest {
        val broken = object : ai.nomi.network.TokenStore {
            override suspend fun read(): Tokens? = error("keychain locked")
            override suspend fun write(tokens: Tokens) = error("keychain locked")
            override suspend fun clear() = error("keychain locked")
        }
        val logged = mutableListOf<String>()
        val credentials = TokenStoreCredentials(broken, log = { logged += it })

        assertNull(credentials.load())
        credentials.save(Credentials("a", "r"))
        credentials.clear()
        assertEquals(3, logged.size)
    }
}
