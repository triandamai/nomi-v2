package ai.nomi.data

import ai.nomi.database.NomiDatabase
import ai.nomi.realtime.RealtimeEvent
import ai.nomi.realtime.Topic
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.runTest
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.put
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull
import kotlin.test.assertTrue

class ChatRepositoryTest {
    private lateinit var db: NomiDatabase
    private val hub = FakeRealtimeHub()
    private val api = FakeMessagesApi()
    private val clock = FakeClock()
    private val chat = Topic.Chat("s1")

    @BeforeTest
    fun open() {
        db = inMemoryDb()
    }

    @AfterTest
    fun close() = db.close()

    private fun TestScope.repository() =
        ChatRepository(api, db, hub, ActivityRepository(db, clock), backgroundScope, clock)

    private fun delta(text: String, turn: String = "t1") = RealtimeEvent.ChatDelta(
        chat, "s1", turn,
        buildJsonObject { put("TextDelta", buildJsonObject { put("index", JsonPrimitive(0)); put("text", text) }) },
    )

    @Test
    fun resynced_event_refetches_open_chat() = runTest {
        val repo = repository()
        api.latest = listOf(messageDto("m1"))
        repo.bind("s1")
        repo.observe("s1").first { it.messages.map(Message::id) == listOf("m1") }
        assertEquals(1, api.pageCalls)

        api.latest = listOf(messageDto("m1"), messageDto("m2", createdAt = "2026-10-18T10:00:01Z"))
        hub.emit(RealtimeEvent.Resynced(chat))

        repo.observe("s1").first { it.messages.map(Message::id) == listOf("m1", "m2") }
        assertEquals(2, api.pageCalls)
    }

    @Test
    fun delta_then_completed_leaves_one_message_and_no_draft() = runTest {
        val repo = repository()
        repo.bind("s1")
        hub.emit(delta("Hel"))
        clock.now += 1
        hub.emit(delta("lo"))
        val streaming = repo.observe("s1").first { it.draft?.text == "Hello" }
        assertEquals(FakeClock.START_MILLIS + 1, streaming.draft?.updatedAt)

        api.byId["m1"] = messageDto("m1", content = "Hello")
        hub.emit(RealtimeEvent.TurnCompleted(chat, "s1", "t1", "m1"))

        val done = repo.observe("s1").first { it.draft == null && it.messages.isNotEmpty() }
        assertEquals(listOf("Hello"), done.messages.map(Message::content))
    }

    @Test
    fun turn_failed_clears_draft_and_surfaces_error() = runTest {
        val repo = repository()
        repo.bind("s1")
        hub.emit(delta("partial"))
        repo.observe("s1").first { it.draft != null }

        hub.emit(RealtimeEvent.TurnFailed(chat, "s1", "t1", "model overloaded"))

        val state = repo.observe("s1").first { it.draft == null }
        assertEquals("model overloaded", state.lastError)
    }

    @Test
    fun access_lost_is_surfaced() = runTest {
        val repo = repository()
        repo.bind("s1")
        hub.emit(RealtimeEvent.AccessLost(chat))
        assertTrue(repo.observe("s1").first { it.accessLost }.accessLost)
    }

    @Test
    fun agent_session_ended_deletes_activity() = runTest {
        val repo = repository()
        repo.bind("s1")
        hub.emit(RealtimeEvent.AgentSessionStarted(chat, "a1", "s1", "planner", "Planner", "web", "you"))
        hub.emit(RealtimeEvent.AgentPhaseChanged(chat, "a1", "calling_tool", "web_search"))
        val working = repo.observe("s1").first { it.activeAgent?.phase == "calling_tool" }
        assertEquals("planner", working.activeAgent?.agentType)
        assertEquals("web_search", working.activeAgent?.detail)

        hub.emit(RealtimeEvent.AgentSessionEnded(chat, "a1", "s1", "done"))

        repo.observe("s1").first { it.activeAgent == null }
        assertNull(db.agentActivityDao().get("a1"))
    }

    @Test
    fun bind_handle_removes_its_listener() = runTest {
        val repo = repository()
        val first = repo.bind("s1")
        val second = repo.bind("s1")
        // The same chat bound twice shares one listener so each event is applied once.
        assertEquals(1, hub.listenerCount(chat))
        first.remove()
        first.remove()
        assertEquals(1, hub.listenerCount(chat))
        second.remove()
        assertEquals(0, hub.listenerCount(chat))
        repo.bind("s1").remove()
        assertEquals(0, hub.listenerCount(chat))
    }

    @Test
    fun send_posts_and_stores_the_server_user_message() = runTest {
        val repo = repository()
        repo.send("s1", "hello")
        assertEquals(listOf("s1" to "hello"), api.sent)
        val state = repo.observe("s1").first { it.messages.isNotEmpty() }
        assertEquals("user", state.messages.single().role)
        assertEquals("hello", state.messages.single().content)
    }
}
