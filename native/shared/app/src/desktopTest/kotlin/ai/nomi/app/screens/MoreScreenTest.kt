package ai.nomi.app.screens

import ai.nomi.app.screens.more.MoreContent
import ai.nomi.designsystem.theme.NomiTheme
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onAllNodesWithText
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollTo
import org.junit.Rule
import kotlin.test.Test
import kotlin.test.assertEquals

class MoreScreenTest {
    @get:Rule val rule = createComposeRule()

    @Test
    fun moreScreenRendersSectionsAndFeatures() {
        var pins by mutableStateOf(listOf("money", "reminders"))
        var openedFeature: String? = null
        var customizeClicked = false

        rule.setContent {
            NomiTheme(reducedMotion = true) {
                MoreContent(
                    pins = pins,
                    onTogglePin = { id ->
                        pins = if (id in pins) pins - id else pins + id
                    },
                    onOpenFeature = { openedFeature = it },
                    onCustomizeDrawer = { customizeClicked = true },
                )
            }
        }

        // Header
        rule.onNodeWithText("All Features").assertExists()
        rule.onNodeWithText("Customize drawer order").assertExists()

        // Sections
        rule.onNodeWithText("Main").assertExists()
        rule.onNodeWithText("Life").assertExists()
        rule.onAllNodesWithText("Crew")[0].assertExists()
        rule.onNodeWithText("Account").assertExists()

        // Features in Main
        rule.onNodeWithText("Home").assertExists()
        rule.onNodeWithText("Chats").assertExists()
        rule.onNodeWithText("Projects").assertExists()

        // Features in Life
        rule.onNodeWithText("Money").assertExists()
        rule.onNodeWithText("Reminders").assertExists()
        rule.onNodeWithText("Memory").assertExists()

        // Test clicking feature tile opens it
        rule.onNodeWithText("Projects").performClick()
        assertEquals("projects", openedFeature)

        // Test toggle pin
        rule.onNodeWithContentDescription("Unpin Money from drawer").performClick()
        assertEquals(listOf("reminders"), pins)

        rule.onNodeWithContentDescription("Pin Memory to drawer").performClick()
        assertEquals(listOf("reminders", "memory"), pins)

        // Test customize drawer link
        rule.onNodeWithText("Customize drawer order").performClick()
        assertEquals(true, customizeClicked)
    }
}
