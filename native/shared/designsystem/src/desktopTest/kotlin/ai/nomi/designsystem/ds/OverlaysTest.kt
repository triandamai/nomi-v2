package ai.nomi.designsystem.ds

import ai.nomi.designsystem.generated.NomiPenColors
import ai.nomi.designsystem.icon.Lucide
import ai.nomi.designsystem.theme.NomiTheme
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.assertHeightIsEqualTo
import androidx.compose.ui.test.assertIsFocused
import androidx.compose.ui.test.assertIsSelected
import androidx.compose.ui.test.assertWidthIsEqualTo
import androidx.compose.ui.test.hasText
import androidx.compose.ui.test.isDialog
import androidx.compose.ui.test.isPopup
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import androidx.compose.ui.test.performKeyInput
import androidx.compose.ui.test.pressKey
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import org.junit.Rule
import kotlin.test.Test
import kotlin.test.assertEquals

private val light = NomiPenColors.light
private fun c(argb: Long) = Color(argb)

/** Design System / Nomi → Overlays: frontend-new `ds/overlays`, case by case. */
@OptIn(ExperimentalTestApi::class)
class OverlaysTest {
    @get:Rule val rule = createComposeRule()

    private fun show(width: Int = 600, content: @Composable () -> Unit) = rule.setContent {
        NomiTheme(dark = false) { Box(Modifier.background(Ds.bg).padding(16.dp).width(width.dp)) { content() } }
    }

    @Test
    fun `a dialog panel is 420 on surface with a border-strong edge and a 15 medium title`() {
        show { DsDialog(open = true, onDismiss = {}, title = "Delete project?", description = "warung-kopi-menu and its deploys go away.", inline = true) }
        val panel = rule.onNodeWithTag(DS_DIALOG_TAG)
        panel.assertWidthIsEqualTo(420.dp)
        assertColor(c(light.surface), panel.pixelAt(rule, 12.dp, 12.dp), "panel fill")
        assertColor(c(light.borderStrong), panel.pixelAt(rule, 0.5.dp, 40.dp), "panel edge")
        val title = rule.onNodeWithText("Delete project?", useUnmergedTree = true).textStyle()
        assertEquals(15.sp, title.fontSize)
        assertEquals(androidx.compose.ui.text.font.FontWeight.Medium, title.fontWeight)
        assertEquals(c(light.text2), rule.onNodeWithText("warung-kopi-menu and its deploys go away.", useUnmergedTree = true).textStyle().color)
    }

    @Test
    fun `a wide dialog is 440`() {
        show { DsDialog(open = true, onDismiss = {}, title = "Move chat", wide = true, inline = true) }
        rule.onNodeWithTag(DS_DIALOG_TAG).assertWidthIsEqualTo(440.dp)
    }

    @Test
    fun `a dialog opens in its own layer and Escape closes it`() {
        var open by mutableStateOf(true)
        show { DsDialog(open, { open = false }, "Delete project?", actions = { DsButton("Delete", {}, variant = DsButtonVariant.Danger) }) }
        rule.onNode(isDialog()).assertExists()
        rule.onNodeWithText("Delete").performKeyInput { pressKey(Key.Escape) }
        rule.waitForIdle()
        assertEquals(false, open)
    }

    private val projectMenu = listOf(
        MenuEntry.Item("Export repo", icon = Lucide.Download),
        MenuEntry.Item("Visit live site", icon = Lucide.ExternalLink),
        MenuEntry.Divider,
        MenuEntry.Item("Delete project", icon = Lucide.Trash, danger = true),
    )

    @Test
    fun `a menu panel is 236 wide with 30 tall rows, danger in red`() {
        show { DsMenu(projectMenu, inline = true) }
        rule.onNodeWithTag(DS_MENU_TAG).assertWidthIsEqualTo(236.dp)
        rule.onNodeWithText("Export repo").assertHeightIsEqualTo(30.dp)
        assertEquals(c(light.danger), rule.onNodeWithText("Delete project", useUnmergedTree = true).textStyle().color)
    }

