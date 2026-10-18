package ai.nomi.realtime

import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.jsonObject
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertNull

class EventsTest {
    private fun obj(s: String): JsonObject = WireJson.parseToJsonElement(s).jsonObject
    private val chat = Topic.Chat("s1")

    @Test
    fun delta() {
        val e = decodeEvent(chat, obj("""{"kind":"Delta","turn_job_id":"t1","event":{"TextDelta":{"index":0,"text":"hi"}}}"""))
        assertIs<RealtimeEvent.ChatDelta>(e)
        assertEquals("s1", e.sessionId)
        assertEquals("t1", e.turnJobId)
        assertEquals("hi", e.textOrNull())
    }

    @Test
    fun textOrNullOnlyForTextDelta() {
        fun d(ev: String) = decodeEvent(chat, obj("""{"kind":"Delta","turn_job_id":"t","event":$ev}""")) as RealtimeEvent.ChatDelta
        assertNull(d("""{"ThinkingDelta":{"index":0,"text":"hmm"}}""").textOrNull())
        assertNull(d("""{"ContentBlockDone":{"index":0}}""").textOrNull())
        assertNull(d("""{"TextDelta":{"index":0}}""").textOrNull())
        assertNull(d("""{"TextDelta":"x"}""").textOrNull())
    }

    @Test
    fun turnEvents() {
        assertEquals(
            RealtimeEvent.TurnCompleted(chat, "s1", "t1", "m1"),
            decodeEvent(chat, obj("""{"kind":"TurnCompleted","turn_job_id":"t1","message_id":"m1"}""")),
        )
        assertEquals(
            RealtimeEvent.TurnFailed(chat, "s1", "t1", "bad"),
            decodeEvent(chat, obj("""{"kind":"TurnFailed","turn_job_id":"t1","error":"bad"}""")),
        )
    }

    @Test
    fun messageAndSessionEvents() {
        assertEquals(RealtimeEvent.MessageCreated(chat, "s1", "m1"), decodeEvent(chat, obj("""{"kind":"MessageCreated","message_id":"m1"}""")))
        assertEquals(RealtimeEvent.MessageUpdated(chat, "s1", "m1"), decodeEvent(chat, obj("""{"kind":"MessageUpdated","message_id":"m1"}""")))
        assertEquals(RealtimeEvent.SessionRenamed(chat, "s1", "New"), decodeEvent(chat, obj("""{"kind":"SessionRenamed","title":"New"}""")))
        assertEquals(RealtimeEvent.PersonalityChanged(chat, "s1"), decodeEvent(chat, obj("""{"kind":"PersonalityChanged"}""")))
        assertEquals(RealtimeEvent.AgentDelegationUpdated(chat, "d1"), decodeEvent(chat, obj("""{"kind":"AgentDelegationUpdated","delegation_id":"d1"}""")))
    }

    @Test
    fun agentEvents() {
        assertEquals(
            RealtimeEvent.AgentPhaseChanged(chat, "a1", "calling_tool", "web_search"),
            decodeEvent(chat, obj("""{"kind":"AgentPhaseChanged","agent_session_id":"a1","phase":"calling_tool","detail":"web_search"}""")),
        )
        assertEquals(
            RealtimeEvent.AgentPhaseChanged(chat, "a1", "thinking", null),
            decodeEvent(chat, obj("""{"kind":"AgentPhaseChanged","agent_session_id":"a1","phase":"thinking","detail":null}""")),
        )
        assertEquals(
            RealtimeEvent.AgentSessionStarted(chat, "a1", "s1", "coder", "Ada", "web", "Trian"),
            decodeEvent(
                chat,
                obj("""{"kind":"AgentSessionStarted","agent_session_id":"a1","session_id":"s1","agent_type":"coder","agent_display_name":"Ada","channel":"web","sender_label":"Trian"}"""),
            ),
        )
        assertEquals(
            RealtimeEvent.AgentSessionEnded(chat, "a1", "s1", "completed"),
            decodeEvent(chat, obj("""{"kind":"AgentSessionEnded","agent_session_id":"a1","session_id":"s1","reason":"completed"}""")),
        )
    }

