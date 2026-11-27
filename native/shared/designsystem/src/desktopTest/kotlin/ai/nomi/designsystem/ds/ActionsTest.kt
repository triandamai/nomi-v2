package ai.nomi.designsystem.ds

import ai.nomi.designsystem.generated.NomiPenColors
import ai.nomi.designsystem.icon.Lucide
import ai.nomi.designsystem.theme.NomiTheme
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.compositeOver
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.assertHeightIsEqualTo
import androidx.compose.ui.test.assertIsNotEnabled
import androidx.compose.ui.test.assertWidthIsEqualTo
import androidx.compose.ui.test.hasClickAction
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performKeyInput
import androidx.compose.ui.test.performMouseInput
import androidx.compose.ui.test.pressKey
import androidx.compose.ui.test.requestFocus
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import org.junit.Rule
import kotlin.test.Test
import kotlin.test.assertEquals

private val light = NomiPenColors.light
private fun c(argb: Long) = Color(argb)
private val ground = c(light.bg)

/** Design System / Nomi → Actions: height 30, radius-sm, 13px; one primary per view, Inverse inside panels. */
@OptIn(ExperimentalTestApi::class)
class ActionsTest {
    @get:Rule val rule = createComposeRule()

    private fun show(coarsePointer: Boolean = false, content: @Composable () -> Unit) = rule.setContent {
        NomiTheme(dark = false, coarsePointer = coarsePointer) {
            Box(Modifier.background(Ds.bg).padding(8.dp)) { content() }
        }
    }

    private fun button() = rule.onNode(hasClickAction())

    @Test
    fun `a button is 30dp tall with a 13sp label 12dp in`() {
        show { DsButton("Start building", {}) }
        button().assertHeightIsEqualTo(30.dp)
        val label = rule.onNodeWithText("Start building", useUnmergedTree = true).fetchSemanticsNode()
        val box = button().fetchSemanticsNode()
        assertEquals(12f, with(rule.density) { (label.boundsInRoot.left - box.boundsInRoot.left).toDp().value }, 0.5f)
        assertEquals(13.sp, rule.onNodeWithText("Start building", useUnmergedTree = true).textStyle().fontSize)
    }

    @Test
    fun `each variant paints its tokens`() {
        val cases = listOf(
            DsButtonVariant.Primary to c(light.accent),
            DsButtonVariant.Inverse to c(light.text),
            DsButtonVariant.Danger to c(light.dangerBg).compositeOver(ground),
            DsButtonVariant.Secondary to ground,
            DsButtonVariant.Ghost to ground,
        )
        cases.forEach { (variant, fill) ->
            rule.setContent { NomiTheme(dark = false) { Box(Modifier.background(Ds.bg).padding(8.dp)) { DsButton(variant.name, {}, variant = variant) } } }
            assertColor(fill, button().pixelAtMiddle(rule, 4.dp), "${variant.name} fill")
        }
    }

    @Test
    fun `secondary and danger draw their edge in border-strong and danger-border`() {
        show { DsButton("Add credits", {}, variant = DsButtonVariant.Secondary) }
        assertColor(c(light.borderStrong), button().pixelAtMiddle(rule, 0.5.dp), "secondary edge")
    }

    @Test
    fun `a trailing icon sits 6dp after the label at 14dp`() {
        show { DsButton("Start building", {}, iconEnd = Lucide.ArrowRight) }
        val label = rule.onNodeWithText("Start building", useUnmergedTree = true).fetchSemanticsNode().boundsInRoot
        val labelWidth = with(rule.density) { label.width.toDp() }
        button().assertWidthIsEqualTo(12.dp + labelWidth + 6.dp + 14.dp + 12.dp)
    }

    @Test
    fun `it does what it says when clicked or pressed`() {
        var clicks = 0
        show { DsButton("Open workspace", { clicks++ }) }
        button().performClick()
        button().requestFocus()
        button().performKeyInput { pressKey(Key.Enter) }
        rule.waitForIdle()
        assertEquals(2, clicks)
    }

