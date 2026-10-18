package ai.nomi.realtime

import kotlinx.coroutines.flow.Flow

/** The realtime upgrade was rejected with 401: the access token is missing or expired. */
class UnauthorizedException(message: String = "Realtime upgrade rejected: 401 Unauthorized", cause: Throwable? = null) :
    Exception(message, cause)

/** One connection attempt; the hub owns retries. */
interface RealtimeTransport {
    /** Opens a socket. Throws [UnauthorizedException] on a 401 upgrade, anything else on other failures. */
    suspend fun connect(): RealtimeSession
}

/** One open socket. */
interface RealtimeSession {
    /** Decoded server frames; completes (or throws) when the socket closes. Collect it once. */
    val incoming: Flow<ServerFrame>

    suspend fun send(frame: ClientFrame)

    suspend fun close()
}
