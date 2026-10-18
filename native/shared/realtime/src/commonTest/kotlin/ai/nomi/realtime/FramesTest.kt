package ai.nomi.realtime

import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.jsonObject
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertNull

class FramesTest {
    private fun obj(s: String): JsonObject = WireJson.parseToJsonElement(s).jsonObject

    @Test
    fun topicParse() {
        assertEquals(Topic.User, Topic.parse("user"))
        assertEquals(Topic.Chat("abc"), Topic.parse("chat:abc"))
        assertEquals(Topic.Project("p1"), Topic.parse("project:p1"))
        assertNull(Topic.parse("x"))
        assertNull(Topic.parse("chat:"))
        assertNull(Topic.parse("other:abc"))
        assertNull(Topic.parse(""))
        assertEquals("chat:abc", Topic.Chat("abc").wire)
        assertEquals("user", Topic.User.wire)
        assertEquals("project:p1", Topic.Project("p1").wire)
    }

    @Test
    fun clientFramesEncodeExactly() {
        assertEquals("""{"op":"subscribe","topic":"chat:s1"}""", WireJson.encodeToString<ClientFrame>(ClientFrame.Subscribe("chat:s1")))
        assertEquals("""{"op":"unsubscribe","topic":"project:p"}""", WireJson.encodeToString<ClientFrame>(ClientFrame.Unsubscribe("project:p")))
        assertEquals("""{"op":"ping"}""", WireJson.encodeToString<ClientFrame>(ClientFrame.Ping))
    }

    @Test
    fun serverFramesDecode() {
        fun dec(s: String) = WireJson.decodeFromString<ServerFrame>(s)
        assertEquals(ServerFrame.Ready("u1"), dec("""{"type":"ready","user_id":"u1"}"""))
        assertEquals(ServerFrame.Subscribed("chat:a"), dec("""{"type":"subscribed","topic":"chat:a"}"""))
        assertEquals(ServerFrame.Unsubscribed("chat:a"), dec("""{"type":"unsubscribed","topic":"chat:a"}"""))
        assertEquals(ServerFrame.Denied("chat:a"), dec("""{"type":"denied","topic":"chat:a"}"""))
        assertEquals(ServerFrame.Error("boom"), dec("""{"type":"error","message":"boom","extra":1}"""))
        assertEquals(ServerFrame.Pong, dec("""{"type":"pong"}"""))
        val ev = dec("""{"type":"event","topic":"user","payload":{"kind":"PersonalityChanged"}}""")
        assertEquals(ServerFrame.Event("user", obj("""{"kind":"PersonalityChanged"}""")), ev)
    }
}
