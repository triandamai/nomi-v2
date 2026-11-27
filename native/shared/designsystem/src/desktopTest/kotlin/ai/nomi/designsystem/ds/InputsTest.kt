package ai.nomi.designsystem.ds

import ai.nomi.designsystem.generated.NomiPenColors

import ai.nomi.designsystem.theme.NomiTheme
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.compositeOver
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.SemanticsMatcher
import androidx.compose.ui.test.assertHeightIsEqualTo
import androidx.compose.ui.test.assertIsNotEnabled
import androidx.compose.ui.test.assertIsOff
import androidx.compose.ui.test.assertIsOn
import androidx.compose.ui.test.assertIsSelected
import androidx.compose.ui.test.assertWidthIsEqualTo
import androidx.compose.ui.test.hasSetTextAction
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithTag
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performKeyInput
import androidx.compose.ui.test.performTextInput
import androidx.compose.ui.test.pressKey
import androidx.compose.ui.test.requestFocus
import androidx.compose.ui.test.withKeyDown
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import org.junit.Rule
import kotlin.test.Test
import kotlin.test.assertEquals

private val light = NomiPenColors.light
private fun c(argb: Long) = Color(argb)
private val ground = c(light.bg)

private fun hasRole(role: Role) = SemanticsMatcher.expectValue(SemanticsProperties.Role, role)

/** Design System / Nomi → Inputs: frontend-new `ds/inputs`, case by case. */
@OptIn(ExperimentalTestApi::class)
class InputsTest {
    @get:Rule val rule = createComposeRule()

    private fun show(coarsePointer: Boolean = false, content: @Composable () -> Unit) = rule.setContent {
        NomiTheme(dark = false, coarsePointer = coarsePointer) {
            Box(Modifier.background(Ds.bg).padding(8.dp).width(480.dp)) { content() }
        }
    }

    @Test
    fun `a check mark is a 16dp box that fills in text colour when checked`() {
        show {
            var on by remember { mutableStateOf(false) }
            CheckMark(on, { on = it })
        }
        val box = rule.onNode(hasRole(Role.Checkbox))
        box.assertWidthIsEqualTo(16.dp).assertHeightIsEqualTo(16.dp).assertIsOff()
        assertColor(c(light.borderStrong), box.pixelAtMiddle(rule, 0.5.dp), "unchecked edge")
        box.performClick().assertIsOn()
        assertColor(c(light.text), box.pixelAt(rule, 1.5.dp, 1.5.dp), "checked fill")
    }

    @Test
    fun `a checkbox toggles from its 13sp label`() {
        var checked = false
        show { DsCheckbox("Email me when a deploy fails", checked, { checked = it }) }
        assertEquals(13.sp, rule.onNodeWithText("Email me when a deploy fails", useUnmergedTree = true).textStyle().fontSize)
        rule.onNodeWithText("Email me when a deploy fails").performClick()
        assertEquals(true, checked)
    }

    @Test
    fun `a toggle is a 32 by 18 switch, mint when on`() {
        show {
            var on by remember { mutableStateOf(false) }
            DsToggle("Auto-deploy", on, { on = it })
        }
        val toggle = rule.onNodeWithContentDescription("Auto-deploy")
        toggle.assertWidthIsEqualTo(32.dp).assertHeightIsEqualTo(18.dp).assertIsOff()
        assertColor(c(light.surface2), toggle.pixelAt(rule, 24.dp, 9.dp), "off fill")
        toggle.performClick().assertIsOn()
        assertColor(c(light.accent), toggle.pixelAt(rule, 6.dp, 9.dp), "on fill")
    }

    @Test
    fun `a chip is a 30dp surface-2 pick that opens a menu`() {
        var opened = 0
        show { DsChip(ChipKind.Model, "deepseek-v3", onClick = { opened++ }) }
        val chip = rule.onNode(hasRole(Role.DropdownList))
        chip.assertHeightIsEqualTo(30.dp)
        assertColor(c(light.surface2), chip.pixelAt(rule, 3.dp, 15.dp), "chip fill")
        chip.performClick()
        assertEquals(1, opened)
        assertEquals(12.sp, rule.onNodeWithText("deepseek-v3", useUnmergedTree = true).textStyle().fontSize)
    }

    @Test
    fun `a provision chip is a 24dp mono label`() {
        show { DsChip(ChipKind.Provision, "warung-kopi-db") }
        rule.onNodeWithText("warung-kopi-db").assertHeightIsEqualTo(24.dp)
    }

    @Test
    fun `on a phone a chip is a 44dp target`() {
        show(coarsePointer = true) { DsChip(ChipKind.Model, "deepseek-v3", onClick = {}) }
        rule.onNode(hasRole(Role.DropdownList)).assertHeightIsEqualTo(44.dp)
    }

    @Test
    fun `search is a quiet 30dp box on the plain border that turns mint when focused`() {
        show {
            var text by remember { mutableStateOf("") }
            DsSearch(text, { text = it }, "Search chats", shortcut = "⌘K")
        }
        val box = rule.onNodeWithContentDescription("Search chats")
        box.assertHeightIsEqualTo(30.dp)
        assertColor(c(light.border), box.pixelAtMiddle(rule, 0.5.dp), "resting edge")
        rule.onNode(hasSetTextAction()).requestFocus()
        rule.waitForIdle()
        assertColor(c(light.accent), box.pixelAtMiddle(rule, 0.5.dp), "focused edge")
    }

