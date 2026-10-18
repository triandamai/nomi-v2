package ai.nomi.data

import ai.nomi.database.NomiDatabase
import ai.nomi.database.entity.AgentActivityEntity
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
        ChatRepository(api, db, hub, ActivityRepository(db, clock), backgroundScope.openSession(), clock)

    private fun delta(text: String, turn: String = "t1") = RealtimeEvent.ChatDelta(
        chat, "s1", turn,
        buildJsonObject { put("TextDelta", buildJsonObject { put("index", JsonPrimitive(0)); put("text", text) }) },
    )

    /** A partial draft and a working agent, as left behind when TurnCompleted / AgentSessionEnded were missed. */
    private suspend fun seedStaleTurn() {
        db.streamingDraftDao().appendDelta("s1", "t-old", "half a rep", clock.now)
        db.agentActivityDao().upsert(AgentActivityEntity("a-old", "s1", "planner", "writing", null, clock.now))
    }

    @Test
    fun resynced_event_refetches_open_chat() = runTest {
        val repo = repository()
        api.latest = listOf(messageDto("m1"))
        repo.bind("s1")
        repo.observe("s1").first { it.messages.map(Message::id) == listOf("m1") }
        assertEquals(1, api.pageCalls)

        seedStaleTurn()
        hub.emit(RealtimeEvent.AccessLost(chat))
        repo.observe("s1").first { it.draft != null && it.activeAgent != null && it.accessLost }
        api.latest = listOf(messageDto("m1"), messageDto("m2", createdAt = "2026-10-18T10:00:01Z"))
        hub.emit(RealtimeEvent.Resynced(chat))

        repo.observe("s1").first {
            it.messages.map(Message::id) == listOf("m1", "m2") && it.draft == null && it.activeAgent == null && !it.accessLost
        }
        assertEquals(2, api.pageCalls)
        assertNull(db.agentActivityDao().get("a-old"))
    }

    @Test
    fun first_bind_drops_draft_and_agents_left_from_before() = runTest {
        seedStaleTurn()
        val repo = repository()
        api.latest = listOf(messageDto("m1"))
        repo.bind("s1")
        val state = repo.observe("s1").first { it.messages.isNotEmpty() && it.draft == null && it.activeAgent == null }
        assertEquals(listOf("m1"), state.messages.map(Message::id))
    }

    @Test
    fun completed_turn_clears_only_its_own_draft_and_ignores_late_deltas() = runTest {
        val repo = repository()
        repo.bind("s1")
        hub.emit(delta("first", turn = "t1"))
        api.byId["m1"] = messageDto("m1", content = "first")
        hub.emit(RealtimeEvent.TurnCompleted(chat, "s1", "t1", "m1"))
        hub.emit(delta("late", turn = "t1"))
        api.byId["m2"] = messageDto("m2", createdAt = "2026-10-18T10:00:01Z")
        hub.emit(RealtimeEvent.MessageUpdated(chat, "s1", "m2"))
        // Events are applied in order, so once m2 is stored the late delta has been handled. Read the draft from the
        // DAO: the combined ChatState may still pair the new messages with a not-yet-requeried draft.
        repo.observe("s1").first { it.messages.size == 2 }
        assertNull(db.streamingDraftDao().get("s1"))

        hub.emit(delta("second", turn = "t2"))
        hub.emit(RealtimeEvent.TurnCompleted(chat, "s1", "t-stale", "m1"))
        api.byId["m3"] = messageDto("m3", createdAt = "2026-10-18T10:00:02Z")
        hub.emit(RealtimeEvent.MessageUpdated(chat, "s1", "m3"))
        repo.observe("s1").first { it.messages.size == 3 }
        assertEquals("second", db.streamingDraftDao().get("s1")?.text)
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

    @Test
    fun a_chat_reopened_in_a_new_session_gets_a_live_binding() = runTest {
        val session = backgroundScope.openSession()
        val repo = ChatRepository(api, db, hub, ActivityRepository(db, clock), session, clock)
        repo.bind("s1") // a screen from the old session, never cleared
        repo.observe("s1").first()
        val callsBefore = api.pageCalls
        session.close()
        session.open()

        api.latest = listOf(messageDto("m-new"))
        repo.bind("s1")

        repo.observe("s1").first { it.messages.map(Message::id) == listOf("m-new") }
        assertTrue(api.pageCalls > callsBefore)
        assertEquals(1, hub.listenerCount(chat))
    }
}