    @Test
    fun userEvents() {
        assertEquals(
            RealtimeEvent.NotificationCreated(Topic.User, "n1", "deploy", "T", "B", "/x"),
            decodeEvent(Topic.User, obj("""{"kind":"NotificationCreated","notification_id":"n1","category":"deploy","title":"T","body":"B","link":"/x"}""")),
        )
        assertEquals(
            RealtimeEvent.NotificationCreated(Topic.User, "n1", "quota", "T", "B", null),
            decodeEvent(Topic.User, obj("""{"kind":"NotificationCreated","notification_id":"n1","category":"quota","title":"T","body":"B","link":null}""")),
        )
        assertEquals(
            RealtimeEvent.ChatReply(Topic.User, "s9", "m9", "hello"),
            decodeEvent(Topic.User, obj("""{"kind":"ChatReply","session_id":"s9","message_id":"m9","preview":"hello"}""")),
        )
    }

    @Test
    fun chatActivity() {
        assertEquals(
            RealtimeEvent.ChatActivity(Topic.User, "s1", "Trip", null, "waiting", "question", "Nomi"),
            decodeEvent(
                Topic.User,
                obj("""{"kind":"ChatActivity","session_id":"s1","title":"Trip","project_id":null,"outcome":"waiting","waiting":"question","agent":"Nomi"}"""),
            ),
        )
        assertEquals(
            RealtimeEvent.ChatActivity(Topic.User, "s2", null, "p1", "done", null, null),
            decodeEvent(Topic.User, obj("""{"kind":"ChatActivity","session_id":"s2","title":null,"project_id":"p1","outcome":"done","waiting":null,"agent":null}""")),
        )
        // A user-topic event arriving on a chat topic is malformed.
        assertEquals("ChatActivity", (decodeEvent(Topic.Chat("s1"), obj("""{"kind":"ChatActivity","session_id":"s1","outcome":"done"}""")) as RealtimeEvent.Unknown).kind)
    }

    @Test
    fun projectEvent() {
        val t = Topic.Project("p1")
        assertEquals(RealtimeEvent.DeployChanged(t, "p1", "publish.succeeded"), decodeEvent(t, obj("""{"kind":"DeployChanged","event":"publish.succeeded"}""")))
    }

    @Test
    fun unknownKind() {
        val raw = obj("""{"kind":"Mystery","x":1}""")
        assertEquals(RealtimeEvent.Unknown(chat, "Mystery", raw), decodeEvent(chat, raw))
    }

    @Test
    fun missingKind() {
        val raw = obj("""{"x":1}""")
        assertEquals(RealtimeEvent.Unknown(chat, null, raw), decodeEvent(chat, raw))
    }

    @Test
    fun malformedPayloadIsUnknown() {
        val raw = obj("""{"kind":"TurnCompleted","turn_job_id":"t1"}""")
        assertEquals(RealtimeEvent.Unknown(chat, "TurnCompleted", raw), decodeEvent(chat, raw))
        val wrongType = obj("""{"kind":"MessageCreated","message_id":{"a":1}}""")
        assertEquals(RealtimeEvent.Unknown(chat, "MessageCreated", wrongType), decodeEvent(chat, wrongType))
    }

    @Test
    fun chatKindOnWrongTopicIsUnknown() {
        val raw = obj("""{"kind":"MessageCreated","message_id":"m1"}""")
        assertEquals(RealtimeEvent.Unknown(Topic.User, "MessageCreated", raw), decodeEvent(Topic.User, raw))
    }

    @Test
    fun syntheticEventsCarryTopic() {
        assertEquals(chat, RealtimeEvent.Resynced(chat).topic)
        assertEquals(chat, RealtimeEvent.AccessLost(chat).topic)
    }
}
