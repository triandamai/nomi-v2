package ai.nomi.feature.settings

import ai.nomi.designsystem.theme.NomiTheme
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performScrollTo
import org.junit.Rule
import kotlin.test.Test
import kotlin.test.assertEquals

class DrawerCustomizerTest {
    @get:Rule val rule = createComposeRule()

    @Test
    fun drawerCustomizerRendersFixedPinnedAndAvailableSections() {
        var pins by mutableStateOf(listOf("money", "reminders", "memory"))
        var recentsEnabled by mutableStateOf(true)
        var recentsLimit by mutableStateOf(10)
        var timezone by mutableStateOf("UTC")

        rule.setContent {
            NomiTheme(reducedMotion = true) {
                Box(modifier = Modifier.verticalScroll(rememberScrollState())) {
                    DrawerCustomizer(
                        pins = pins,
                        onMovePin = { from, to ->
                            val next = pins.toMutableList()
                            val item = next.removeAt(from)
                            next.add(to, item)
                            pins = next
                        },
                        onUnpin = { id -> pins = pins - id },
                        onPin = { id -> pins = pins + id },
                        recentsEnabled = recentsEnabled,
                        onToggleRecents = { recentsEnabled = it },
                        recentsLimit = recentsLimit,
                        onSelectRecentsLimit = { recentsLimit = it },
                        currentTimezone = timezone,
                        onSelectTimezone = { timezone = it },
                    )
                }
            }
        }

        // Fixed items
        rule.onNodeWithText("Home").assertExists()
        rule.onNodeWithText("Chats").assertExists()
        rule.onNodeWithText("Projects").assertExists()

        // Pinned items
        rule.onNodeWithText("Money").assertExists()
        rule.onNodeWithText("Reminders").assertExists()
        rule.onNodeWithText("Memory").assertExists()

        // Available items to pin
        rule.onNodeWithText("Inbox").assertExists()
        rule.onNodeWithText("Connections").assertExists()

        // Test unpinning Money
        rule.onNodeWithContentDescription("Remove Money from drawer").performClick()
        assertEquals(listOf("reminders", "memory"), pins)

        // Test adding Inbox to pins
        rule.onNodeWithContentDescription("Add Inbox to drawer").performScrollTo().performClick()
        assertEquals(listOf("reminders", "memory", "notifications"), pins)

        // Test Move Up / Down
        rule.onNodeWithContentDescription("Move Memory up").performClick()
        assertEquals(listOf("memory", "reminders", "notifications"), pins)

        // Recents limit group
        rule.onNodeWithText("5").assertExists()
        rule.onNodeWithText("10").assertExists()
        rule.onNodeWithText("15").assertExists()
        rule.onNodeWithText("20").assertExists()

        rule.onNodeWithText("20").performScrollTo().performClick()
        assertEquals(20, recentsLimit)

        // Timezone
        rule.onNodeWithText("UTC").assertExists()
    }
}
