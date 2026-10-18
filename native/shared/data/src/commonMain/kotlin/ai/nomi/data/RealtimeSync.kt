package ai.nomi.data

import ai.nomi.realtime.ListenerHandle
import ai.nomi.realtime.RealtimeEvent
import ai.nomi.realtime.RealtimeHub
import ai.nomi.realtime.Topic
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.channels.BufferOverflow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.launch

/** Signals kept for a slow companion before the oldest is dropped (they are transient UI cues). */
private const val SIGNAL_BUFFER = 16

/**
 * The app-wide listener on [Topic.User], started once after sign-in: notifications and chat replies land in Room
 * whether or not their screen is open, and [companionSignals] tells the island / chat head what to show.
 *
 * Limitation: the user topic carries no agent phases. [CompanionSignal.Working]/[CompanionSignal.Idle] are derived
 * from [ActivityRepository], which only chats bound by [ChatRepository] (open screens) feed, so work in a chat that
 * is not open does not show as Working; its reply still arrives as [CompanionSignal.NewReply].
 */
class RealtimeSync(
    private val hub: RealtimeHub,
    private val sessions: SessionRepository,
    private val notifications: NotificationRepository,
    private val activity: ActivityRepository,
    private val foreground: ForegroundChat,
    private val scope: CoroutineScope,
    private val log: (String) -> Unit = {},
) {
    private val signals = MutableSharedFlow<CompanionSignal>(
        extraBufferCapacity = SIGNAL_BUFFER,
        onBufferOverflow = BufferOverflow.DROP_OLDEST,
    )

    /** Hot, no replay: a companion sees what happens while it is collecting. */
    val companionSignals: SharedFlow<CompanionSignal> = signals.asSharedFlow()

    private var registration: ListenerHandle? = null
    private var queue: WorkQueue? = null
    private var activityJob: Job? = null

    /** Idempotent. Call from the main thread, like [stop]. */
    fun start() {
        if (registration != null) return
        val work = WorkQueue(scope, log)
        queue = work
        registration = hub.addListener(Topic.User) { event -> work.submit { handle(event) } }
        activityJob = scope.launch {
            activity.observeActive()
                .map { active -> active.firstOrNull()?.let { CompanionSignal.Working(it.agentType, it.phase) } ?: CompanionSignal.Idle }
                .distinctUntilChanged()
                .collect { signals.emit(it) }
        }
    }

    /** Sign-out: stops listening. Idempotent. */
    fun stop() {
        registration?.remove()
        registration = null
        queue?.close()
        queue = null
        activityJob?.cancel()
        activityJob = null
    }

    private suspend fun handle(event: RealtimeEvent) {
        when (event) {
            is RealtimeEvent.NotificationCreated -> {
                notifications.onCreated(event)
                signals.emit(CompanionSignal.Notification(event.title, event.body, event.link))
            }
            is RealtimeEvent.ChatReply -> {
                val onScreen = foreground.sessionId.value == event.sessionId
                sessions.onReply(event.sessionId, event.preview, unread = !onScreen)
                if (!onScreen) signals.emit(CompanionSignal.NewReply(event.sessionId, event.preview, agentType = null))
            }
            is RealtimeEvent.Resynced -> {
                attempt("refresh sessions") { sessions.refresh() }
                attempt("refresh notifications") { notifications.refresh() }
            }
            else -> Unit
        }
    }

    /** One failed refetch must not skip the other. */
    private suspend fun attempt(what: String, block: suspend () -> Unit) {
        try {
            block()
        } catch (e: CancellationException) {
            throw e
        } catch (e: Exception) {
            log("data: $what failed: $e")
        }
    }
}
