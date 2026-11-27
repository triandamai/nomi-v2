package ai.nomi.feature.settings

import ai.nomi.designsystem.theme.NomiTheme
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.getValue
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import org.junit.Rule
import kotlin.test.Test
import kotlin.test.assertEquals

class SettingsPickerTest {
    @get:Rule val rule = createComposeRule()

    @Test
    fun appearancePickerOffersOnlyTheMode() {
        var selectedMode by androidx.compose.runtime.mutableStateOf("system")

        rule.setContent {
            NomiTheme {
                Box(modifier = Modifier.verticalScroll(rememberScrollState())) {
                    AppearancePicker(mode = selectedMode, onSelectMode = { selectedMode = it })
                }
            }
        }

        rule.onNodeWithText("Light").assertExists()
        rule.onNodeWithText("Dark").assertExists()
        rule.onNodeWithText("System").assertExists()

        rule.onNodeWithText("Dark").performClick()
        assertEquals("dark", selectedMode)

        // The pen has one accent, so there is no colour theme to pick.
        rule.onNodeWithText("COLOR THEME").assertDoesNotExist()
        rule.onNodeWithText("Canopy").assertDoesNotExist()
        rule.onNodeWithText("All themes").assertDoesNotExist()
    }

    @Test
    fun homeLayoutPickerRendersOptionsAndSelects() {
        var selectedLayout = "stage"

        rule.setContent {
            NomiTheme {
                HomeLayoutPicker(
                    current = selectedLayout,
                    onSelectLayout = { selectedLayout = it },
                )
            }
        }

        rule.onNodeWithText("Stage").assertExists()
        rule.onNodeWithText("Classic").assertExists()
        rule.onNodeWithText("Today").assertExists()

        rule.onNodeWithText("Classic").performClick()
        assertEquals("classic", selectedLayout)

        rule.onNodeWithText("Today").performClick()
        assertEquals("today", selectedLayout)
    }
}
