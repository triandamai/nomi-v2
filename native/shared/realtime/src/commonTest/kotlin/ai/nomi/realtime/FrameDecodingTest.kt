package ai.nomi.realtime

import io.ktor.websocket.Frame
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.toList
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull
import kotlin.test.assertTrue

/** R18: a frame the app cannot decode is dropped and logged; the stream (and so the socket) carries on. */
class FrameDecodingTest {
    @Test fun undecodable_frames_are_null_and_logged() {
        val logs = mutableListOf<String>()
        assertNull(decodeServerFrame("not json", logs::add))
        assertNull(decodeServerFrame("""{"type":"mystery"}""", logs::add))
        assertNull(decodeServerFrame("""{"type":"event","topic":"chat:s1","payload":[1,2]}""", logs::add))
        assertNull(decodeServerFrame("""[1,2]""", logs::add))
        assertEquals(4, logs.size)
    }

    @Test fun valid_frame_decodes() {
        assertEquals(ServerFrame.Ready("u1"), decodeServerFrame("""{"type":"ready","user_id":"u1"}"""))
    }

    @Test fun stream_skips_garbage_and_keeps_going() = runTest {
        val logs = mutableListOf<String>()
        val frames = flowOf<Frame>(
            Frame.Text("garbage"),
            Frame.Binary(true, byteArrayOf(1)),
            Frame.Text("""{"type":"unknown_type"}"""),
            Frame.Text("""{"type":"ready","user_id":"u1"}"""),
            Frame.Text("""{"type":"pong"}"""),
        ).toServerFrames(logs::add).toList()
        assertEquals(listOf(ServerFrame.Ready("u1"), ServerFrame.Pong), frames)
        assertTrue(logs.size >= 2)
    }
}
