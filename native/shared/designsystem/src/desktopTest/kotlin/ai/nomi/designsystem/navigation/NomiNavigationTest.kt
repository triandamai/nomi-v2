package ai.nomi.designsystem.navigation

import ai.nomi.designsystem.icon.NomiIcons
import ai.nomi.designsystem.theme.NomiTheme
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.width
import androidx.compose.ui.Modifier
import androidx.compose.ui.test.assertHeightIsAtLeast
import androidx.compose.ui.test.assertIsDisplayed
import androidx.compose.ui.test.assertIsNotDisplayed
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.unit.dp
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollTo
import org.junit.Rule
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class NomiNavigationTest {
    @get:Rule val rule = createComposeRule()

    @Test
    fun navDrawerRendersAllWebParityItems() {
        val items = listOf(
            NomiNavItem("home", "Home", NomiIcons.Home),
            NomiNavItem("chats", "Chats", NomiIcons.Chat),
            NomiNavItem("projects", "Projects", NomiIcons.Folder),
            NomiNavItem("money", "Money", NomiIcons.Wallet),
            NomiNavItem("reminders", "Reminders", NomiIcons.Clock),
            NomiNavItem("memory", "Memory", NomiIcons.Memory),
            NomiNavItem("more", "More", NomiIcons.Apps),
        )

        val recents = listOf(
            NomiRecentChat("c1", "Design discussion"),
            NomiRecentChat("c2", "Refactor ideas"),
        )

        val usage = NomiUsageBrief(
            planName = "Pro",
            tokensUsed = 500_000,
            monthlyTokens = 1_000_000,
            canUpgrade = false,
        )

        val account = NomiAccount(
            displayName = "Ana Lima",
            email = "ana@nomi.test",
        )

        var selectedKey = "home"
        var openedChat: String? = null

        rule.setContent {
            NomiTheme(reducedMotion = true) {
                NomiNavDrawerContent(
                    items = items,
                    selected = selectedKey,
                    onSelect = { selectedKey = it },
                    onNewChat = {},
                    collapsed = false,
                    recents = recents,
                    onOpenRecentChat = { openedChat = it },
                    usage = usage,
                    account = account,
                )
            }
        }

        rule.onNodeWithText("Home").assertExists()
        rule.onNodeWithText("Chats").assertExists()
        rule.onNodeWithText("Projects").assertExists()
        rule.onNodeWithText("Money").assertExists()
        rule.onNodeWithText("Reminders").assertExists()
        rule.onNodeWithText("Memory").assertExists()
        rule.onNodeWithText("More").assertExists()

        // Recents
        rule.onNodeWithText("Recents").assertExists()
        rule.onNodeWithText("Design discussion").assertExists()

        // Usage
        rule.onNodeWithText("Pro").assertExists()

        // Account
        rule.onNodeWithText("Ana Lima").assertExists()

        // Test clicking recent chat
        rule.onNodeWithText("Design discussion").performScrollTo().performClick()
        assertEquals("c1", openedChat)
    }

    @Test
    fun avatarClickOpensAccountPopoverMenuWithAllNineItems() {
        val items = listOf(
            NomiNavItem("home", "Home", NomiIcons.Home),
            NomiNavItem("more", "More", NomiIcons.Apps),
        )
        val account = NomiAccount(displayName = "Trian Damai", email = "trian@nomi.test")
        var clickedMenuItem: NomiAccountMenuItem? = null

        rule.setContent {
            NomiTheme(reducedMotion = true) {
                NomiNavDrawerContent(
                    items = items,
                    selected = "home",
                    onSelect = {},
                    onNewChat = {},
                    collapsed = false,
                    account = account,
                    onAccountMenuItemClick = { clickedMenuItem = it },
                )
            }
        }

        // Initially menu items should not be visible
        rule.onNodeWithText("Preferences").assertDoesNotExist()

        // Click on the avatar / account row
        rule.onNodeWithText("Trian Damai").performClick()

        // All 9 menu items specified by the user should appear in the popover
        rule.onNodeWithText("Profile").assertExists()
        rule.onNodeWithText("Preferences").assertExists()
        rule.onNodeWithText("Account Setting").assertExists()
        rule.onNodeWithText("Billing & Usage").assertExists()
        rule.onNodeWithText("Connections").assertExists()
        rule.onNodeWithText("Model").assertExists()
        rule.onNodeWithText("Providers").assertExists()
        rule.onNodeWithText("Memory").assertExists()
        rule.onNodeWithText("Logout").assertExists()
        rule.onNodeWithText("Nomi 2.0").assertExists()

        // Click Preferences
        rule.onNodeWithText("Preferences").performClick()
        assertEquals(NomiAccountMenuItem.Preferences, clickedMenuItem)

        // Menu should be dismissed
        rule.onNodeWithText("Preferences").assertDoesNotExist()
    }

    @Test
    fun onAShortWindowTheBrandNewChatAndFootStayWhileTheMiddleScrolls() {
        val items = listOf(
            NomiNavItem("home", "Home", NomiIcons.Home),
            NomiNavItem("chats", "Chats", NomiIcons.Chat),
            NomiNavItem("projects", "Projects", NomiIcons.Folder),
            NomiNavItem("money", "Money", NomiIcons.Wallet),
            NomiNavItem("more", "More", NomiIcons.Apps),
        )
        val recents = (1..20).map { NomiRecentChat("c$it", "Chat number $it") }

        rule.setContent {
            NomiTheme(reducedMotion = true) {
                NomiNavDrawerContent(
                    items = items,
                    selected = "home",
                    onSelect = {},
                    onNewChat = {},
                    collapsed = false,
                    modifier = Modifier.width(300.dp).height(640.dp),
                    recents = recents,
                    onSeeAllChats = {},
                    usage = NomiUsageBrief(planName = "Free", tokensUsed = 80_000, monthlyTokens = 1_000_000, canUpgrade = false),
                    account = NomiAccount(displayName = "Ada Lovelace", email = "ada@nomi.test"),
                )
            }
        }

        // The rows keep their height: the list is longer than the window, and scrolls.
        rule.onNodeWithText("Home").assertIsDisplayed().assertHeightIsAtLeast(20.dp)
        rule.onNodeWithText("Chat number 20").assertIsNotDisplayed()

        rule.onNodeWithText("See all chats").performScrollTo().assertIsDisplayed()

        // Scrolled to the end, the top and the foot haven't moved off screen.
        rule.onNodeWithText("Chat number 20").assertIsDisplayed()
        rule.onNodeWithText("Home").assertIsNotDisplayed()
        rule.onNodeWithText("New chat").assertIsDisplayed()
        rule.onNodeWithText("Free").assertIsDisplayed()
        rule.onNodeWithText("Ada Lovelace").assertIsDisplayed()
    }
}
