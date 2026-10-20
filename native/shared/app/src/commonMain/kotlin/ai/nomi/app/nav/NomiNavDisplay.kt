package ai.nomi.app.nav

import androidx.compose.ui.unit.dp
import androidx.compose.ui.Alignment
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.background
import ai.nomi.designsystem.theme.NomiTheme
import ai.nomi.designsystem.shape.NomiIdentity
import ai.nomi.designsystem.shape.AgentShape
import ai.nomi.designsystem.layout.NomiPage
import ai.nomi.feature.auth.SignInScreen
import ai.nomi.feature.chat.ChatScreen
import ai.nomi.feature.chats.ChatsScreen
import ai.nomi.feature.home.HomeScreen
import ai.nomi.app.screens.connections.ConnectionsScreen
import ai.nomi.app.screens.memory.MemoryScreen
import ai.nomi.app.screens.money.MoneyScreen
import ai.nomi.app.screens.reminders.RemindersScreen
import ai.nomi.feature.notifications.NotificationsScreen
import ai.nomi.feature.settings.SettingsScreen
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.adaptive.currentWindowAdaptiveInfoV2
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.runtime.snapshots.SnapshotStateList
import androidx.compose.ui.Modifier
import androidx.lifecycle.viewmodel.navigation3.rememberViewModelStoreNavEntryDecorator
import androidx.navigation3.runtime.entryProvider
import androidx.navigation3.runtime.rememberSaveableStateHolderNavEntryDecorator
import androidx.navigation3.ui.NavDisplay
import androidx.window.core.layout.WindowSizeClass
import org.koin.compose.viewmodel.koinViewModel
import org.koin.core.parameter.parametersOf

/**
 * The app's screens over [backStack]. Each entry gets its own ViewModel store, cleared when the entry leaves the
 * back stack, so a chat's realtime binding (made by its ViewModel) lives exactly as long as the chat is stacked.
 * When [wide], Chats and the Chat opened from it share the window (list-detail).
 */
@Composable
fun NomiNavDisplay(
    backStack: SnapshotStateList<NomiRoute>,
    modifier: Modifier = Modifier,
    wide: Boolean = isWideWindow(),
) {
    val listDetail = remember(wide) { ListDetailSceneStrategy<NomiRoute>(enabled = wide, placeholder = { NoChatSelected() }) }
    val openLink: (String) -> Unit = { link -> NomiRoute.fromLink(link)?.let(backStack::open) }
    NavDisplay(
        backStack = backStack,
        modifier = modifier,
        onBack = { backStack.back() },
        entryDecorators = listOf(
            rememberSaveableStateHolderNavEntryDecorator(),
            rememberViewModelStoreNavEntryDecorator(),
        ),
        sceneStrategies = listOf(listDetail),
        entryProvider = entryProvider {
            entry<SignIn> { SignInScreen(koinViewModel()) }
            entry<Home> {
                HomeScreen(
                    viewModel = koinViewModel(),
                    onOpenChat = backStack::openChat,
                    onOpenChats = { backStack.selectTopLevel(Chats) },
                    onOpenReminders = { backStack.selectTopLevel(Reminders) },
                    onOpenPlans = { backStack.selectTopLevel(Projects) },
                )
            }
            entry<Chats>(metadata = ListPaneMetadata) {
                ChatsScreen(
                    koinViewModel(),
                    onOpenChat = backStack::openChat,
                    selectedId = (backStack.lastOrNull() as? Chat)?.sessionId,
                )
            }
            entry<Chat>(metadata = DetailPaneMetadata) { key ->
                ChatScreen(koinViewModel(key = key.sessionId) { parametersOf(key.sessionId) }, onBack = { backStack.back() })
            }
            entry<Projects> { ai.nomi.app.screens.projects.ProjectsScreen(koinViewModel(), onOpenChat = backStack::openChat) }
            entry<Money> { MoneyScreen(koinViewModel()) }
            entry<Reminders> { RemindersScreen(koinViewModel()) }
            entry<Memory> { MemoryScreen(koinViewModel()) }
            entry<Notifications> { NotificationsScreen(koinViewModel(), onOpenLink = openLink) }
            entry<Connections> { ConnectionsScreen(koinViewModel()) }
            entry<Settings> { SettingsScreen(koinViewModel(), onOpenConnections = { backStack.open(Connections) }) }
        },
    )
}

/** Medium width and up: room for two panes. */
@Composable
fun isWideWindow(): Boolean =
    currentWindowAdaptiveInfoV2().windowSizeClass.isWidthAtLeastBreakpoint(WindowSizeClass.WIDTH_DP_MEDIUM_LOWER_BOUND)

private val NoChatAvatar = 56.dp
private val NoChatGap = 12.dp

/** The detail pane before a chat is picked (wide windows). */
@Composable
private fun NoChatSelected() {
    Column(
        Modifier.fillMaxSize().background(NomiTheme.extendedColors.ground),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(NoChatGap, Alignment.CenterVertically),
    ) {
        AgentShape(NomiIdentity, NoChatAvatar, face = true)
        Text("Pick a chat to read it here.", style = MaterialTheme.typography.bodyLarge, color = MaterialTheme.colorScheme.onSurfaceVariant)
    }
}
