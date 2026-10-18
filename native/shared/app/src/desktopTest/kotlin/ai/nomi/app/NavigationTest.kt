package ai.nomi.app

import ai.nomi.app.nav.Chat
import ai.nomi.app.nav.Chats
import ai.nomi.app.nav.Home
import ai.nomi.app.nav.NomiNavDisplay
import ai.nomi.app.nav.NomiRoute
import ai.nomi.app.nav.Notifications
import ai.nomi.app.nav.Settings
import ai.nomi.app.nav.back
import ai.nomi.app.nav.openChat
import ai.nomi.app.nav.selectTopLevel
import ai.nomi.designsystem.theme.NomiTheme
import ai.nomi.feature.chat.CHAT_SCREEN_TAG
import ai.nomi.feature.chats.CHATS_SCREEN_TAG
import ai.nomi.feature.home.HOME_SCREEN_TAG
import ai.nomi.realtime.Topic
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithTag
import org.junit.Rule
import org.koin.core.context.startKoin
import org.koin.core.context.stopKoin
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNull

class NavigationTest {
    @get:Rule val rule = createComposeRule()

    private val graph = TestGraph()

    @BeforeTest
    fun start() {
        startKoin { modules(graph.modules) }
    }

    @AfterTest
    fun stop() = stopKoin()

    @Test
    fun links_map_to_routes() {
        assertEquals(Chat("abc"), NomiRoute.fromLink("/chat/abc"))
        assertEquals(Chat("abc"), NomiRoute.fromLink("https://nomi.example/chat/abc?from=push"))
        assertEquals(Notifications, NomiRoute.fromLink("/notifications"))
        assertEquals(Home, NomiRoute.fromLink("/projects/p1"))
        assertEquals(Home, NomiRoute.fromLink("/"))
        assertNull(NomiRoute.fromLink(""))
        assertEquals(Chat("abc"), NomiRoute.fromLink("nomi://chat/abc"))
        assertEquals(Notifications, NomiRoute.fromLink("nomi://notifications"))
        assertEquals(Home, NomiRoute.fromLink("nomi://"))
        assertEquals(Chat("a b/c"), NomiRoute.fromLink("/chat/a%20b%2Fc"))
        assertEquals(Chat("abc"), NomiRoute.fromLink("http://localhost:8080/chat/abc#latest"))
    }

    @Test
    fun back_stack_helpers_keep_home_at_the_root() {
        val stack = mutableStateListOf<NomiRoute>(Home)
        stack.selectTopLevel(Chats)
        stack.openChat("a")
        stack.openChat("b")
        assertEquals(listOf(Home, Chats, Chat("b")), stack.toList())
        stack.selectTopLevel(Settings)
        assertEquals(listOf(Home, Settings), stack.toList())
        stack.back()
        assertFalse(stack.back())
        assertEquals(listOf<NomiRoute>(Home), stack.toList())
    }

    @Test
    fun pushing_chat_then_back_returns_home_and_clears_the_chat_listener() {
        val backStack = mutableStateListOf<NomiRoute>(Home)
        val topic = Topic.Chat("abc")
        rule.setContent { NomiTheme(reducedMotion = true) { NomiNavDisplay(backStack, wide = false) } }
        rule.onNodeWithTag(HOME_SCREEN_TAG).assertExists()

        rule.runOnIdle { backStack.openChat("abc") }
        rule.onNodeWithTag(CHAT_SCREEN_TAG).assertExists()
        rule.runOnIdle { assertEquals(1, graph.hub.listenerCount(topic)) }

        rule.runOnIdle { backStack.back() }
        rule.waitForIdle()
        rule.onNodeWithTag(HOME_SCREEN_TAG).assertExists()
        rule.onNodeWithTag(CHAT_SCREEN_TAG).assertDoesNotExist()
        // The chat entry's ViewModel store was cleared with the entry, which removed its realtime listener.
        rule.runOnIdle { assertEquals(0, graph.hub.listenerCount(topic)) }
    }

    @Test
    fun wide_window_shows_chats_and_the_open_chat_side_by_side() {
        val backStack = mutableStateListOf(Home, Chats, Chat("abc"))
        rule.setContent { NomiTheme(reducedMotion = true) { NomiNavDisplay(backStack, wide = true) } }

        rule.onNodeWithTag(CHATS_SCREEN_TAG).assertExists()
        rule.onNodeWithTag(CHAT_SCREEN_TAG).assertExists()
    }

    @Test
    fun compact_window_shows_only_the_open_chat() {
        val backStack = mutableStateListOf(Home, Chats, Chat("abc"))
        rule.setContent { NomiTheme(reducedMotion = true) { NomiNavDisplay(backStack, wide = false) } }

        rule.onNodeWithTag(CHAT_SCREEN_TAG).assertExists()
        rule.onNodeWithTag(CHATS_SCREEN_TAG).assertDoesNotExist()
    }
}
