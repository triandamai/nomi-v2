package ai.nomi.app.di

import ai.nomi.companion.CompanionController
import ai.nomi.data.ActivityRepository
import ai.nomi.data.AppearanceRepository
import ai.nomi.data.AuthRepository
import ai.nomi.data.ChatRepository
import ai.nomi.data.CompanionPreferences
import ai.nomi.data.EngineSession
import ai.nomi.data.NomiEngine
import ai.nomi.data.SdkAuthRepository
import ai.nomi.data.SdkSetup
import ai.nomi.data.SessionEngine
import ai.nomi.data.TokenStoreCredentials
import ai.nomi.data.ForegroundChat
import ai.nomi.data.NotificationRepository
import ai.nomi.data.RealtimeSync
import ai.nomi.data.SessionGate
import ai.nomi.data.SessionRepository
import ai.nomi.data.SessionScope
import ai.nomi.feature.auth.SignInViewModel
import ai.nomi.feature.chat.ChatViewModel
import ai.nomi.feature.chats.ChatsViewModel
import ai.nomi.feature.home.HomeViewModel
import ai.nomi.feature.notifications.NotificationsViewModel
import ai.nomi.feature.settings.SettingsViewModel
import ai.nomi.network.ConnectionsApi
import ai.nomi.network.HomeApi
import ai.nomi.network.HttpConnectionsApi
import ai.nomi.network.HttpHomeApi
import ai.nomi.network.HttpMemoryApi
import ai.nomi.network.HttpMoneyApi
import ai.nomi.network.HttpRemindersApi
import ai.nomi.network.MemoryApi
import ai.nomi.network.MoneyApi
import ai.nomi.network.RemindersApi
import ai.nomi.network.nomiHttpClient
import ai.nomi.data.ConnectionsRepository
import ai.nomi.data.MemoryRepository
import ai.nomi.data.MoneyRepository
import ai.nomi.data.RemindersRepository
import ai.nomi.app.screens.connections.ConnectionsViewModel
import ai.nomi.app.screens.memory.MemoryViewModel
import ai.nomi.app.screens.money.MoneyViewModel
import ai.nomi.app.screens.reminders.RemindersViewModel
import ai.nomi.network.TokenStore
import ai.nomi.sdk.NativeNomiClient
import ai.nomi.sdk.defaultSdkConfig
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

/** The app-lifetime main-thread scope; each signed-in [SessionScope] is its child. */
val AppScope = named("app")

private const val LOG_PREFIX = "Nomi: "

/** Diagnostics for the shared layers. They never pass tokens or message text, only what failed. */
internal fun appLog(message: String) = println(LOG_PREFIX + message)

/**
 * The Rust engine, the remaining Ktor APIs and the repositories. A platform module supplies the four platform pieces:
 * [ai.nomi.network.ApiConfig], [TokenStore], [SdkSetup] and [io.ktor.client.engine.HttpClientEngine].
 */
val coreModule: Module = module {
    single<CoroutineScope>(AppScope) { CoroutineScope(SupervisorJob() + Dispatchers.Main) }

    // The Rust engine: chats, messages, notifications, agent activity, realtime and the session.
    single { ForegroundChat() }
    single {
        val setup = get<SdkSetup>()
        val credentials = TokenStoreCredentials(get<TokenStore>(), log = ::appLog)
        NomiEngine(get(AppScope), get()) { NativeNomiClient(defaultSdkConfig(setup.baseUrl, setup.dataDir), credentials) }
    }
    single<AuthRepository> { SdkAuthRepository(get(), get()) }
    single<SessionEngine> { EngineSession(get()) }

    // The APIs the engine doesn't cover yet, on Ktor with the same session (TokenStore). A refresh the server rejects
    // ends the session; resolved lazily, SessionGate needs this client itself.
    single<HttpClient> { nomiHttpClient(get(), get(), get(), onSignedOut = { get<SessionGate>().signOut() }) }
    single<HomeApi> { HttpHomeApi(get(), get()) }
    single<MoneyApi> { HttpMoneyApi(get(), get()) }
    single<RemindersApi> { HttpRemindersApi(get(), get()) }
    single<MemoryApi> { HttpMemoryApi(get(), get()) }
    single<ConnectionsApi> { HttpConnectionsApi(get(), get()) }
    single<ai.nomi.network.UsageApi> { ai.nomi.network.HttpUsageApi(get(), get()) }
    single<ai.nomi.network.ProjectsApi> { ai.nomi.network.HttpProjectsApi(get(), get()) }

    // The signed-in account's work (a child of AppScope); SessionGate opens it at sign-in and closes it before the wipe.
    single { SessionScope(get(AppScope)) }
    single { AppearanceRepository(get(), get(), log = ::appLog) }
    single { CompanionPreferences() }
    single { SessionRepository(get()) }
    single { NotificationRepository(get()) }
    single { ActivityRepository(get()) }
    single { ChatRepository(get(), get(), session = get(), log = ::appLog) }
    single { MoneyRepository(get(), get()) }
    single { RemindersRepository(get(), get()) }
    single { MemoryRepository(get(), get()) }
    single { ConnectionsRepository(get(), get()) }
    single { ai.nomi.data.UsageRepository(get(), get()) }
    single { ai.nomi.data.ProjectsRepository(get(), get()) }
    single { RealtimeSync(get(), get(), get(), session = get(), log = ::appLog) }
    single { CompanionController(get(AppScope), get(), get(), get(), sessionRepository = get(), log = ::appLog) }
    single { SessionGate(auth = get(), engine = get(), sync = get(), session = get(), scope = get(AppScope), log = ::appLog) }
}

/** One ViewModel per screen; the chat's takes its session id as a parameter. */
val featureModule: Module = module {
    viewModel { SignInViewModel(get()) }
    viewModel { HomeViewModel(get(), get(), get(), getOrNull(), log = ::appLog) }
    viewModel { ChatsViewModel(get(), log = ::appLog) }
    viewModel { (sessionId: String) -> ChatViewModel(sessionId, get(), get(), get(), log = ::appLog) }
    viewModel { NotificationsViewModel(get(), log = ::appLog) }
    viewModel { SettingsViewModel(get(), get(), get(), get(), log = ::appLog) }
    viewModel { MoneyViewModel(get()) }
    viewModel { RemindersViewModel(get()) }
    viewModel { MemoryViewModel(get()) }
    viewModel { ConnectionsViewModel(get()) }
    viewModel { ai.nomi.app.screens.projects.ProjectsViewModel(get()) }
}

fun appModules(platformModule: Module): List<Module> = listOf(platformModule, coreModule, featureModule)

/** Starts Koin once per process, from the platform entry point. [configure] adds e.g. Android's `androidContext`. */
fun initKoin(platformModule: Module, configure: KoinApplication.() -> Unit = {}): KoinApplication = startKoin {
    configure()
    modules(appModules(platformModule))
}
