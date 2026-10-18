package ai.nomi.app

import ai.nomi.app.di.appModules
import ai.nomi.data.AuthRepository
import ai.nomi.database.DatabaseFactory
import ai.nomi.database.NomiDatabase
import ai.nomi.network.ApiConfig
import ai.nomi.network.MessageDto
import ai.nomi.network.MessagePageDto
import ai.nomi.network.MessagesApi
import ai.nomi.network.NotificationPageDto
import ai.nomi.network.NotificationsApi
import ai.nomi.network.SendMessageResponseDto
import ai.nomi.network.SessionDto
import ai.nomi.network.SessionsApi
import ai.nomi.network.SignInResult
import ai.nomi.network.TokenStore
import ai.nomi.network.Tokens
import ai.nomi.network.UnreadCountDto
import ai.nomi.realtime.ConnectionState
import ai.nomi.realtime.ListenerHandle
import ai.nomi.realtime.RealtimeEvent
import ai.nomi.realtime.RealtimeHub
import ai.nomi.realtime.RealtimeListener
import ai.nomi.realtime.Topic
import androidx.room.Room
import androidx.sqlite.driver.bundled.BundledSQLiteDriver
import io.ktor.client.engine.HttpClientEngine
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpStatusCode
import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.callbackFlow
import org.koin.core.module.Module
import org.koin.dsl.module

/** Records listeners per topic, so tests can see when a screen stops listening. Main-thread only. */
class FakeRealtimeHub : RealtimeHub {
    override val state: StateFlow<ConnectionState> = MutableStateFlow(ConnectionState.Idle)
    private val listeners = mutableListOf<Pair<Topic, RealtimeListener>>()
    var starts = 0
        private set
    var stops = 0
        private set

    override fun start() {
        starts++
    }

    override fun stop() {
        stops++
    }

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
}

class FakeTokenStore(private var tokens: Tokens? = null) : TokenStore {
    override suspend fun read(): Tokens? = tokens
    override suspend fun write(tokens: Tokens) {
        this.tokens = tokens
    }
    override suspend fun clear() {
        tokens = null
    }
}

class FakeAuthRepository(var session: Tokens? = null) : AuthRepository {
    var nextSignIn: SignInResult = SignInResult.SignedIn(Tokens("a", "r"))
    val signIns = mutableListOf<Pair<String, String>>()

    override suspend fun hasSession(): Boolean = session != null

    override suspend fun signIn(email: String, password: String): SignInResult {
        signIns += email to password
        return nextSignIn.also { if (it is SignInResult.SignedIn) session = it.tokens }
    }

    override suspend fun refresh(): Boolean = session != null

    override suspend fun signOut() {
        session = null
    }
}

class FakeSessionsApi : SessionsApi {
    var sessions: List<SessionDto> = emptyList()
    override suspend fun list(): List<SessionDto> = sessions
    override suspend fun create(): String = CREATED_SESSION_ID

    companion object {
        const val CREATED_SESSION_ID = "created"
    }
}

class FakeMessagesApi : MessagesApi {
    val sent = mutableListOf<Pair<String, String>>()

    override suspend fun page(sessionId: String, before: String?, limit: Int) = MessagePageDto(emptyList(), hasMore = false)

    override suspend fun get(sessionId: String, messageId: String): MessageDto = error("no message $messageId")

    override suspend fun send(sessionId: String, text: String): SendMessageResponseDto {
        sent += sessionId to text
        return SendMessageResponseDto(
            MessageDto(id = "u${sent.size}", sender = "user", content = text, contentBlocks = null, createdAt = "2026-10-18T10:00:00Z", agentDisplayName = null),
        )
    }
}

class FakeNotificationsApi : NotificationsApi {
    override suspend fun list(filter: String?, page: Int?) = NotificationPageDto(items = emptyList(), unread = 0)
    override suspend fun unread() = UnreadCountDto(0, latest = null)
    override suspend fun markRead(id: String) = Unit
    override suspend fun markAllRead() = Unit
}

class InMemoryDatabaseFactory : DatabaseFactory {
    override fun create(): NomiDatabase =
        Room.inMemoryDatabaseBuilder<NomiDatabase>().setDriver(BundledSQLiteDriver()).build()
}

/** What a platform module provides, with nothing touching the network or disk. */
val testPlatformModule: Module = module {
    single { ApiConfig("http://nomi.test") }
    single<TokenStore> { FakeTokenStore() }
    single<DatabaseFactory> { InMemoryDatabaseFactory() }
    single<HttpClientEngine> { MockEngine { respond("", HttpStatusCode.ServiceUnavailable) } }
}

/** The real graph with the realtime hub, auth and REST APIs swapped for fakes. */
class TestGraph {
    val hub = FakeRealtimeHub()
    val auth = FakeAuthRepository()
    val sessionsApi = FakeSessionsApi()
    val messagesApi = FakeMessagesApi()

    val modules: List<Module> = appModules(testPlatformModule) + module {
        single<RealtimeHub> { hub }
        single<AuthRepository> { auth }
        single<SessionsApi> { sessionsApi }
        single<MessagesApi> { messagesApi }
        single<NotificationsApi> { FakeNotificationsApi() }
    }
}
