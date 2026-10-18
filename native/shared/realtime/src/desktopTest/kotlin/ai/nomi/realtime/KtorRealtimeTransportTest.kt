package ai.nomi.realtime

import ai.nomi.network.ApiConfig
import ai.nomi.network.TokenStore
import ai.nomi.network.Tokens
import ai.nomi.network.nomiHttpClient
import io.ktor.client.engine.okhttp.OkHttp
import io.ktor.client.plugins.websocket.WebSocketException
import kotlinx.coroutines.flow.take
import kotlinx.coroutines.flow.toList
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withTimeout
import java.io.BufferedReader
import java.io.InputStreamReader
import java.net.ServerSocket
import java.net.Socket
import java.security.MessageDigest
import java.util.Base64
import java.util.concurrent.CopyOnWriteArrayList
import kotlin.concurrent.thread
import kotlin.test.AfterTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertTrue

/**
 * Real OkHttp engine against a minimal JDK socket server, to pin down how Ktor 3.6 reports a rejected WebSocket
 * upgrade and that the upgrade carries the bearer token from `nomiHttpClient`.
 */
class KtorRealtimeTransportTest {
    private class MemoryTokens(var tokens: Tokens?) : TokenStore {
        override suspend fun read() = tokens
        override suspend fun write(tokens: Tokens) { this.tokens = tokens }
        override suspend fun clear() { tokens = null }
    }

    /** Request line + headers of every request the server saw. */
    private val requests = CopyOnWriteArrayList<List<String>>()
    private val server = ServerSocket(0)
    private val baseUrl = "http://127.0.0.1:${server.localPort}"

    @AfterTest fun tearDown() = server.close()

    private fun serve(handler: (request: List<String>, socket: Socket) -> Unit) {
        thread(isDaemon = true) {
            while (!server.isClosed) {
                val socket = try { server.accept() } catch (_: Exception) { return@thread }
                thread(isDaemon = true) {
                    socket.use {
                        val reader = BufferedReader(InputStreamReader(it.getInputStream(), Charsets.ISO_8859_1))
                        val head = generateSequence { reader.readLine() }.takeWhile { line -> line.isNotEmpty() }.toList()
                        requests += head
                        handler(head, it)
                    }
                }
            }
        }
    }

    private fun Socket.respond(raw: String) {
        getOutputStream().apply { write(raw.toByteArray(Charsets.ISO_8859_1)); flush() }
    }

    private fun client(tokens: TokenStore) = nomiHttpClient(OkHttp.create(), ApiConfig(baseUrl), tokens) {}

    @Test fun unauthorized_upgrade_surfaces_as_UnauthorizedException_with_bearer_attached() = runBlocking {
        serve { _, socket -> socket.respond("HTTP/1.1 401 Unauthorized\r\nContent-Length: 0\r\nConnection: close\r\n\r\n") }
        val transport = KtorRealtimeTransport(client(MemoryTokens(Tokens("a1", "r1"))), ApiConfig(baseUrl))

        val error = assertFailsWith<UnauthorizedException> { withTimeout(TIMEOUT_MS) { transport.connect() } }

        // What Ktor 3.6 + OkHttp actually throws for a non-101 upgrade (the response is not attached).
        val handshake = generateSequence(error.cause) { it.cause }.filterIsInstance<WebSocketException>().first()
        assertEquals("Handshake exception, expected status code 101 but was 401", handshake.message)
        // The Auth plugin tried its own refresh before the 401 reached the transport.
        assertTrue(requests.any { it.first().startsWith("POST /api/auth/refresh") })
        val upgrade = requests.first { it.first().startsWith("GET $REALTIME_WS_PATH") }
        assertTrue(upgrade.any { it.equals("Authorization: Bearer a1", ignoreCase = true) }, "headers: $upgrade")
    }

    @Test fun other_handshake_failures_are_not_unauthorized() = runBlocking {
        serve { _, socket -> socket.respond("HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\nConnection: close\r\n\r\n") }
        val transport = KtorRealtimeTransport(client(MemoryTokens(Tokens("a1", "r1"))), ApiConfig(baseUrl))

        val error = assertFailsWith<Exception> { withTimeout(TIMEOUT_MS) { transport.connect() } }
        assertTrue(error !is UnauthorizedException, "got $error")
    }

