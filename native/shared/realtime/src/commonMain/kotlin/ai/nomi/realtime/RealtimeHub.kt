package ai.nomi.realtime

import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.NonCancellable
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.buffer
import kotlinx.coroutines.flow.callbackFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import kotlinx.coroutines.withTimeoutOrNull
import kotlin.concurrent.Volatile

/** Receives realtime events for one topic, on the hub's dispatcher (Main in the apps). */
fun interface RealtimeListener {
    fun onEvent(event: RealtimeEvent)
}

/** Returned by [RealtimeHub.addListener]; [remove] is idempotent. */
interface ListenerHandle {
    fun remove()
}

sealed interface ConnectionState {
    /** Signed out / stopped. */
    data object Idle : ConnectionState
    data object Connecting : ConnectionState
    data class Connected(val userId: String) : ConnectionState
    data class Waiting(val retryInMs: Long, val attempt: Int) : ConnectionState
}

/**
 * The app's single realtime socket. Screens and ViewModels register listeners per [Topic]; the hub ref-counts them
 * into server subscriptions, survives reconnects and tells listeners to refetch with [RealtimeEvent.Resynced].
 */
interface RealtimeHub {
    val state: StateFlow<ConnectionState>

    /** Idempotent; called on sign-in / app start. */
    fun start()

    /** Sign-out: closes the socket, which drops every subscription server-side. Listeners stay registered. */
    fun stop()

    fun addListener(topic: Topic, listener: RealtimeListener): ListenerHandle

    /** Cold: adds a listener when collected, removes it when the collector is cancelled. */
    fun events(topic: Topic): Flow<RealtimeEvent>
}

/**
 * [RealtimeHub] as an actor: every piece of mutable state below is touched only by the coroutine draining
 * [commands]. Public methods and the connection/timer coroutines only enqueue [Command]s. Connection attempts carry
 * a [generation]; commands from a superseded attempt are ignored, so a stop/restart can never be confused by a
 * late frame or timer.
 *
 * Listener dispatch happens on the actor, i.e. on [scope]'s dispatcher (the apps pass a Main-dispatcher scope).
 */
