package ai.nomi.app

import ai.nomi.app.nav.Chat
import ai.nomi.app.nav.Home
import ai.nomi.app.nav.NomiNavDisplay
import ai.nomi.app.nav.NomiRoute
import ai.nomi.app.nav.Notifications
import ai.nomi.app.nav.Settings
import ai.nomi.app.nav.SignIn
import ai.nomi.app.nav.TopLevelRoutes
import ai.nomi.app.nav.Chats
import ai.nomi.app.nav.currentTopLevel
import ai.nomi.app.nav.isWideWindow
import ai.nomi.app.nav.resetTo
import ai.nomi.app.nav.selectTopLevel
import ai.nomi.data.SessionGate
import ai.nomi.designsystem.component.NomiButtonGroup
import ai.nomi.designsystem.component.NomiGroupOption
import ai.nomi.designsystem.component.NomiLoadingIndicator
import ai.nomi.designsystem.layout.NomiSpacing
import ai.nomi.designsystem.theme.NomiTheme
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.consumeWindowInsets
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.statusBars
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import org.koin.compose.koinInject

private const val FILL = 1f
private val TabBarMaxWidth = 560.dp

private fun NomiRoute.tabLabel(): String = when (this) {
    Home -> "Home"
    Chats -> "Chats"
    Notifications -> "Inbox"
    Settings -> "Settings"
    else -> ""
}

/**
 * Root of the Nomi UI, hosted by the Android, iOS and desktop shells (after `initKoin`). Routes on
 * [SessionGate.signedIn]: sign-in, or the signed-in shell (tabs + [NomiNavDisplay]).
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
    // A phone shows a chat full screen; tabs come back when it is popped.
    val showTabs = signedIn && backStack.lastOrNull() != SignIn && (wide || backStack.lastOrNull() !is Chat)
    Column(Modifier.fillMaxSize().background(NomiTheme.extendedColors.ground)) {
        if (showTabs && wide) Tabs(backStack, Modifier.statusBarsPadding())
        // The tab bar already sits clear of the system bar on its side; screens must not pad for it again.
        val consumed = when {
            !showTabs -> Modifier
            wide -> Modifier.consumeWindowInsets(WindowInsets.statusBars)
            else -> Modifier.consumeWindowInsets(WindowInsets.navigationBars)
        }
        NomiNavDisplay(backStack, Modifier.weight(FILL).then(consumed), wide = wide)
        if (showTabs && !wide) Tabs(backStack, Modifier.navigationBarsPadding())
    }
}

@Composable
private fun Tabs(backStack: MutableList<NomiRoute>, modifier: Modifier = Modifier) {
    Box(modifier.fillMaxWidth().padding(NomiSpacing.Small), contentAlignment = Alignment.Center) {
        NomiButtonGroup(
            options = TopLevelRoutes.map { NomiGroupOption(it, it.tabLabel()) },
            selected = backStack.currentTopLevel,
            onSelect = backStack::selectTopLevel,
            modifier = Modifier.widthIn(max = TabBarMaxWidth),
        )
    }
}

/** Whether the person asked the system for less motion (read where it is cheap; desktop has no such setting here). */
@Composable
expect fun rememberReducedMotion(): Boolean
