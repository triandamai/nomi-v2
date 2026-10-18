package ai.nomi.app

import ai.nomi.app.nav.Chat
import ai.nomi.app.nav.Chats
import ai.nomi.app.nav.Home
import ai.nomi.app.nav.NomiNavDisplay
import ai.nomi.app.nav.NomiRoute
import ai.nomi.app.nav.Notifications
import ai.nomi.app.nav.Settings
import ai.nomi.app.nav.SignIn
import ai.nomi.app.nav.TopLevelRoutes
import ai.nomi.app.nav.currentTopLevel
import ai.nomi.app.nav.isWideWindow
import ai.nomi.app.nav.resetTo
import ai.nomi.app.nav.selectTopLevel
import ai.nomi.data.NotificationRepository
import ai.nomi.data.SessionGate
import ai.nomi.designsystem.component.NomiLoadingIndicator
import ai.nomi.designsystem.icon.NomiIcons
import ai.nomi.designsystem.navigation.NomiNavItem
import ai.nomi.designsystem.navigation.NomiNavigationBar
import ai.nomi.designsystem.navigation.NomiSidebar
import ai.nomi.designsystem.theme.NomiTheme
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.consumeWindowInsets
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import org.koin.compose.koinInject

private const val FILL = 1f

private fun NomiRoute.navItem(unread: Int): NomiNavItem<NomiRoute>? = when (this) {
    Home -> NomiNavItem(this, "Home", NomiIcons.Home)
    Chats -> NomiNavItem(this, "Chats", NomiIcons.Chat)
    Notifications -> NomiNavItem(this, "Inbox", NomiIcons.Bell, badge = unread)
    Settings -> NomiNavItem(this, "Settings", NomiIcons.Settings)
    else -> null
}

/**
 * Root of the Nomi UI, hosted by the Android, iOS and desktop shells (after `initKoin`). Routes on
 * [SessionGate.signedIn]: sign-in, or the signed-in shell (navigation + [NomiNavDisplay]).
 */
@Composable
fun NomiApp(reducedMotion: Boolean = rememberReducedMotion()) {
    NomiTheme(reducedMotion = reducedMotion) {
        val gate = koinInject<SessionGate>()
        val restored by gate.restored.collectAsStateWithLifecycle()
        val signedIn by gate.signedIn.collectAsStateWithLifecycle()
        LaunchedEffect(gate) { gate.restore() }
        if (!restored) {
            Box(Modifier.fillMaxSize().background(NomiTheme.extendedColors.ground), contentAlignment = Alignment.Center) {
                NomiLoadingIndicator(contentDescription = "Loading")
            }
        } else {
            NomiShell(signedIn)
        }
    }
}

@Composable
private fun NomiShell(signedIn: Boolean) {
    val backStack = remember { mutableStateListOf<NomiRoute>(if (signedIn) Home else SignIn) }
    LaunchedEffect(signedIn) {
        val onSignIn = backStack.lastOrNull() == SignIn
        if (signedIn && onSignIn) backStack.resetTo(Home)
        if (!signedIn && !onSignIn) backStack.resetTo(SignIn)
    }
    val wide = isWideWindow()
    val inApp = signedIn && backStack.lastOrNull() != SignIn
    val unread by koinInject<NotificationRepository>().observeUnreadCount().collectAsStateWithLifecycle(0)
    val items = TopLevelRoutes.mapNotNull { it.navItem(unread) }
    var collapsed by rememberSaveable { mutableStateOf(false) }
    // Starting a conversation happens in Home's composer (the server saves a chat on its first message).
    val newChat = { backStack.selectTopLevel(Home) }

    Row(Modifier.fillMaxSize().background(NomiTheme.extendedColors.ground)) {
        if (inApp && wide) {
            NomiSidebar(
                items, backStack.currentTopLevel, backStack::selectTopLevel,
                onNewChat = newChat, collapsed = collapsed, onToggleCollapsed = { collapsed = !collapsed },
            )
        }
        Column(Modifier.weight(FILL).fillMaxHeight()) {
            // A phone shows a chat full screen; the bar comes back when it is popped.
            val bottomBar = inApp && !wide && backStack.lastOrNull() !is Chat
            val consumed = if (bottomBar) Modifier.consumeWindowInsets(WindowInsets.navigationBars) else Modifier
            NomiNavDisplay(backStack, Modifier.weight(FILL).then(consumed), wide = wide)
            if (bottomBar) NomiNavigationBar(items, backStack.currentTopLevel, backStack::selectTopLevel)
        }
    }
}

/** Whether the person asked the system for less motion (read where it is cheap; desktop has no such setting here). */
@Composable
expect fun rememberReducedMotion(): Boolean