    @Test
    fun `a phone search is 38dp on surface-2 with 16sp text`() {
        show {
            var text by remember { mutableStateOf("") }
            DsPhoneSearch(text, { text = it }, "Search")
        }
        val box = rule.onNodeWithContentDescription("Search")
        box.assertHeightIsEqualTo(38.dp)
        assertColor(c(light.surface2), box.pixelAt(rule, 4.dp, 19.dp), "fill")
        rule.onNode(hasSetTextAction()).performTextInput("warung")
        assertEquals(16.sp, rule.onNode(hasSetTextAction()).textStyle().fontSize)
    }

    @Test
    fun `segmented marks the chosen view on bg and arrow keys move the choice`() {
        val options = listOf(SegmentOption("all", "All", "4"), SegmentOption("live", "Live", "2"), SegmentOption("building", "Building", "1"))
        var picked = "all"
        show {
            var value by remember { mutableStateOf("all") }
            DsSegmented("Projects", options, value, { value = it; picked = it })
        }
        val all = rule.onNode(hasRole(Role.RadioButton) and androidx.compose.ui.test.hasText("All", substring = true))
        all.assertIsSelected().assertHeightIsEqualTo(26.dp)
        assertColor(c(light.bg), all.pixelAt(rule, 4.dp, 13.dp), "chosen fill")
        assertColor(c(light.borderStrong), all.pixelAt(rule, 0.5.dp, 13.dp), "chosen edge")
        all.requestFocus()
        all.performKeyInput { pressKey(Key.DirectionRight) }
        rule.waitForIdle()
        assertEquals("live", picked)
    }

    @Test
    fun `tabs are 40dp with a text-colour line under the open one`() {
        val tabs = listOf(TabItem("chat", "Chat"), TabItem("files", "Files"), TabItem("deploys", "Deploys"))
        var open = "chat"
        show {
            var value by remember { mutableStateOf("chat") }
            DsTabs("Workspace", tabs, value, { value = it; open = it })
        }
        val chat = rule.onNode(hasRole(Role.Tab) and androidx.compose.ui.test.hasText("Chat"))
        chat.assertIsSelected().assertHeightIsEqualTo(40.dp)
        assertColor(c(light.text), chat.pixelAt(rule, 2.dp, 39.5.dp), "open line")
        chat.requestFocus()
        chat.performKeyInput { pressKey(Key.DirectionRight) }
        rule.waitForIdle()
        assertEquals("files", open)
    }

    @Test
    fun `a text field is a 34dp box on surface with a border-strong edge and a mint caret edge when focused`() {
        show {
            var text by remember { mutableStateOf("") }
            DsTextField("Project name", text, { text = it }, helper = "Lowercase, no spaces")
        }
        assertEquals(13.sp, rule.onNodeWithText("Project name", useUnmergedTree = true).textStyle().fontSize)
        val field = rule.onNodeWithTag(DS_FIELD_TAG)
        field.assertHeightIsEqualTo(34.dp)
        assertColor(c(light.surface), field.pixelAt(rule, 4.dp, 17.dp), "field fill")
        assertColor(c(light.borderStrong), field.pixelAt(rule, 0.5.dp, 17.dp), "resting edge")
        rule.onNode(hasSetTextAction()).requestFocus()
        rule.waitForIdle()
        assertColor(c(light.accent), field.pixelAt(rule, 0.5.dp, 17.dp), "focused edge")
        assertEquals(12.sp, rule.onNodeWithText("Lowercase, no spaces", useUnmergedTree = true).textStyle().fontSize)
    }

    @Test
    fun `an error swaps the edge and the helper for what to do next`() {
        show { DsTextField("Project name", "Warung Kopi", {}, helper = "Lowercase, no spaces", error = "Use lowercase letters and dashes") }
        val field = rule.onNodeWithTag(DS_FIELD_TAG)
        assertColor(c(light.danger), field.pixelAt(rule, 0.5.dp, 17.dp), "error edge")
        rule.onNodeWithText("Use lowercase letters and dashes").assertExists()
        rule.onNodeWithText("Lowercase, no spaces").assertDoesNotExist()
        rule.onNode(SemanticsMatcher.keyIsDefined(SemanticsProperties.Error)).assertExists()
    }

    @Test
    fun `a select is a 30dp field that lists its options and says the one chosen`() {
        val options = listOf(SelectOption("id", "Bahasa Indonesia"), SelectOption("en", "English"))
        var chosen = "id"
        show {
            var value by remember { mutableStateOf("id") }
            DsSelect("Language", options, value, { value = it; chosen = it })
        }
        val select = rule.onNode(hasRole(Role.DropdownList))
        select.assertHeightIsEqualTo(30.dp)
        assertColor(c(light.borderStrong), select.pixelAt(rule, 0.5.dp, 15.dp), "edge")
        rule.onNodeWithText("Bahasa Indonesia").assertExists()
        select.performClick()
        rule.onNodeWithText("English").performClick()
        assertEquals("en", chosen)
    }

    @Test
    fun `the composer sends on Enter, keeps Shift-Enter as a new line, and holds Send until there is text`() {
        val sent = mutableListOf<String>()
        show { DsComposer("Ask the crew…", ComposerAgent(Crew.Nomi, "Nomi"), "deepseek-v3", onSend = { sent += it }) }
        rule.onNodeWithContentDescription("Send").assertIsNotEnabled()
        val field = rule.onNode(hasSetTextAction())
        field.performTextInput("  Find Whoosh trains for Sat 1 Nov  ")
        field.performKeyInput { withKeyDown(Key.ShiftLeft) { pressKey(Key.Enter) } }
        assertEquals(emptyList(), sent)
        field.performKeyInput { pressKey(Key.Enter) }
        rule.waitForIdle()
        assertEquals(1, sent.size)
        assertEquals("Find Whoosh trains for Sat 1 Nov", sent.single().trim())
        rule.onNodeWithContentDescription("Send").assertIsNotEnabled()
    }
}