    @Test
    fun `a menu opens from its button, focuses its first item, and the keyboard moves and picks`() {
        val picked = mutableListOf<String>()
        val items = projectMenu.map { if (it is MenuEntry.Item) it.copy(onSelect = { picked += it.label }) else it }
        show {
            DsMenu(items) { open, _ -> DsButton("Project actions", open, variant = DsButtonVariant.Secondary) }
        }
        rule.onNodeWithText("Project actions").performClick()
        rule.onNode(isPopup()).assertExists()
        rule.onNodeWithText("Export repo").assertIsFocused()
        rule.onNodeWithText("Export repo").performKeyInput { pressKey(Key.DirectionDown) }
        rule.onNodeWithText("Visit live site").assertIsFocused()
        rule.onNodeWithText("Visit live site").performKeyInput { pressKey(Key.MoveEnd) }
        rule.onNodeWithText("Delete project").assertIsFocused()
        rule.onNodeWithText("Delete project").performKeyInput { pressKey(Key.DirectionDown) }
        rule.onNodeWithText("Export repo").assertIsFocused()
        rule.onNodeWithText("Export repo").performKeyInput { pressKey(Key.DirectionUp); pressKey(Key.Enter) }
        rule.waitForIdle()
        assertEquals(listOf("Delete project"), picked)
        rule.onNode(isPopup()).assertDoesNotExist()
    }

    @Test
    fun `Escape closes a menu, choosing nothing`() {
        val picked = mutableListOf<String>()
        show { DsMenu(projectMenu.map { if (it is MenuEntry.Item) it.copy(onSelect = { picked += it.label }) else it }) { open, _ -> DsButton("Project actions", open) } }
        rule.onNodeWithText("Project actions").performClick()
        rule.onNodeWithText("Export repo").performKeyInput { pressKey(Key.Escape) }
        rule.waitForIdle()
        rule.onNode(isPopup()).assertDoesNotExist()
        assertEquals(emptyList(), picked)
    }

    @Test
    fun `a toast says what happened on surface, its detail in mono, with one action and a close`() {
        var undone = 0
        var dismissed = 0
        show { DsToast(ToastKind.Success, "Deployed", detail = "warung-kopi-menu is live", actionLabel = "View", onAction = { undone++ }, onDismiss = { dismissed++ }) }
        assertEquals(c(light.text), rule.onNodeWithText("Deployed", useUnmergedTree = true).textStyle().color)
        val detail = rule.onNodeWithText("warung-kopi-menu is live", useUnmergedTree = true).textStyle()
        assertEquals(11.sp, detail.fontSize)
        assertEquals(c(light.text3), detail.color)
        rule.onNodeWithText("View").performClick()
        rule.onNodeWithContentDescription("Dismiss").performClick()
        assertEquals(1 to 1, undone to dismissed)
        assertColor(c(light.surface), rule.onNodeWithTag(DS_TOAST_TAG).pixelAt(rule, 6.dp, 6.dp), "toast fill")
    }

    @Test
    fun `toasts go after six seconds, errors stay, and a toast can keep its own time`() {
        val toasts = DsToasts()
        rule.mainClock.autoAdvance = false
        show { DsToastRegion(toasts) }
        toasts.show(ToastKind.Info, "Repo exported: salah-daily-tracker.zip", detail = "23 files · 184 KB")
        toasts.show(ToastKind.Error, "Deploy failed")
        toasts.show(ToastKind.Info, "Reply deleted", actionLabel = "Undo", onAction = {}, durationMs = 10_000)
        rule.mainClock.advanceTimeBy(6_100)
        rule.onNodeWithText("Repo exported: salah-daily-tracker.zip").assertDoesNotExist()
        rule.onNodeWithText("Deploy failed").assertExists()
        rule.onNodeWithText("Reply deleted").assertExists()
        rule.mainClock.advanceTimeBy(4_000)
        rule.onNodeWithText("Reply deleted").assertDoesNotExist()
    }

