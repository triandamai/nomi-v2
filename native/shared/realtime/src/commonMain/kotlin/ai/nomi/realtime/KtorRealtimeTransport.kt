package ai.nomi.realtime

import ai.nomi.network.ApiConfig
import io.ktor.client.HttpClient
import io.ktor.client.plugins.ResponseException
import io.ktor.client.plugins.websocket.DefaultClientWebSocketSession
import io.ktor.client.plugins.websocket.WebSocketException
import io.ktor.client.plugins.websocket.webSocketSession
import io.ktor.http.HttpStatusCode
import io.ktor.websocket.Frame
import io.ktor.websocket.close
import io.ktor.websocket.readText
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.consumeAsFlow
import kotlinx.coroutines.flow.mapNotNull
import kotlinx.serialization.SerializationException

/** The gateway's realtime endpoint, relative to [ApiConfig.baseUrl]. */
const val REALTIME_WS_PATH = "/api/realtime/ws"

/**
 * [RealtimeTransport] over Ktor WebSockets. [client] must be the app's `nomiHttpClient`: the upgrade request then
 * carries the bearer token (the Auth plugin's `sendWithoutRequest` matches the API host) and a 401 on the upgrade
 * first goes through the Auth plugin's own refresh-and-retry.
 *
 * A failed handshake surfaces from Ktor 3.6 as a [WebSocketException] whose message names the status
 * ("Handshake exception, expected status code 101 but was 401"); the response itself is not attached, so the
 * status is read from that message. With `expectSuccess = true` it would be a [ResponseException] instead, which is
 * handled too. Both become [UnauthorizedException] for a 401; every other failure is rethrown as-is.
 */
class KtorRealtimeTransport(
    private val client: HttpClient,
    private val config: ApiConfig,
    private val log: (String) -> Unit = {},
) : RealtimeTransport {
    override suspend fun connect(): RealtimeSession {
        val socket = try {
            client.webSocketSession(config.toWebSocketUrl(REALTIME_WS_PATH))
        } catch (e: ResponseException) {
            if (e.response.status == HttpStatusCode.Unauthorized) throw UnauthorizedException(cause = e)
            throw e
        } catch (e: WebSocketException) {
            if (e.isUnauthorizedHandshake()) throw UnauthorizedException(cause = e)
            throw e
        }
        return KtorRealtimeSession(socket, log)
    }
}

private val HANDSHAKE_STATUS = Regex("""\bwas (\d{3})\b""")

internal fun WebSocketException.isUnauthorizedHandshake(): Boolean =
    message?.let { HANDSHAKE_STATUS.find(it) }?.groupValues?.get(1)?.toIntOrNull() == HttpStatusCode.Unauthorized.value

private class KtorRealtimeSession(
    private val socket: DefaultClientWebSocketSession,
    log: (String) -> Unit,
) : RealtimeSession {
    override val incoming: Flow<ServerFrame> = socket.incoming.consumeAsFlow().toServerFrames(log)

    override suspend fun send(frame: ClientFrame) {
        socket.send(Frame.Text(WireJson.encodeToString(ClientFrame.serializer(), frame)))
    }

    override suspend fun close() {
        socket.close()
    }
}

/** R18: text frames are decoded leniently; anything undecodable (or non-text) is logged and skipped. */
internal fun Flow<Frame>.toServerFrames(log: (String) -> Unit = {}): Flow<ServerFrame> = mapNotNull { frame ->
    when (frame) {
        is Frame.Text -> decodeServerFrame(frame.readText(), log)
        else -> {
            log("realtime: ignoring non-text frame ${frame.frameType}")
            null
        }
    }
}

/** Decodes one server frame, or logs and returns null for garbage, unknown `type`s and malformed payloads. */
internal fun decodeServerFrame(text: String, log: (String) -> Unit = {}): ServerFrame? = try {
    WireJson.decodeFromString(ServerFrame.serializer(), text)
} catch (e: SerializationException) {
    log("realtime: dropping undecodable frame: ${e.message}")
    null
} catch (e: IllegalArgumentException) {
    log("realtime: dropping undecodable frame: ${e.message}")
    null
}
