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

/** How long the engine may take to start keeping an opened chat in sync, or to stop. */
private const val SYNC_TIMEOUT_MS = 5_000L
private const val TEST_ACCOUNT = "test-user"

class NavigationTest {
    @get:Rule val rule = createComposeRule()

    private lateinit var graph: TestGraph

    @BeforeTest
    fun start() {
        graph = TestGraph()
        val app = startKoin { modules(graph.modules) }
        app.koin.get<ai.nomi.data.SessionScope>().open()
        // What the session gate's engine would do at sign-in.
        kotlinx.coroutines.runBlocking { app.koin.get<ai.nomi.data.NomiEngine>().get().openAccount(TEST_ACCOUNT) }
    }

    @AfterTest
    fun stop() {
        kotlinx.coroutines.runBlocking {
            org.koin.mp.KoinPlatformTools.defaultContext().getOrNull()?.get<ai.nomi.data.SessionScope>()?.close()
        }
        stopKoin()
    }

    @Test
    fun links_map_to_routes() {
        assertEquals(Chat("abc"), NomiRoute.fromLink("/chat/abc"))
        assertEquals(Chat("abc"), NomiRoute.fromLink("https://nomi.example/chat/abc?from=push"))
        assertEquals(Notifications, NomiRoute.fromLink("/notifications"))
        assertEquals(ai.nomi.app.nav.Projects, NomiRoute.fromLink("/projects"))
        assertEquals(ai.nomi.app.nav.Projects, NomiRoute.fromLink("/projects/p1"))
        assertEquals(Home, NomiRoute.fromLink("/"))
        assertNull(NomiRoute.fromLink(""))
        assertEquals(Chat("abc"), NomiRoute.fromLink("nomi://chat/abc"))
        assertEquals(Notifications, NomiRoute.fromLink("nomi://notifications"))
        assertEquals(ai.nomi.app.nav.Projects, NomiRoute.fromLink("nomi://projects"))
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
    fun pushing_chat_then_back_returns_home_and_stops_syncing_the_chat() {
        val backStack = mutableStateListOf<NomiRoute>(Home)
        val syncing = { graph.backend.syncing.value["abc"] ?: 0 }
        rule.setContent { NomiTheme(reducedMotion = true) { NomiNavDisplay(backStack, wide = false) } }
        rule.onNodeWithTag(HOME_SCREEN_TAG).assertExists()

        rule.runOnIdle { backStack.openChat("abc") }
        rule.onNodeWithTag(CHAT_SCREEN_TAG).assertExists()
        rule.waitUntil(SYNC_TIMEOUT_MS) { syncing() == 1 }

        rule.runOnIdle { backStack.back() }
        rule.waitForIdle()
        rule.onNodeWithTag(HOME_SCREEN_TAG).assertExists()
        rule.onNodeWithTag(CHAT_SCREEN_TAG).assertDoesNotExist()
        // The chat entry's ViewModel store was cleared with the entry, which stopped keeping the chat in sync.
        rule.waitUntil(SYNC_TIMEOUT_MS) { syncing() == 0 }
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
