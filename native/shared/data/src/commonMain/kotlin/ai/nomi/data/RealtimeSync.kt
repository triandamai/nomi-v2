package ai.nomi.data

import ai.nomi.sdk.RealtimeEvent
import ai.nomi.sdk.SdkEvent
import kotlinx.coroutines.Job
import kotlinx.coroutines.channels.BufferOverflow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

/** Signals kept for a slow companion before the oldest is dropped (they are transient UI cues). */
private const val SIGNAL_BUFFER = 16
private const val USER_TOPIC = "user"

/** [RealtimeEvent.ChatActivity.outcome] values. */
internal const val OUTCOME_DONE = "done"
internal const val OUTCOME_FAILED = "failed"
internal const val OUTCOME_WAITING = "waiting"

/**
 * What the island / chat head show, from the Rust engine, started once after sign-in. The engine itself writes
 * notifications and chat replies to the store whether or not their screen is open; this only turns them into
 * companion cues: [companionState] (Working/Idle, always current) and [companionSignals] (one-off events).
 *
 * The user topic also carries the server's chat activity: when the crew finishes, fails or waits on the person in a
 * chat not on screen, that chat gets one [CompanionSignal.NeedsYou] / [CompanionSignal.ChatFailed] /
 * [CompanionSignal.ChatDone].
 *
 * Limitation: the user topic carries no agent phases. [companionState] comes from [ActivityRepository], which only
 * chats kept in sync (open screens) feed, so work in a chat that isn't open doesn't show as Working; its reply still
 * arrives as [CompanionSignal.NewReply].
 */
class RealtimeSync(
    private val engine: NomiEngine,
    private val activity: ActivityRepository,
    private val foreground: ForegroundChat,
    private val session: SessionScope,
    private val log: (String) -> Unit = {},
) {
    private val signals = MutableSharedFlow<CompanionSignal>(
        extraBufferCapacity = SIGNAL_BUFFER,
        onBufferOverflow = BufferOverflow.DROP_OLDEST,
    )

    /**
     * Transient [CompanionSignal.NewReply] / [CompanionSignal.Notification] events. No replay: an event emitted while
     * nothing collects is dropped by design (the data itself is in the store; only the pop-up is missed).
     */
    val companionSignals: SharedFlow<CompanionSignal> = signals.asSharedFlow()

    private val state = MutableStateFlow<CompanionSignal>(CompanionSignal.Idle)

    /** [CompanionSignal.Working] or [CompanionSignal.Idle], from agent activity; late collectors get the current value. */
    val companionState: StateFlow<CompanionSignal> = state.asStateFlow()

    /**
     * Chats that got a [CompanionSignal.NewReply] since their last chat activity. A turn's reply and the chat's
     * "done" both arrive when the crew finishes; the reply already told the person, so "done" stays quiet for them.
     * Only touched by the event collector, one event at a time.
     */
    private val repliedSinceActivity = mutableSetOf<String>()

    private var jobs: List<Job> = emptyList()

    /** Idempotent. Call from the main thread, like [stop]. */
    fun start() {
        if (jobs.isNotEmpty()) return
        jobs = listOf(
            session.scope.launch {
                engine.get().events.collect { event ->
                    if (event is SdkEvent.Realtime && event.topic == USER_TOPIC) handle(event.event)
                }
            },
            session.scope.launch {
                activity.observeActive().collect { active ->
                    state.value = active.firstOrNull()?.let { CompanionSignal.Working(it.agentType, it.phase) }
                        ?: CompanionSignal.Idle
                }
            },
        )
    }

    /** Sign-out: stops listening. Idempotent. */
    fun stop() {
        jobs.forEach { it.cancel() }
        jobs = emptyList()
        repliedSinceActivity.clear()
        state.value = CompanionSignal.Idle
    }

    private suspend fun handle(event: RealtimeEvent) {
        when (event) {
            is RealtimeEvent.NotificationCreated ->
                signals.emit(CompanionSignal.Notification(event.title, event.body, event.link))
            is RealtimeEvent.ChatReply -> {
                if (!isOnScreen(event.sessionId)) {
                    signals.emit(CompanionSignal.NewReply(event.sessionId, event.preview, agentType = null))
                }
                repliedSinceActivity += event.sessionId
            }
            is RealtimeEvent.ChatActivity -> onChatActivity(event)
            else -> Unit
        }
    }

    /** A chat that is not on screen gets one pop-up, unless it is "done" and a reply already announced it. */
    private suspend fun onChatActivity(event: RealtimeEvent.ChatActivity) {
        val alreadyReplied = repliedSinceActivity.remove(event.sessionId)
        if (isOnScreen(event.sessionId)) return
        val signal = when (event.outcome) {
            OUTCOME_WAITING -> event.waiting?.let { CompanionSignal.NeedsYou(event.sessionId, event.title, it) }
            OUTCOME_FAILED -> CompanionSignal.ChatFailed(event.sessionId, event.title)
            OUTCOME_DONE -> if (alreadyReplied) null else CompanionSignal.ChatDone(event.sessionId, event.title)
            else -> {
                log("data: unknown chat outcome ${event.outcome}")
                null
            }
        }
        signal?.let { signals.emit(it) }
    }

    private fun isOnScreen(sessionId: String) = foreground.sessionId.value?.lowercase() == sessionId
}