class DefaultRealtimeHub(
    private val transport: RealtimeTransport,
    private val refreshToken: suspend () -> Boolean,
    private val scope: CoroutineScope,
    private val backoff: Backoff = Backoff(),
    private val log: (String) -> Unit = {},
) : RealtimeHub {

    private val commands = Channel<Command>(Channel.UNLIMITED)
    private val mutableState = MutableStateFlow<ConnectionState>(ConnectionState.Idle)
    override val state: StateFlow<ConnectionState> = mutableState.asStateFlow()

    // ---- Actor-confined state ----
    private val registrations = LinkedHashMap<Topic, MutableList<Registration>>()
    private val denied = HashSet<Topic>()
    private var running = false
    private var generation = 0
    private var ready = false
    private var session: RealtimeSession? = null
    private var connectionJob: Job? = null
    private var timerJob: Job? = null
    private var failures = 0
    private var refreshedSinceReady = false
    private var connectedBefore = false

    init {
        scope.launch {
            for (command in commands) {
                try {
                    handle(command)
                } catch (e: CancellationException) {
                    throw e
                } catch (e: Exception) {
                    log("realtime: hub failed handling $command: $e")
                }
            }
        }
    }

    override fun start() = post(Command.Start)

    override fun stop() = post(Command.Stop)

    override fun addListener(topic: Topic, listener: RealtimeListener): ListenerHandle =
        Registration(topic, listener).also { post(Command.Add(it)) }

    override fun events(topic: Topic): Flow<RealtimeEvent> = callbackFlow {
        val handle = addListener(topic) { trySend(it) }
        awaitClose { handle.remove() }
    }.buffer(Channel.UNLIMITED)

    private fun post(command: Command) {
        commands.trySend(command)
    }

    private inner class Registration(val topic: Topic, val listener: RealtimeListener) : ListenerHandle {
        /** Set on the caller's thread so no event reaches a removed listener while its Remove is queued. */
        @Volatile var removed = false

        override fun remove() {
            if (removed) return
            removed = true
            post(Command.Remove(this))
        }
    }

    private sealed interface Command {
        data object Start : Command
        data object Stop : Command
        class Add(val registration: DefaultRealtimeHub.Registration) : Command
        class Remove(val registration: DefaultRealtimeHub.Registration) : Command
        class Opened(val generation: Int, val session: RealtimeSession) : Command
        class Received(val generation: Int, val frame: ServerFrame) : Command
        class Closed(val generation: Int) : Command
        class ConnectFailed(val generation: Int, val cause: Throwable) : Command
        class Unauthorized(val generation: Int) : Command
        /** `ok == null`: the refresh call itself failed (network), not a verdict on the session. */
        class Refreshed(val generation: Int, val ok: Boolean?) : Command
        class Retry(val generation: Int) : Command
        class Stable(val generation: Int) : Command
    }

    // ---- Actor ----

    private suspend fun handle(command: Command) {
        when (command) {
            Command.Start -> if (!running) {
                running = true
                connect()
            }
            Command.Stop -> stopInternal()
            is Command.Add -> onAdd(command.registration)
            is Command.Remove -> onRemove(command.registration)
            is Command.Opened -> if (command.generation == generation) session = command.session
            is Command.Received -> if (command.generation == generation) onFrame(command.frame)
            is Command.Closed -> if (command.generation == generation) onDisconnected()
            is Command.ConnectFailed -> if (command.generation == generation) {
                log("realtime: connect failed: ${command.cause}")
                onDisconnected()
            }
            is Command.Unauthorized -> if (command.generation == generation) onUnauthorized()
            is Command.Refreshed -> if (command.generation == generation) onRefreshed(command.ok)
            is Command.Retry -> if (command.generation == generation && running) connect()
            is Command.Stable -> if (command.generation == generation && ready) failures = 0
        }
    }

    private suspend fun onAdd(registration: Registration) {
        if (registration.removed) return
        val topic = registration.topic
        val list = registrations.getOrPut(topic) { mutableListOf() }
        val first = list.isEmpty()
        list += registration
        val wasDenied = denied.remove(topic)
        if (first || wasDenied) sendSubscription(topic, subscribe = true)
    }

    private suspend fun onRemove(registration: Registration) {
        val topic = registration.topic
        val list = registrations[topic] ?: return
        if (!list.remove(registration) || list.isNotEmpty()) return
        registrations.remove(topic)
        // A denied topic has no server-side subscription left to drop.
        if (!denied.remove(topic)) sendSubscription(topic, subscribe = false)
    }

    /** `User` is implicit on the gateway; before `ready` nothing is sent, the `ready` handler subscribes everything. */
    private suspend fun sendSubscription(topic: Topic, subscribe: Boolean) {
        if (topic == Topic.User || !ready) return
        send(if (subscribe) ClientFrame.Subscribe(topic.wire) else ClientFrame.Unsubscribe(topic.wire))
    }

    private suspend fun send(frame: ClientFrame) {
        val current = session ?: return
        try {
            current.send(frame)
        } catch (e: CancellationException) {
            throw e
        } catch (e: Exception) {
            // The socket is going away; its Closed command will run the reconnect path.
            log("realtime: send $frame failed: $e")
        }
    }

    private suspend fun onFrame(frame: ServerFrame) {
        when (frame) {
            is ServerFrame.Ready -> onReady(frame.userId)
            is ServerFrame.Event -> {
                val topic = Topic.parse(frame.topic) ?: return log("realtime: event on unknown topic '${frame.topic}'")
                dispatch(decodeEvent(topic, frame.payload))
            }
            is ServerFrame.Denied -> {
                val topic = Topic.parse(frame.topic) ?: return log("realtime: denied unknown topic '${frame.topic}'")
                if (topic !in registrations) return
                denied += topic
                dispatch(RealtimeEvent.AccessLost(topic))
            }
            is ServerFrame.Error -> log("realtime: server error: ${frame.message}")
            is ServerFrame.Subscribed, is ServerFrame.Unsubscribed, ServerFrame.Pong -> Unit
        }
    }

    private suspend fun onReady(userId: String) {
        if (ready) return
        ready = true
        refreshedSinceReady = false
        mutableState.value = ConnectionState.Connected(userId)
        val gen = generation
        timerJob?.cancel()
        timerJob = scope.launch {
            delay(STABLE_AFTER_MS)
            post(Command.Stable(gen))
        }
        val live = registrations.keys.filter { it !in denied }
        live.forEach { sendSubscription(it, subscribe = true) }
        if (connectedBefore) live.forEach { dispatch(RealtimeEvent.Resynced(it)) }
        connectedBefore = true
    }

    private fun dispatch(event: RealtimeEvent) {
        val listeners = registrations[event.topic]?.toList() ?: return
        for (registration in listeners) {
            if (registration.removed) continue
            try {
                registration.listener.onEvent(event)
            } catch (e: Exception) {
                log("realtime: listener for ${event.topic.wire} threw: $e")
            }
        }
    }

    private fun connect() {
        resetConnection()
        mutableState.value = ConnectionState.Connecting
        val gen = generation
        connectionJob = scope.launch { runConnection(gen) }
    }

    private fun onDisconnected() {
        resetConnection()
        if (!running) return
        val delayMs = backoff.delayFor(failures)
        failures++
        mutableState.value = ConnectionState.Waiting(delayMs, failures)
        val gen = generation
        timerJob = scope.launch {
            delay(delayMs)
            post(Command.Retry(gen))
        }
    }

    private fun onUnauthorized() {
        if (refreshedSinceReady) {
            log("realtime: still unauthorized after a token refresh; stopping")
            stopInternal()
            return
        }
        refreshedSinceReady = true
        val gen = generation
        timerJob?.cancel()
        timerJob = scope.launch {
            val ok = try {
                refreshToken()
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                log("realtime: token refresh failed: $e")
                null
            }
            post(Command.Refreshed(gen, ok))
        }
    }

    private fun onRefreshed(ok: Boolean?) {
        when (ok) {
            true -> connect()
            false -> stopInternal()
            null -> {
                refreshedSinceReady = false
                onDisconnected()
            }
        }
    }

    private fun stopInternal() {
        running = false
        resetConnection()
        failures = 0
        refreshedSinceReady = false
        // connectedBefore survives a stop: listeners still registered across stop/start missed events meanwhile.
        denied.clear()
        mutableState.value = ConnectionState.Idle
    }

    /** Ends the current attempt (its coroutine closes the socket) and invalidates its pending commands. */
    private fun resetConnection() {
        generation++
        connectionJob?.cancel()
        connectionJob = null
        timerJob?.cancel()
        timerJob = null
        session = null
        ready = false
    }

    // ---- Connection coroutine: owns one socket, never touches hub state, only posts commands ----

    private suspend fun runConnection(gen: Int) {
        val opened = try {
            transport.connect()
        } catch (e: CancellationException) {
            throw e
        } catch (_: UnauthorizedException) {
            post(Command.Unauthorized(gen))
            return
        } catch (e: Exception) {
            post(Command.ConnectFailed(gen, e))
            return
        }
        try {
            post(Command.Opened(gen, opened))
            pump(gen, opened)
        } finally {
            withContext(NonCancellable) {
                try {
                    opened.close()
                } catch (e: Exception) {
                    log("realtime: close failed: $e")
                }
            }
        }
        post(Command.Closed(gen))
    }

    /** Reads frames until the socket closes or goes [STALE_AFTER_MS] without a frame, pinging meanwhile. */
    private suspend fun pump(gen: Int, socket: RealtimeSession) = coroutineScope {
        val activity = Channel<Unit>(Channel.CONFLATED)
        val reader = launch {
            try {
                socket.incoming.collect { frame ->
                    activity.trySend(Unit)
                    post(Command.Received(gen, frame))
                }
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                log("realtime: socket error: $e")
            }
        }
        val pinger = launch {
            while (true) {
                delay(PING_INTERVAL_MS)
                try {
                    socket.send(ClientFrame.Ping)
                } catch (e: CancellationException) {
                    throw e
                } catch (e: Exception) {
                    log("realtime: ping failed: $e")
                }
            }
        }
        val watchdog = launch {
            while (true) {
                withTimeoutOrNull(STALE_AFTER_MS) { activity.receive() } ?: break
            }
            log("realtime: no frame for ${STALE_AFTER_MS}ms; recycling socket")
            reader.cancel()
        }
        reader.join()
        pinger.cancel()
        watchdog.cancel()
    }

    companion object {
        /** Client-level `ping` cadence; the gateway answers `pong`, which also feeds the stale watchdog. */
        const val PING_INTERVAL_MS: Long = 25_000L

        /** A socket with no frame for this long is considered dead and recycled. */
        const val STALE_AFTER_MS: Long = 60_000L

        /** A connection that stays `ready` this long resets the backoff attempt counter. */
        const val STABLE_AFTER_MS: Long = 10_000L
    }
}
