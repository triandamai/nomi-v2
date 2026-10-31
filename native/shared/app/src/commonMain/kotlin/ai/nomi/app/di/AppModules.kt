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
import ai.nomi.data.SdkConnectionsApi
import ai.nomi.data.SdkHomeApi
import ai.nomi.data.SdkMemoryApi
import ai.nomi.data.SdkMoneyApi
import ai.nomi.data.SdkProjectsApi
import ai.nomi.data.SdkRemindersApi
import ai.nomi.data.SdkSetup
import ai.nomi.data.SdkUsageApi
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
import ai.nomi.network.ApiConfig
import ai.nomi.network.ConnectionsApi
import ai.nomi.network.HomeApi
import ai.nomi.network.MemoryApi
import ai.nomi.network.MoneyApi
import ai.nomi.network.RemindersApi
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
import kotlinx.coroutines.CoroutineExceptionHandler
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

/** What app-lifetime and signed-in work does with a failure nothing caught: log it, keep the app running. */
private val backgroundFailures = CoroutineExceptionHandler { _, failure ->
    appLog("background work failed: ${failure.stackTraceToString()}")
}

/**
 * The Rust engine and the repositories over it. A platform module supplies the three platform pieces:
 * [ai.nomi.network.ApiConfig], [TokenStore] and [SdkSetup].
 */
val coreModule: Module = module {
    // A failure in background work is logged, never fatal: without a handler, one failed task ends the app.
    single<CoroutineScope>(AppScope) { CoroutineScope(SupervisorJob() + Dispatchers.Main + backgroundFailures) }

    // The Rust engine: chats, messages, notifications, agent activity, realtime and the session.
    single { ForegroundChat() }
    single {
        val setup = get<SdkSetup>()
        val credentials = TokenStoreCredentials(get<TokenStore>(), log = ::appLog)
        NomiEngine(get(AppScope), get(), setup.device, log = ::appLog) {
            NativeNomiClient(defaultSdkConfig(setup.baseUrl, setup.dataDir), credentials)
        }
    }
    single<AuthRepository> { SdkAuthRepository(get(), get()) }
    single<SessionEngine> { EngineSession(get()) }

    // The account pages, on the engine too.
    single<HomeApi> { SdkHomeApi(get()) }
    single<MoneyApi> { SdkMoneyApi(get()) }
    single<RemindersApi> { SdkRemindersApi(get()) }
    single<MemoryApi> { SdkMemoryApi(get()) }
    single<ConnectionsApi> { SdkConnectionsApi(get()) }
    single<ai.nomi.network.UsageApi> { SdkUsageApi(get()) }
    single<ai.nomi.network.ProjectsApi> { SdkProjectsApi(get()) }
    single<ai.nomi.network.AttachmentsApi> { ai.nomi.data.SdkAttachmentsApi(get()) }

    // The signed-in account's work (a child of AppScope); SessionGate opens it at sign-in and closes it before the wipe.
    single { SessionScope(get(AppScope)) }
    single { AppearanceRepository(get(), get(), log = ::appLog) }
    single { CompanionPreferences() }
    single { SessionRepository(get()) }
    single { NotificationRepository(get()) }
    single { ActivityRepository(get()) }
    single { ChatRepository(get(), get(), session = get(), log = ::appLog, webUrl = get<ApiConfig>().webUrl) }
    single { MoneyRepository(get(), get()) }
    single { RemindersRepository(get(), get()) }
    single { MemoryRepository(get(), get()) }
    single { ConnectionsRepository(get(), get()) }
    single { ai.nomi.data.UsageRepository(get(), get()) }
    single { ai.nomi.data.ProjectsRepository(get(), get()) }
    single { ai.nomi.data.AttachmentPreviews(get(), log = ::appLog) }
    single { ai.nomi.data.ProjectWorkspaceRepository(get()) }
    single { ai.nomi.data.ProjectToolsRepository(get()) }
    single { RealtimeSync(get(), get(), get(), session = get(), log = ::appLog) }
    single {
        val home = get<HomeApi>()
        val projects = get<ai.nomi.data.ProjectsRepository>()
        CompanionController(
            get(AppScope), get(), get(), get(),
            sessionRepository = get(),
            activityRepository = get(),
            loadCrewNames = { home.crew().associate { it.agentType to it.name } },
            loadProjects = {
                projects.list().getOrThrow()
                    .filter { it.sessionId.isNotBlank() }
                    .sortedByDescending { it.updatedAt }
                    .map { ai.nomi.companion.CompanionProject(it.id, it.sessionId, it.name, it.status) }
            },
            log = ::appLog,
        )
    }
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
    viewModel {
        val workspaces = get<ai.nomi.data.ProjectWorkspaceRepository>()
        ai.nomi.app.screens.projects.ProjectsViewModel(get(), onProjectChats = workspaces::rememberProjectChats)
    }
    viewModel { (sessionId: String) -> ai.nomi.app.screens.projects.ProjectWorkspaceViewModel(sessionId, get(), log = ::appLog) }
    viewModel { (projectId: String, name: String, relay: Boolean) ->
        ai.nomi.app.screens.projects.ProjectToolsViewModel(projectId, name, relay, get(), log = ::appLog)
    }
}

fun appModules(platformModule: Module): List<Module> = listOf(platformModule, coreModule, featureModule)

/** Starts Koin once per process, from the platform entry point. [configure] adds e.g. Android's `androidContext`. */
fun initKoin(platformModule: Module, configure: KoinApplication.() -> Unit = {}): KoinApplication = startKoin {
    configure()
    modules(appModules(platformModule))
}
