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
import androidx.compose.ui.test.performScrollTo
import org.junit.Rule
import kotlin.test.Test
import kotlin.test.assertEquals

class SettingsPickerTest {
    @get:Rule val rule = createComposeRule()

    @Test
    fun appearancePickerRendersModesAndSelectsTheme() {
        var selectedMode by androidx.compose.runtime.mutableStateOf("system")
        var selectedAccent by androidx.compose.runtime.mutableStateOf("canopy")

        rule.setContent {
            NomiTheme {
                Box(modifier = Modifier.verticalScroll(rememberScrollState())) {
                    AppearancePicker(
                        mode = selectedMode,
                        accentColor = selectedAccent,
                        onSelectMode = { selectedMode = it },
                        onSelectAccentColor = { selectedAccent = it },
                    )
                }
            }
        }

        // Modes
        rule.onNodeWithText("Light").assertExists()
        rule.onNodeWithText("Dark").assertExists()
        rule.onNodeWithText("System").assertExists()

        rule.onNodeWithText("Dark").performClick()
        assertEquals("dark", selectedMode)

        // Themes
        rule.onNodeWithText("Canopy").assertExists()
        rule.onNodeWithText("Coral Reef").assertExists()

        rule.onNodeWithText("Coral Reef").performClick()
        assertEquals("coral-reef", selectedAccent)

        // Expand all
        rule.onNodeWithText("All themes").performScrollTo().performClick()
        rule.onNodeWithText("Lautan").assertExists()
        rule.onNodeWithText("Matcha").assertExists()
        rule.onNodeWithText("Graphite").assertExists()

        rule.onNodeWithText("Graphite").performScrollTo().performClick()
        assertEquals("graphite", selectedAccent)
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
