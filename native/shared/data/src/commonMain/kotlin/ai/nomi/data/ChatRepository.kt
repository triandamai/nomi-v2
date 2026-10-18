package ai.nomi.data

import ai.nomi.database.NomiDatabase
import ai.nomi.network.ApiException
import ai.nomi.network.MessageDto
import ai.nomi.network.MessagesApi
import ai.nomi.realtime.ListenerHandle
import ai.nomi.realtime.RealtimeEvent
import ai.nomi.realtime.RealtimeHub
import ai.nomi.realtime.Topic
import ai.nomi.realtime.textOrNull
import io.ktor.http.HttpStatusCode
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.update

/**
 * One chat, offline-first: REST pages and realtime events are written to Room and [observe] reads Room only.
 *
 * [bind] is how a chat screen goes live; it is ref-counted per chat so the same chat open twice (e.g. on the back
 * stack twice) applies each event once. Call [bind] and [ListenerHandle.remove] from one thread (the main thread,
 * like ViewModel init / onCleared). Event handling runs on [scope], serially per chat.
 */
class ChatRepository(
    private val api: MessagesApi,
    private val db: NomiDatabase,
    private val hub: RealtimeHub,
    private val activity: ActivityRepository,
    private val scope: CoroutineScope,
    private val clock: EpochClock,
    private val log: (String) -> Unit = {},
) {
    /** In-memory per-chat state that has no Room table: it only matters while the chat is on screen. */
    private data class Flags(val lastError: String? = null, val accessLost: Boolean = false)

    private class Binding(val registration: ListenerHandle, val queue: WorkQueue) {
        /** The live handles given out by [bind] for this chat. */
        val holders = HashSet<ListenerHandle>()
    }

    private val flags = MutableStateFlow<Map<String, Flags>>(emptyMap())
    private val bindings = HashMap<String, Binding>()

    fun observe(sessionId: String): Flow<ChatState> = combine(
        db.messageDao().observe(sessionId).map { rows -> rows.map { it.toDomain() } },
        db.streamingDraftDao().observe(sessionId).map { it?.toDomain() },
        activity.observeActive(sessionId),
        flags.map { it[sessionId] ?: Flags() },
    ) { messages, draft, agent, chatFlags ->
        ChatState(messages, draft, agent, chatFlags.lastError, chatFlags.accessLost)
    }.distinctUntilChanged()

    /** Listens on the chat's topic and loads its latest page; removing the last handle stops listening. */
    fun bind(sessionId: String): ListenerHandle {
        val binding = bindings.getOrPut(sessionId) {
            val queue = WorkQueue(scope, log)
            val registration = hub.addListener(Topic.Chat(sessionId)) { event -> queue.submit { handle(sessionId, event) } }
            Binding(registration, queue)
        }
        binding.queue.submit { refresh(sessionId) }
        val handle = object : ListenerHandle {
            override fun remove() {
                if (!binding.holders.remove(this) || binding.holders.isNotEmpty()) return
                bindings.remove(sessionId)
                binding.registration.remove()
                binding.queue.close()
            }
        }
        binding.holders += handle
        return handle
    }

    /** Loads the newest page into Room. */
    suspend fun refresh(sessionId: String) {
        store(sessionId, api.page(sessionId).messages)
    }

    /** Loads the page before [beforeMessageId]; returns whether even older messages exist. */
    suspend fun loadOlder(sessionId: String, beforeMessageId: String): Boolean {
        val page = api.page(sessionId, before = beforeMessageId)
        store(sessionId, page.messages)
        return page.hasMore
    }

    /** Posts the message and stores what the server stored; the reply arrives over realtime. */
    suspend fun send(sessionId: String, text: String) {
        clearError(sessionId)
        val response = api.send(sessionId, text)
        store(sessionId, listOfNotNull(response.userMessage, response.supervisorReply))
    }

    fun clearError(sessionId: String) = updateFlags(sessionId) { it.copy(lastError = null) }

    private suspend fun handle(sessionId: String, event: RealtimeEvent) {
        val drafts = db.streamingDraftDao()
        when (event) {
            is RealtimeEvent.ChatDelta -> {
                val text = event.textOrNull() ?: return
                drafts.appendDelta(sessionId, event.turnJobId, text, clock())
                clearError(sessionId)
            }
            is RealtimeEvent.TurnCompleted -> {
                fetchAndStore(sessionId, event.messageId)
                // Cleared even if the fetch failed: the turn is over; the next refresh brings the message.
                drafts.clear(sessionId)
                clearError(sessionId)
            }
            is RealtimeEvent.MessageCreated -> {
                // A user message (e.g. sent from another device) must not wipe a reply that is still streaming.
                if (fetchAndStore(sessionId, event.messageId)?.sender == ROLE_ASSISTANT) drafts.clear(sessionId)
            }
            is RealtimeEvent.MessageUpdated -> fetchAndStore(sessionId, event.messageId)
            is RealtimeEvent.TurnFailed -> {
                drafts.clear(sessionId)
                updateFlags(sessionId) { it.copy(lastError = event.error) }
            }
            is RealtimeEvent.SessionRenamed -> {
                val sessions = db.sessionDao()
                sessions.get(sessionId)?.let { sessions.upsert(it.copy(title = event.title)) }
            }
            is RealtimeEvent.Resynced -> refresh(sessionId)
            is RealtimeEvent.AccessLost -> updateFlags(sessionId) { it.copy(accessLost = true) }
            is RealtimeEvent.AgentSessionStarted,
            is RealtimeEvent.AgentPhaseChanged,
            is RealtimeEvent.AgentSessionEnded,
            -> activity.onEvent(event)
            else -> Unit
        }
    }

    /** Fetches one message into Room; a message the server no longer has is removed. Null when nothing was stored. */
    private suspend fun fetchAndStore(sessionId: String, messageId: String): MessageDto? = try {
        api.get(sessionId, messageId).also { store(sessionId, listOf(it)) }
    } catch (e: CancellationException) {
        throw e
    } catch (e: ApiException) {
        if (e.status == HttpStatusCode.NotFound.value) db.messageDao().delete(messageId)
        log("data: fetching message $messageId failed: $e")
        null
    } catch (e: Exception) {
        log("data: fetching message $messageId failed: $e")
        null
    }

    private suspend fun store(sessionId: String, messages: List<MessageDto>) {
        if (messages.isNotEmpty()) db.messageDao().upsert(messages.map { it.toEntity(sessionId) })
    }

    private fun updateFlags(sessionId: String, change: (Flags) -> Flags) {
        flags.update { all -> all + (sessionId to change(all[sessionId] ?: Flags())) }
    }
}