    @Test
    fun `a tooltip bubble is inverted, 12 on text, with its shortcut in mono`() {
        show { TooltipBubble("New chat", shortcut = "⌘N") }
        assertEquals(c(light.bg), rule.onNodeWithText("New chat", useUnmergedTree = true).textStyle().color)
        assertEquals(12.sp, rule.onNodeWithText("New chat", useUnmergedTree = true).textStyle().fontSize)
        assertEquals(c(light.text3), rule.onNodeWithText("⌘N", useUnmergedTree = true).textStyle().color)
    }

    @Test
    fun `a bottom sheet rises with its title and closes from its header`() {
        var open by mutableStateOf(true)
        show { DsBottomSheet(open, { open = false }, "Share warung-kopi-menu") { DsSheetOption("Copy link", onClick = {}) } }
        rule.onNodeWithText("Share warung-kopi-menu").assertExists()
        rule.onNodeWithContentDescription("Close").performClick()
        rule.waitForIdle()
        assertEquals(false, open)
    }

    @Test
    fun `an action sheet's actions are 56 tall, danger in red, and cancel closes it`() {
        var open by mutableStateOf(true)
        var deleted = 0
        show {
            DsActionSheet(open, { open = false }, "Delete this chat?", actions = listOf(SheetAction("Delete chat", danger = true) { deleted++ }), cancel = "Cancel", body = "Its replies go too.")
        }
        rule.onNodeWithText("Delete chat").assertHeightIsEqualTo(56.dp)
        assertEquals(c(light.danger), rule.onNodeWithText("Delete chat", useUnmergedTree = true).textStyle().color)
        rule.onNodeWithText("Cancel").performClick()
        rule.waitForIdle()
        assertEquals(false to 0, open to deleted)
    }

    @Test
    fun `a sheet option is 52 tall, or 60 with a detail, and says when it's chosen`() {
        show {
            androidx.compose.foundation.layout.Column {
                DsSheetOption("Bahasa Indonesia", onClick = {}, chosen = true)
                DsSheetOption("English", onClick = {}, detail = "Used for the crew's replies")
            }
        }
        rule.onNodeWithText("Bahasa Indonesia").assertHeightIsEqualTo(52.dp).assertIsSelected()
        rule.onNode(hasText("English")).assertHeightIsEqualTo(60.dp)
        assertEquals(16.sp, rule.onNodeWithText("Bahasa Indonesia", useUnmergedTree = true).textStyle().fontSize)
    }

    @Test
    fun `sheet tiles are 68 tall on surface-2 and a pressed one is mint`() {
        show(width = 360) { DsSheetTiles(listOf(SheetTile(Lucide.Copy, "Copy link") {}, SheetTile(Lucide.Bell, "Notify", pressed = true) {})) }
        rule.onNodeWithText("Copy link").assertHeightIsEqualTo(68.dp)
        assertColor(c(light.surface2), rule.onNodeWithText("Copy link").pixelAt(rule, 6.dp, 6.dp), "tile fill")
        assertEquals(c(light.accent), rule.onNodeWithText("Notify", useUnmergedTree = true).textStyle().color)
    }

    @Test
    fun `a phone toast region shows toasts on top and a tap on one lets it go`() {
        val toasts = DsToasts()
        show(width = 360) { DsPhoneToastRegion(toasts) }
        toasts.show(ToastKind.Success, "Published", detail = "warung-kopi-menu.nomi.app")
        rule.waitForIdle()
        rule.onNodeWithText("Published").performClick()
        rule.waitForIdle()
        rule.onNodeWithText("Published").assertDoesNotExist()
    }

    private fun androidx.compose.ui.test.junit4.ComposeContentTestRule.onNodeWithTag(tag: String) =
        onNode(androidx.compose.ui.test.hasTestTag(tag), useUnmergedTree = true)
}
