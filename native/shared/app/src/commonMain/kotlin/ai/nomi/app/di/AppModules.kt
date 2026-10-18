package ai.nomi.app.di

import ai.nomi.data.ActivityRepository
import ai.nomi.data.AuthRepository
import ai.nomi.data.ChatRepository
import ai.nomi.data.DefaultAuthRepository
import ai.nomi.data.EpochClock
import ai.nomi.data.ForegroundChat
import ai.nomi.data.NotificationRepository
import ai.nomi.data.RealtimeSync
import ai.nomi.data.SessionGate
import ai.nomi.data.SessionRepository
import ai.nomi.data.SystemEpochClock
import ai.nomi.database.DatabaseFactory
import ai.nomi.database.NomiDatabase
import ai.nomi.database.clearAll
import ai.nomi.feature.auth.SignInViewModel
import ai.nomi.feature.chat.ChatViewModel
import ai.nomi.feature.chats.ChatsViewModel
import ai.nomi.feature.home.HomeViewModel
import ai.nomi.feature.notifications.NotificationsViewModel
import ai.nomi.feature.settings.SettingsViewModel
import ai.nomi.network.AuthApi
import ai.nomi.network.HttpMessagesApi
import ai.nomi.network.HttpNotificationsApi
import ai.nomi.network.HttpSessionsApi
import ai.nomi.network.MessagesApi
import ai.nomi.network.NotificationsApi
import ai.nomi.network.SessionsApi
import ai.nomi.network.nomiHttpClient
import ai.nomi.realtime.DefaultRealtimeHub
import ai.nomi.realtime.KtorRealtimeTransport
import ai.nomi.realtime.RealtimeHub
import ai.nomi.realtime.RealtimeTransport
import io.ktor.client.HttpClient
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import org.koin.core.KoinApplication
import org.koin.core.context.startKoin
import org.koin.core.module.Module
import org.koin.core.module.dsl.viewModel
import org.koin.core.qualifier.named
import org.koin.dsl.module

/** The app-lifetime main-thread scope: the realtime hub's actor and the repositories' event handling run on it. */
val AppScope = named("app")

private const val LOG_PREFIX = "Nomi: "

/** Diagnostics for the shared layers. They never pass tokens or message text, only what failed. */
internal fun appLog(message: String) = println(LOG_PREFIX + message)

/**
 * Network, realtime, database and repositories. A platform module supplies the four platform pieces:
 * [ai.nomi.network.ApiConfig], [ai.nomi.network.TokenStore], [DatabaseFactory] and
 * [io.ktor.client.engine.HttpClientEngine].
 */
val coreModule: Module = module {
    single<CoroutineScope>(AppScope) { CoroutineScope(SupervisorJob() + Dispatchers.Main) }
    single<EpochClock> { SystemEpochClock }

    // A refresh the server rejects ends the session; resolved lazily, SessionGate needs this client itself.
    single<HttpClient> { nomiHttpClient(get(), get(), get(), onSignedOut = { get<SessionGate>().signOut() }) }
    single { AuthApi(get(), get(), get()) }
    single<SessionsApi> { HttpSessionsApi(get(), get()) }
    single<MessagesApi> { HttpMessagesApi(get(), get()) }
    single<NotificationsApi> { HttpNotificationsApi(get(), get()) }
    single<AuthRepository> { DefaultAuthRepository(get(), get()) }

    single<RealtimeTransport> { KtorRealtimeTransport(get(), get(), log = ::appLog) }
    single<RealtimeHub> {
        DefaultRealtimeHub(
            transport = get(),
            refreshToken = { get<SessionGate>().refreshSession() },
            scope = get(AppScope),
            log = ::appLog,
            // Rejected even with a freshly refreshed token: back to sign-in rather than a silently dead socket.
            onAuthLost = { get<SessionGate>().signOut() },
        )
    }

    single<NomiDatabase> { get<DatabaseFactory>().create() }
    single { SessionRepository(get(), get(), get()) }
    single { NotificationRepository(get(), get(), get()) }
    single { ActivityRepository(get(), get()) }
    single { ForegroundChat() }
    single { ChatRepository(get(), get(), get(), get(), scope = get(AppScope), clock = get(), log = ::appLog) }
    single { RealtimeSync(get(), get(), get(), get(), get(), scope = get(AppScope), log = ::appLog) }
    single {
        val database = get<NomiDatabase>()
        SessionGate(
            auth = get(),
            hub = get(),
            sync = get(),
            clearLocalData = { database.clearAll() },
            scope = get(AppScope),
            log = ::appLog,
        )
    }
}

/** One ViewModel per screen; the chat's takes its session id as a parameter. */
val featureModule: Module = module {
    viewModel { SignInViewModel(get()) }
    viewModel { HomeViewModel(get(), get(), log = ::appLog) }
    viewModel { ChatsViewModel(get(), log = ::appLog) }
    viewModel { (sessionId: String) -> ChatViewModel(sessionId, get(), get(), get(), log = ::appLog) }
    viewModel { NotificationsViewModel(get(), log = ::appLog) }
    viewModel { SettingsViewModel(get()) }
}

fun appModules(platformModule: Module): List<Module> = listOf(platformModule, coreModule, featureModule)

/** Starts Koin once per process, from the platform entry point. [configure] adds e.g. Android's `androidContext`. */
fun initKoin(platformModule: Module, configure: KoinApplication.() -> Unit = {}): KoinApplication = startKoin {
    configure()
    modules(appModules(platformModule))
}
