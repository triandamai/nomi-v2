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
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.update

/** Finished turn ids remembered per bound chat, to drop their late deltas; turns finish one at a time. */
private const val FINISHED_TURNS_KEPT = 8

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
    private val session: SessionScope,
    private val clock: EpochClock,
    private val log: (String) -> Unit = {},
) {
    /** In-memory per-chat state that has no Room table: it only matters while the chat is on screen. */
    private data class Flags(val lastError: String? = null, val accessLost: Boolean = false)

    private class Binding(val queue: WorkQueue) {
        lateinit var registration: ListenerHandle

        /** The live handles given out by [bind] for this chat. */
        val holders = HashSet<ListenerHandle>()

        /** Turns that already completed or failed, oldest first; touched only by [queue]. */
        val finishedTurns = ArrayDeque<String>()

        fun finish(turnJobId: String) {
            if (turnJobId in finishedTurns) return
            finishedTurns.addLast(turnJobId)
            if (finishedTurns.size > FINISHED_TURNS_KEPT) finishedTurns.removeFirst()
        }
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

    /**
     * Listens on the chat's topic and loads its latest page; removing the last handle stops listening. The first
     * binding also resyncs (drops a draft / agent rows left over from before process death), later ones only refresh.
     */
    fun bind(sessionId: String): ListenerHandle {
        // A binding from a session that has since ended has a dead queue: replace it.
        val existing = bindings[sessionId]?.takeIf { it.queue.isActive } ?: run {
            bindings.remove(sessionId)?.registration?.remove()
            null
        }
        val binding = existing ?: Binding(WorkQueue(session.scope, log)).also { created ->
            created.registration = hub.addListener(Topic.Chat(sessionId)) { event ->
                created.queue.submit { handle(sessionId, created, event) }
            }
            bindings[sessionId] = created
        }
        binding.queue.submit { if (existing == null) resync(sessionId) else refresh(sessionId) }
        val handle = object : ListenerHandle {
            override fun remove() {
                if (!binding.holders.remove(this) || binding.holders.isNotEmpty()) return
                if (bindings[sessionId] === binding) bindings.remove(sessionId)
                binding.registration.remove()
                binding.queue.close()
            }
        }
        binding.holders += handle
        return handle
    }

    /** Loads the newest page into Room; a successful load means the chat is readable again. */
    suspend fun refresh(sessionId: String) = session.run {
        store(sessionId, api.page(sessionId).messages)
        updateFlags(sessionId) { it.copy(accessLost = false) }
    }

    /**
     * Refetches, then drops the streaming draft and agent rows: events that would have ended them (TurnCompleted,
     * AgentSessionEnded) may have been missed while disconnected or dead. A turn that is still live recreates both
     * from its next delta / phase event.
     */
    private suspend fun resync(sessionId: String) {
        refresh(sessionId)
        db.streamingDraftDao().clear(sessionId)
        activity.clearForSession(sessionId)
    }

    /** Loads the page before [beforeMessageId]; returns whether even older messages exist. */
    suspend fun loadOlder(sessionId: String, beforeMessageId: String): Boolean = session.run {
        val page = api.page(sessionId, before = beforeMessageId)
        store(sessionId, page.messages)
        page.hasMore
    }

    /** Posts the message and stores what the server stored; the reply arrives over realtime. */
    suspend fun send(sessionId: String, text: String) = session.run {
        clearError(sessionId)
        val response = api.send(sessionId, text)
        store(sessionId, listOfNotNull(response.userMessage, response.supervisorReply))
    }

    fun clearError(sessionId: String) = updateFlags(sessionId) { it.copy(lastError = null) }

    private suspend fun handle(sessionId: String, binding: Binding, event: RealtimeEvent) {
        val drafts = db.streamingDraftDao()
        when (event) {
            is RealtimeEvent.ChatDelta -> {
                val text = event.textOrNull() ?: return
                // A delta delivered after its turn ended must not resurrect the draft.
                if (event.turnJobId in binding.finishedTurns) return
                drafts.appendDelta(sessionId, event.turnJobId, text, clock())
                clearError(sessionId)
            }
            is RealtimeEvent.TurnCompleted -> {
                binding.finish(event.turnJobId)
                fetchAndStore(sessionId, event.messageId)
                // Cleared even if the fetch failed (the turn is over; the next refresh brings the message), but
                // only this turn's draft: a newer turn may already be streaming.
                drafts.clearTurn(sessionId, event.turnJobId)
                clearError(sessionId)
            }
            is RealtimeEvent.MessageCreated -> {
                // A user message (e.g. sent from another device) must not wipe a reply that is still streaming.
                if (fetchAndStore(sessionId, event.messageId)?.sender == ROLE_ASSISTANT) drafts.clear(sessionId)
            }
            is RealtimeEvent.MessageUpdated -> fetchAndStore(sessionId, event.messageId)
            is RealtimeEvent.TurnFailed -> {
                binding.finish(event.turnJobId)
                drafts.clearTurn(sessionId, event.turnJobId)
                updateFlags(sessionId) { it.copy(lastError = event.error) }
            }
            is RealtimeEvent.SessionRenamed -> {
                val sessions = db.sessionDao()
                sessions.get(sessionId)?.let { sessions.upsert(it.copy(title = event.title)) }
            }
            is RealtimeEvent.Resynced -> resync(sessionId)
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