    @Test
    fun `focus draws a 3dp ring against the edge in focus`() {
        show { DsButton("Start building", {}) }
        button().requestFocus()
        rule.waitForIdle()
        assertColor(c(light.focus).compositeOver(ground), button().pixelLeftOf(rule, 1.5.dp), "ring")
        assertColor(ground, button().pixelLeftOf(rule, 4.dp), "outside the ring")
    }

    @Test
    fun `a focused secondary trades its edge for the ring`() {
        show { DsButton("Add credits", {}, variant = DsButtonVariant.Secondary) }
        button().requestFocus()
        rule.waitForIdle()
        assertColor(ground, button().pixelAtMiddle(rule, 0.5.dp), "secondary edge while focused")
    }

    @Test
    fun `a secondary button darkens on hover`() {
        show { DsButton("Add credits", {}, variant = DsButtonVariant.Secondary) }
        button().performMouseInput { moveTo(center) }
        rule.waitForIdle()
        assertColor(c(light.surface2), button().pixelAtMiddle(rule, 4.dp), "hovered fill")
    }

    @Test
    fun `a disabled button is dimmed to 40 percent and does nothing`() {
        var clicks = 0
        show { DsButton("Start building", { clicks++ }, enabled = false) }
        button().assertIsNotEnabled().performClick()
        assertEquals(0, clicks)
        assertColor(c(light.accent).copy(alpha = 0.4f).compositeOver(ground), button().pixelAtMiddle(rule, 4.dp), "dimmed fill")
    }

    @Test
    fun `loading shows its label at full strength and takes no second click`() {
        var clicks = 0
        show { DsButton("Deploying · 00:14", { clicks++ }, variant = DsButtonVariant.Inverse, loading = true) }
        rule.onNodeWithText("Deploying · 00:14").assertIsNotEnabled().performClick()
        assertEquals(0, clicks)
        assertColor(c(light.text), button().pixelAtMiddle(rule, 4.dp), "loading fill")
    }

    @Test
    fun `on a phone a button is a 44dp target`() {
        show(coarsePointer = true) { DsButton("Start building", {}) }
        button().assertHeightIsEqualTo(44.dp)
    }

    @Test
    fun `an icon button is a named 30dp square with a border-strong edge`() {
        show { DsIconButton(Lucide.Plus, "Attach a file", {}) }
        rule.onNodeWithContentDescription("Attach a file").assertWidthIsEqualTo(30.dp).assertHeightIsEqualTo(30.dp)
        assertColor(c(light.borderStrong), rule.onNodeWithContentDescription("Attach a file").pixelAtMiddle(rule, 0.5.dp), "edge")
    }

    @Test
    fun `send is mint, and greys out rather than fading when there is nothing to send`() {
        var clicks = 0
        show { DsIconButton(Lucide.ArrowUp, "Send", { clicks++ }, variant = DsIconButtonVariant.Send) }
        assertColor(c(light.accent), rule.onNodeWithContentDescription("Send").pixelAt(rule, 3.dp, 3.dp), "send fill")
        rule.setContent {
            NomiTheme(dark = false) { Box(Modifier.background(Ds.bg).padding(8.dp)) { DsIconButton(Lucide.ArrowUp, "Send", { clicks++ }, variant = DsIconButtonVariant.Send, enabled = false) } }
        }
        rule.onNodeWithContentDescription("Send").assertIsNotEnabled().performClick()
        assertEquals(0, clicks)
        assertColor(c(light.surface2), rule.onNodeWithContentDescription("Send").pixelAt(rule, 3.dp, 3.dp), "empty send fill")
    }

    @Test
    fun `on a phone an icon button is a 44dp square`() {
        show(coarsePointer = true) { DsIconButton(Lucide.Plus, "Attach a file", {}) }
        rule.onNodeWithContentDescription("Attach a file").assertWidthIsEqualTo(44.dp).assertHeightIsEqualTo(44.dp)
    }

    @Test
    fun `a key is a 20dp keycap with a border-strong edge`() {
        show { Kbd("⌘K") }
        val key = rule.onNodeWithText("⌘K")
        key.assertHeightIsEqualTo(20.dp)
        assertColor(c(light.borderStrong), key.pixelAtMiddle(rule, 0.5.dp), "keycap edge")
    }
}
