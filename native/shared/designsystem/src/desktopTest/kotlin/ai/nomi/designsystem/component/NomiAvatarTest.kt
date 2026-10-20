package ai.nomi.designsystem.component

import ai.nomi.designsystem.theme.NomiTheme
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithText
import org.junit.Rule
import kotlin.test.Test
import kotlin.test.assertEquals

class NomiAvatarTest {
    @get:Rule val rule = createComposeRule()

    @Test
    fun computeInitialsExtractsFirstTwoParts() {
        assertEquals("AL", computeInitials("Ana Lima"))
        assertEquals("JD", computeInitials("John Doe Jr"))
        assertEquals("A", computeInitials("Alice"))
        assertEquals("?", computeInitials(""))
        assertEquals("?", computeInitials("   "))
    }

    @Test
    fun avatarRendersInitialsWhenNoUrl() {
        rule.setContent {
            NomiTheme {
                NomiAvatar(name = "Ana Lima")
            }
        }
        rule.onNodeWithText("AL").assertExists()
    }
}
