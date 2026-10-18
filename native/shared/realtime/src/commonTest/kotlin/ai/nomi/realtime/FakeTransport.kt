package ai.nomi.realtime

import kotlinx.coroutines.awaitCancellation
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.receiveAsFlow

/**
 * In-memory [RealtimeTransport]. Every [connect] hands out a session reading the "current" frame queue, so frames
 * [push]ed before the hub has connected are delivered once it does. [dropConnection] ends the current socket.
 */
class FakeTransport(private var failWith401Times: Int = 0) : RealtimeTransport {
    /** Thrown, one per attempt and in order, by the next [connect]s (after any 401s). */
    val connectErrors = ArrayDeque<Throwable>()

    /** A dead socket whose close never returns (it only ends when cancelled). */
    var closeHangs = false

    /** Every frame the hub sent, across all sessions. */
    val sent = mutableListOf<ClientFrame>()
    var closedCount = 0
        private set
    var connectCount = 0
        private set

    private var frames = Channel<ServerFrame>(Channel.UNLIMITED)

    override suspend fun connect(): RealtimeSession {
        connectCount++
        if (failWith401Times > 0) {
            failWith401Times--
            throw UnauthorizedException()
        }
        connectErrors.removeFirstOrNull()?.let { throw it }
        val queue = frames
        return object : RealtimeSession {
            override val incoming: Flow<ServerFrame> = queue.receiveAsFlow()
            override suspend fun send(frame: ClientFrame) {
                sent += frame
            }
            override suspend fun close() {
                closedCount++
                endSocket(queue)
                if (closeHangs) awaitCancellation()
            }
        }
    }

    fun push(frame: ServerFrame) {
        frames.trySend(frame)
    }

    /** The server answers the pending connection with `ready`. */
    fun acceptNext(userId: String) = push(ServerFrame.Ready(userId))

    /** The server (or network) closes the current socket. */
    fun dropConnection() = endSocket(frames)

    private fun endSocket(queue: Channel<ServerFrame>) {
        queue.close()
        if (queue === frames) frames = Channel(Channel.UNLIMITED)
    }
}