    @Test fun open_socket_decodes_frames_and_skips_garbage() = runBlocking {
        serve { head, socket ->
            val key = head.first { it.startsWith("Sec-WebSocket-Key:", ignoreCase = true) }.substringAfter(':').trim()
            val accept = Base64.getEncoder().encodeToString(
                MessageDigest.getInstance("SHA-1").digest((key + WS_GUID).toByteArray(Charsets.ISO_8859_1)),
            )
            socket.respond(
                "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\n" +
                    "Sec-WebSocket-Accept: $accept\r\n\r\n",
            )
            listOf("garbage", """{"type":"mystery"}""", """{"type":"ready","user_id":"u1"}""", """{"type":"pong"}""")
                .forEach { socket.getOutputStream().write(textFrame(it)) }
            socket.getOutputStream().flush()
            Thread.sleep(SERVER_LINGER_MS)
        }
        val transport = KtorRealtimeTransport(client(MemoryTokens(Tokens("a1", "r1"))), ApiConfig(baseUrl))

        val frames = withTimeout(TIMEOUT_MS) {
            val session = transport.connect()
            session.incoming.take(2).toList().also { session.close() }
        }
        assertEquals(listOf(ServerFrame.Ready("u1"), ServerFrame.Pong), frames)
    }

    @Test fun token_expired_close_ends_the_stream_like_any_close() = runBlocking {
        serve { head, socket ->
            val key = head.first { it.startsWith("Sec-WebSocket-Key:", ignoreCase = true) }.substringAfter(':').trim()
            val accept = Base64.getEncoder().encodeToString(
                MessageDigest.getInstance("SHA-1").digest((key + WS_GUID).toByteArray(Charsets.ISO_8859_1)),
            )
            socket.respond(
                "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\n" +
                    "Sec-WebSocket-Accept: $accept\r\n\r\n",
            )
            socket.getOutputStream().write(textFrame("""{"type":"ready","user_id":"u1"}"""))
            socket.getOutputStream().write(closeFrame(CLOSE_TOKEN_EXPIRED, "token expired"))
            socket.getOutputStream().flush()
            Thread.sleep(SERVER_LINGER_MS)
        }
        val transport = KtorRealtimeTransport(client(MemoryTokens(Tokens("a1", "r1"))), ApiConfig(baseUrl))

        // The stream completes normally, so the hub takes its ordinary reconnect path (and the Auth plugin's
        // refresh-and-retry supplies a fresh token for the next upgrade).
        val frames = withTimeout(TIMEOUT_MS) {
            val session = transport.connect()
            session.incoming.toList().also { session.close() }
        }
        assertEquals(listOf<ServerFrame>(ServerFrame.Ready("u1")), frames)
    }

    /** An unmasked server-to-client close frame carrying [code] and [reason]. */
    private fun closeFrame(code: Int, reason: String): ByteArray {
        val payload = byteArrayOf((code shr BYTE_BITS).toByte(), code.toByte()) + reason.toByteArray(Charsets.UTF_8)
        return byteArrayOf(FIN_CLOSE_OPCODE.toByte(), payload.size.toByte()) + payload
    }

    /** An unmasked server-to-client text frame (payloads here are < 126 bytes). */
    private fun textFrame(text: String): ByteArray {
        val payload = text.toByteArray(Charsets.UTF_8)
        return byteArrayOf(FIN_TEXT_OPCODE.toByte(), payload.size.toByte()) + payload
    }

    private companion object {
        const val TIMEOUT_MS = 10_000L
        const val SERVER_LINGER_MS = 2_000L
        const val WS_GUID = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11"
        const val FIN_TEXT_OPCODE = 0x81
        const val FIN_CLOSE_OPCODE = 0x88
        const val BYTE_BITS = 8

        /** The gateway's close code for a socket whose access token expired (routes/realtime.rs). */
        const val CLOSE_TOKEN_EXPIRED = 4001
    }
}
