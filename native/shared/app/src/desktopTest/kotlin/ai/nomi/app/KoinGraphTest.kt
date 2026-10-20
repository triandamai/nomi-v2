package ai.nomi.app

import ai.nomi.app.di.appModules
import ai.nomi.data.ActivityRepository
import ai.nomi.data.AuthRepository
import ai.nomi.data.ChatRepository
import ai.nomi.data.ForegroundChat
import ai.nomi.data.NotificationRepository
import ai.nomi.data.RealtimeSync
import ai.nomi.data.SessionGate
import ai.nomi.data.SessionRepository
import ai.nomi.database.NomiDatabase
import ai.nomi.feature.auth.SignInViewModel
import ai.nomi.feature.chat.ChatViewModel
import ai.nomi.feature.chats.ChatsViewModel
import ai.nomi.feature.home.HomeViewModel
import ai.nomi.feature.notifications.NotificationsViewModel
import ai.nomi.feature.settings.SettingsViewModel
import ai.nomi.realtime.RealtimeHub
import ai.nomi.realtime.RealtimeTransport
import io.ktor.client.HttpClient
import org.koin.core.context.startKoin
import org.koin.core.context.stopKoin
import org.koin.core.parameter.parametersOf
import kotlin.test.AfterTest
import kotlin.test.Test
import kotlin.test.assertSame

/**
 * Starts the production graph on a test platform module and resolves every definition, so a missing binding or a
 * construction-time cycle fails here instead of at app start. (koin-test's `verify()` cannot follow the lambda
 * definitions with platform and function-typed parameters, so the graph is exercised for real.)
 */
class KoinGraphTest {
    @AfterTest
    fun tearDown() = stopKoin()

    @Test
    fun every_definition_resolves() {
        val koin = startKoin { modules(appModules(testPlatformModule)) }.koin

        koin.get<HttpClient>()
        koin.get<RealtimeTransport>()
        koin.get<RealtimeHub>()
        koin.get<NomiDatabase>()
        koin.get<AuthRepository>()
        koin.get<SessionRepository>()
        koin.get<NotificationRepository>()
        koin.get<ActivityRepository>()
        koin.get<ChatRepository>()
        koin.get<ForegroundChat>()
        koin.get<ai.nomi.data.UsageRepository>()
        koin.get<ai.nomi.data.ProjectsRepository>()
        koin.get<RealtimeSync>()
        koin.get<SessionGate>()
        koin.get<SignInViewModel>()
        koin.get<HomeViewModel>()
        koin.get<ChatsViewModel>()
        koin.get<ChatViewModel> { parametersOf("s1") }
        koin.get<NotificationsViewModel>()
        koin.get<SettingsViewModel>()
        koin.get<ai.nomi.app.screens.projects.ProjectsViewModel>()
    }

    @Test
    fun app_wide_objects_are_single_instances() {
        val koin = startKoin { modules(appModules(testPlatformModule)) }.koin

        assertSame(koin.get<RealtimeHub>(), koin.get<RealtimeHub>())
        assertSame(koin.get<NomiDatabase>(), koin.get<NomiDatabase>())
        assertSame(koin.get<SessionGate>(), koin.get<SessionGate>())
        assertSame(koin.get<ForegroundChat>(), koin.get<ForegroundChat>())
    }
}
