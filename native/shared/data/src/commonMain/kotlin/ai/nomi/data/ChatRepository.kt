package ai.nomi.data

import ai.nomi.sdk.NomiException
import ai.nomi.sdk.OutboxOperation
import ai.nomi.sdk.RealtimeEvent
import ai.nomi.sdk.SdkEvent
import ai.nomi.sdk.draft
import ai.nomi.sdk.messages
import kotlinx.coroutines.CoroutineStart
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

/** The most messages a chat shows; the engine caps every query here. */
private const val MESSAGE_LIMIT = 500u
private const val CHAT_TOPIC_PREFIX = "chat:"

/**
 * One chat, offline-first: the Rust engine writes REST pages, realtime events and the person's queued messages to
 * its store, and [observe] reads only that.
 *
 * [bind] is how a chat screen goes live: while bound, the engine watches the chat's topic and applies its events.
 * Call [bind] and [ListenerHandle.remove] from one thread (the main thread, like ViewModel init / onCleared).
 */
class ChatRepository(
    private val engine: NomiEngine,
    private val activity: ActivityRepository,
    private val session: SessionScope,
    private val log: (String) -> Unit = {},
) {
    /** Per-chat state the store doesn't keep: it only matters while the chat is on screen. */
    private data class Flags(val lastError: String? = null, val accessLost: Boolean = false)

    private val flags = MutableStateFlow<Map<String, Flags>>(emptyMap())

    fun observe(sessionId: String): Flow<ChatState> = combine(
        engine.fromAccount(emptyList()) { store ->
            store.messages(sessionId, MESSAGE_LIMIT).map { rows -> rows.map { it.toDomain() } }
        },
        engine.fromAccount(null) { store -> store.draft(sessionId).map { it?.toDomain() } },
        activity.observeActive(sessionId),
        flags.map { it[sessionId] ?: Flags() },
    ) { messages, draft, agent, chatFlags ->
        ChatState(messages, draft, agent, chatFlags.lastError, chatFlags.accessLost)
    }.distinctUntilChanged()

    /**
     * Keeps the chat in sync until the handle is removed or the session ends: the engine resyncs it (newest page,
     * stale drafts and agents dropped) and applies its events; a failed turn or a lost topic shows in [observe].
     */
    fun bind(sessionId: String): ListenerHandle {
        val job = session.scope.launch(start = CoroutineStart.UNDISPATCHED) {
            val client = engine.get()
            // Kept in sync on whichever account is open: none yet means wait, a switch means again on the new one.
            launch {
                client.account.collectLatest { store ->
                    if (store == null) return@collectLatest
                    try {
                        client.syncChat(sessionId).collect {}
                    } catch (e: NomiException.AccountClosed) {
                        log("data: chat $sessionId stopped syncing, its account closed")
                    }
                }
            }
            val topic = CHAT_TOPIC_PREFIX + sessionId.lowercase()
            client.events.collect { event ->
                when {
                    event is SdkEvent.ChatTurnFailed && event.sessionId == sessionId.lowercase() ->
                        updateFlags(sessionId) { it.copy(lastError = event.error) }
                    event is SdkEvent.TopicAccessLost && event.topic == topic ->
                        updateFlags(sessionId) { it.copy(accessLost = true) }
                    event is SdkEvent.Realtime && event.topic == topic && event.event.movesTurnOn() ->
                        clearError(sessionId)
                }
            }
        }
        return ListenerHandle { job.cancel() }
    }

    /** Loads the newest page; a successful load means the chat is readable again. */
    suspend fun refresh(sessionId: String) = translated {
        engine.get().refreshMessages(sessionId)
        updateFlags(sessionId) { it.copy(accessLost = false) }
    }

    /** Loads the page before the oldest stored message; returns whether even older messages exist. */
    suspend fun loadOlder(sessionId: String): Boolean = translated { engine.get().loadOlderMessages(sessionId) }

    /**
     * Queues the message; it shows at once as `sending` and the engine delivers it (in order, retrying, across
     * restarts), then shows the server's copy.
     */
    suspend fun send(sessionId: String, text: String) {
        clearError(sessionId)
        translated { engine.get().sendMessage(sessionId, text) }
    }

    /**
     * Approves or denies a pending approval card. The card itself changes when the resumed turn updates it; an
     * [ai.nomi.network.ApiException] with 409 means the turn behind it has already moved on.
     */
    suspend fun resolveApproval(sessionId: String, messageId: String, approve: Boolean, remember: Boolean) = translated {
        engine.get().resolveApproval(sessionId, messageId, approve, remember)
    }

    /**
     * Sends a `failed` message again (it shows as `sending` until the backend has it). False when [messageId] isn't
     * one of the person's messages still in the outbox.
     */
    suspend fun retrySend(messageId: String): Boolean = translated {
        outboxKey(messageId)?.let { engine.get().retryOutbox(it) } ?: false
    }

    /** Drops a message that wasn't sent, and its row. False when [messageId] isn't one still in the outbox. */
    suspend fun discardSend(messageId: String): Boolean = translated {
        outboxKey(messageId)?.let { engine.get().discardOutbox(it) } ?: false
    }

    /** The outbox entry that sends the local message [messageId]. */
    private suspend fun outboxKey(messageId: String): String? =
        engine.account().outboxEntries().firstOrNull { entry ->
            (entry.operation as? OutboxOperation.SendMessage)?.localMessageId == messageId
        }?.key

    fun clearError(sessionId: String) = updateFlags(sessionId) { it.copy(lastError = null) }

    private fun updateFlags(sessionId: String, change: (Flags) -> Flags) {
        flags.update { all -> all + (sessionId to change(all[sessionId] ?: Flags())) }
    }
}

/** A delta or a finished turn: whatever failed before is behind the chat now. */
private fun RealtimeEvent.movesTurnOn(): Boolean = this is RealtimeEvent.ChatDelta || this is RealtimeEvent.TurnCompleted
