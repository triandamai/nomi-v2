package ai.nomi.data

import ai.nomi.database.NomiDatabase
import ai.nomi.network.MessageDto
import ai.nomi.network.MessagePageDto
import ai.nomi.network.MessagesApi
import ai.nomi.network.NotificationPageDto
import ai.nomi.network.NotificationsApi
import ai.nomi.network.SendMessageResponseDto
import ai.nomi.network.SessionDto
import ai.nomi.network.SessionsApi
import ai.nomi.network.UnreadCountDto
import ai.nomi.realtime.ConnectionState
import ai.nomi.realtime.ListenerHandle
import ai.nomi.realtime.RealtimeEvent
import ai.nomi.realtime.RealtimeHub
import ai.nomi.realtime.RealtimeListener
import ai.nomi.realtime.Topic
import androidx.room.Room
import androidx.sqlite.driver.bundled.BundledSQLiteDriver
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.callbackFlow

fun inMemoryDb(): NomiDatabase = Room.inMemoryDatabaseBuilder<NomiDatabase>().setDriver(BundledSQLiteDriver()).build()

/** Dispatches [emit]ted events synchronously to the listeners registered for the event's topic. */
class FakeRealtimeHub : RealtimeHub {
    override val state: StateFlow<ConnectionState> = MutableStateFlow(ConnectionState.Idle)
    private val listeners = mutableListOf<Pair<Topic, RealtimeListener>>()

    override fun start() = Unit
    override fun stop() = Unit

    override fun addListener(topic: Topic, listener: RealtimeListener): ListenerHandle {
        val entry = topic to listener
        listeners += entry
        return object : ListenerHandle {
            override fun remove() {
                listeners.remove(entry)
            }
        }
    }

    override fun events(topic: Topic): Flow<RealtimeEvent> = callbackFlow {
        val handle = addListener(topic) { trySend(it) }
        awaitClose { handle.remove() }
    }

    fun listenerCount(topic: Topic): Int = listeners.count { it.first == topic }

    fun emit(event: RealtimeEvent) {
        listeners.filter { it.first == event.topic }.forEach { it.second.onEvent(event) }
    }
}

fun messageDto(id: String, sender: String = "assistant", content: String = "text $id", createdAt: String = "2026-10-18T10:00:00Z") =
    MessageDto(id = id, sender = sender, content = content, contentBlocks = null, createdAt = createdAt, agentDisplayName = null)

class FakeMessagesApi : MessagesApi {
    /** Served by [page]; tests swap it to simulate the server changing. */
    var latest: List<MessageDto> = emptyList()
    val byId = mutableMapOf<String, MessageDto>()
    var pageCalls = 0
    val sent = mutableListOf<Pair<String, String>>()

    override suspend fun page(sessionId: String, before: String?, limit: Int): MessagePageDto {
        pageCalls++
        return MessagePageDto(latest, hasMore = false)
    }

    override suspend fun get(sessionId: String, messageId: String): MessageDto =
        byId[messageId] ?: error("no message $messageId")

    override suspend fun send(sessionId: String, text: String): SendMessageResponseDto {
        sent += sessionId to text
        return SendMessageResponseDto(messageDto("u-${sent.size}", sender = "user", content = text))
    }
}

class FakeSessionsApi : SessionsApi {
    var sessions: List<SessionDto> = emptyList()
    var listCalls = 0

    override suspend fun list(): List<SessionDto> {
        listCalls++
        return sessions
    }

    override suspend fun create(): String = "created"
}

class FakeNotificationsApi : NotificationsApi {
    var page = NotificationPageDto(items = emptyList(), unread = 0)
    var listCalls = 0
    val markedRead = mutableListOf<String>()

    override suspend fun list(filter: String?, page: Int?): NotificationPageDto {
        listCalls++
        return this.page
    }

    override suspend fun unread(): UnreadCountDto = UnreadCountDto(page.unread, latest = null)

    override suspend fun markRead(id: String) {
        markedRead += id
    }

    override suspend fun markAllRead() = Unit
}

class FakeClock(var now: Long = START_MILLIS) : () -> Long {
    override fun invoke(): Long = now

    companion object {
        const val START_MILLIS = 1_760_000_000_000L
    }
}
