package ai.nomi.app

import ai.nomi.data.ChatRepository
import ai.nomi.data.ForegroundChat
import ai.nomi.data.SessionRepository
import ai.nomi.designsystem.theme.NomiTheme
import ai.nomi.feature.chat.ChatScreen
import ai.nomi.feature.chat.ChatViewModel
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleOwner
import androidx.lifecycle.LifecycleRegistry
import androidx.lifecycle.compose.LocalLifecycleOwner
import org.junit.Rule
import org.koin.core.context.startKoin
import org.koin.core.context.stopKoin
import org.koin.mp.KoinPlatform
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

/**
 * The open chat counts as "on screen" (its replies are not unread nor popped up) only while its screen is RESUMED: a
 * minimised / unfocused desktop window or a backgrounded phone app pauses it.
 */
class ChatForegroundTest {
    @get:Rule val rule = createComposeRule()

    private val graph = TestGraph()

    private class ScreenLifecycle : LifecycleOwner {
        val registry = LifecycleRegistry.createUnsafe(this)
        override val lifecycle: Lifecycle get() = registry
    }

    @BeforeTest
    fun start() {
        startKoin { modules(graph.modules) }
    }

    @AfterTest
    fun stop() = stopKoin()

    @Test
    fun the_chat_is_foreground_only_while_its_screen_is_resumed() {
        val koin = KoinPlatform.getKoin()
        val foreground = koin.get<ForegroundChat>()
        val owner = ScreenLifecycle()
        val viewModel = ChatViewModel(CHAT_ID, koin.get<ChatRepository>(), koin.get<SessionRepository>(), foreground)
        rule.runOnIdle { owner.registry.currentState = Lifecycle.State.RESUMED }
        rule.setContent {
            CompositionLocalProvider(LocalLifecycleOwner provides owner) {
                NomiTheme { ChatScreen(viewModel) }
            }
        }
        rule.runOnIdle { assertEquals(CHAT_ID, foreground.sessionId.value) }

        rule.runOnIdle { owner.registry.currentState = Lifecycle.State.STARTED } // window minimised / app backgrounded
        rule.runOnIdle { assertNull(foreground.sessionId.value) }

        rule.runOnIdle { owner.registry.currentState = Lifecycle.State.RESUMED }
        rule.runOnIdle { assertEquals(CHAT_ID, foreground.sessionId.value) }
    }

    private companion object {
        const val CHAT_ID = "s1"
    }
}
