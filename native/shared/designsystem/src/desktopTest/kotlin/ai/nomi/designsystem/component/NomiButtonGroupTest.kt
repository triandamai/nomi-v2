package ai.nomi.designsystem.component

import ai.nomi.designsystem.theme.NomiTheme
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import org.junit.Rule
import kotlin.test.Test
import kotlin.test.assertEquals

class NomiButtonGroupTest {
    @get:Rule val rule = createComposeRule()

    @Test
    fun buttonGroupRendersOptionsAndSelects() {
        var selected = "light"
        val options = listOf(
            ButtonGroupOption("light", "Light"),
            ButtonGroupOption("dark", "Dark"),
            ButtonGroupOption("system", "System"),
        )

        rule.setContent {
            NomiTheme {
                NomiButtonGroup(
                    options = options,
                    selected = selected,
                    onSelect = { selected = it },
                )
            }
        }

        rule.onNodeWithText("Light").assertExists()
        rule.onNodeWithText("Dark").assertExists()
        rule.onNodeWithText("System").assertExists()

        rule.onNodeWithText("Dark").performClick()
        assertEquals("dark", selected)

        rule.onNodeWithText("System").performClick()
        assertEquals("system", selected)
    }
}
