package ai.nomi.designsystem.ds

import ai.nomi.designsystem.icon.Lucide
import ai.nomi.designsystem.icon.NomiGlyph
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.MutableTransitionState
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.scaleIn
import androidx.compose.animation.scaleOut
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutVertically
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.layout.width
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.PlainTooltip
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TooltipBox
import androidx.compose.material3.TooltipDefaults
import androidx.compose.material3.rememberTooltipState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.Stable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.semantics.LiveRegionMode
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.liveRegion
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import kotlinx.coroutines.delay

private val DialogWidth = 400.dp
private val DialogWideWidth = 520.dp
private val MenuWidth = 220.dp
private val MenuWideWidth = 280.dp
private const val TOAST_MS = 5000L

/** The dialog's panel: title, description, body, then its actions on the right (web DialogPanel). */
@Composable
fun DialogPanel(
    title: String,
    modifier: Modifier = Modifier,
    description: String? = null,
    wide: Boolean = false,
    actions: (@Composable () -> Unit)? = null,
    content: (@Composable ColumnScope.() -> Unit)? = null,
) {
    Surface(
        modifier.widthIn(max = if (wide) DialogWideWidth else DialogWidth).fillMaxWidth(),
        shape = MaterialTheme.shapes.extraLarge,
        color = MaterialTheme.colorScheme.surfaceContainerHigh,
        tonalElevation = 6.dp,
    ) {
        Column(Modifier.padding(24.dp), verticalArrangement = Arrangement.spacedBy(16.dp)) {
            Column(verticalArrangement = Arrangement.spacedBy(16.dp)) {
                Text(title, style = MaterialTheme.typography.headlineSmall, color = Ds.text, modifier = Modifier.semantics { heading() })
                if (description != null) Text(description, style = MaterialTheme.typography.bodyMedium, color = Ds.text2)
                content?.invoke(this)
            }
            if (actions != null) {
                FlowRow(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp, Alignment.End)) { actions() }
            }
        }
    }
}

/**
 * A modal dialog (web Dialog): scales and fades in over a scrim, closes on the scrim or back. [inline] draws only
 * the panel in place, as the design sheet shows it.
 */
@Composable
fun DsDialog(
    open: Boolean,
    onDismiss: () -> Unit,
    title: String,
    modifier: Modifier = Modifier,
    description: String? = null,
    wide: Boolean = false,
    inline: Boolean = false,
    actions: (@Composable () -> Unit)? = null,
    content: (@Composable ColumnScope.() -> Unit)? = null,
) {
    if (inline) {
        DialogPanel(title, modifier, description, wide, actions, content)
        return
    }
    if (!open) return
    Dialog(onDismissRequest = onDismiss, properties = DialogProperties(usePlatformDefaultWidth = false)) {
        val shown = remember { MutableTransitionState(false) }.apply { targetState = true }
        AnimatedVisibility(
            shown,
            enter = scaleIn(DsMotion.spatial(), initialScale = DIALOG_START_SCALE) + fadeIn(DsMotion.effects()),
            exit = scaleOut(DsMotion.fast()) + fadeOut(DsMotion.fast()),
        ) {
            Box(Modifier.padding(24.dp)) { DialogPanel(title, modifier, description, wide, actions, content) }
        }
    }
}

private const val DIALOG_START_SCALE = 0.92f

/** A menu's entries: a row, a hairline, or a group heading. */
sealed interface MenuEntry {
    data class Item(
        val label: String,
        val icon: NomiGlyph? = null,
        val shortcut: String? = null,
        val meta: String? = null,
        val danger: Boolean = false,
        val onSelect: () -> Unit = {},
    ) : MenuEntry

    data object Divider : MenuEntry
    data class Group(val label: String) : MenuEntry
}

@Composable
private fun MenuRow(item: MenuEntry.Item, wide: Boolean, onPick: (MenuEntry.Item) -> Unit) {
    val ink = if (item.danger) Ds.danger else Ds.text
    Row(
        Modifier
            .fillMaxWidth()
            .heightIn(min = if (wide) 48.dp else 44.dp)
            .clickable(role = Role.Button) { onPick(item) }
            .padding(horizontal = 12.dp),
        horizontalArrangement = Arrangement.spacedBy(12.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        if (item.icon != null) DsIcon(item.icon, 20.dp, if (item.danger) Ds.danger else Ds.text2)
        Text(item.label, style = MaterialTheme.typography.labelLarge, color = ink, modifier = Modifier.weight(1f))
        item.meta?.let { Text(it, style = Ds.label, color = Ds.text2) }
        item.shortcut?.let { Text(it, style = Ds.label, color = Ds.text2) }
    }
}

/** A menu's rows with their groups, dividers and footer (web MenuPanel). */
@Composable
fun MenuPanel(
    items: List<MenuEntry>,
    modifier: Modifier = Modifier,
    title: String? = null,
    footer: String? = null,
    wide: Boolean = false,
    onPick: (MenuEntry.Item) -> Unit = { it.onSelect() },
) {
    Column(modifier.width(if (wide) MenuWideWidth else MenuWidth).padding(vertical = 8.dp)) {
        if (title != null) MenuHeading(title)
        items.forEach { entry ->
            when (entry) {
                is MenuEntry.Item -> MenuRow(entry, wide, onPick)
                MenuEntry.Divider -> DsRule(Modifier.padding(vertical = 8.dp))
                is MenuEntry.Group -> MenuHeading(entry.label)
            }
        }
        if (footer != null) {
            DsRule(Modifier.padding(vertical = 8.dp))
            Text(footer, style = Ds.caption, color = Ds.text2, modifier = Modifier.padding(horizontal = 12.dp, vertical = 4.dp))
        }
    }
}

@Composable
private fun MenuHeading(text: String) {
    Text(text.uppercase(), style = Ds.labelCaps, color = Ds.text2, modifier = Modifier.padding(horizontal = 12.dp, vertical = 8.dp))
}

/**
 * A menu that opens from its [trigger] (web Menu): the M3 dropdown, growing from the trigger. [inline] draws the
 * panel open in place, as the design sheet shows it.
 */
@Composable
fun DsMenu(
    items: List<MenuEntry>,
    modifier: Modifier = Modifier,
    title: String? = null,
    footer: String? = null,
    wide: Boolean = false,
    inline: Boolean = false,
    trigger: @Composable (open: () -> Unit, expanded: Boolean) -> Unit = { _, _ -> },
) {
    if (inline) {
        Surface(modifier, shape = MaterialTheme.shapes.large, color = MaterialTheme.colorScheme.surfaceContainer, tonalElevation = 3.dp, shadowElevation = 3.dp) {
            MenuPanel(items, title = title, footer = footer, wide = wide)
        }
        return
    }
    var expanded by remember { mutableStateOf(false) }
    Box(modifier) {
        trigger({ expanded = !expanded }, expanded)
        DropdownMenu(expanded, onDismissRequest = { expanded = false }, shape = MaterialTheme.shapes.large) {
            MenuPanel(items, title = title, footer = footer, wide = wide) { item ->
                expanded = false
                item.onSelect()
            }
        }
    }
}

enum class ToastKind { Success, Error, Info }

/** A toast: its mark, what happened, an optional action and a close (web Toast, on the inverse surface). */
@Composable
fun DsToast(
    kind: ToastKind,
    title: String,
    modifier: Modifier = Modifier,
    detail: String? = null,
    icon: NomiGlyph? = null,
    actionLabel: String? = null,
    onAction: (() -> Unit)? = null,
    onDismiss: (() -> Unit)? = null,
) {
    val colors = MaterialTheme.colorScheme
    val glyph = icon ?: when (kind) {
        ToastKind.Success -> Lucide.Check
        ToastKind.Error -> Lucide.Alert
        ToastKind.Info -> Lucide.Info
    }
    val mark = when (kind) {
        ToastKind.Success -> colors.inversePrimary
        ToastKind.Error -> colors.errorContainer
        ToastKind.Info -> colors.inverseOnSurface
    }
    Row(
        modifier
            .widthIn(max = 480.dp)
            .clip(MaterialTheme.shapes.small)
            .background(colors.inverseSurface)
            .padding(start = 16.dp, end = 8.dp, top = 8.dp, bottom = 8.dp)
            .heightIn(min = 48.dp)
            .semantics { liveRegion = if (kind == ToastKind.Error) LiveRegionMode.Assertive else LiveRegionMode.Polite },
        horizontalArrangement = Arrangement.spacedBy(12.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        DsIcon(glyph, 20.dp, mark)
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
            Text(title, style = MaterialTheme.typography.bodyMedium, color = colors.inverseOnSurface)
            if (detail != null) Text(detail, style = Ds.caption, color = colors.inverseOnSurface.copy(alpha = TOAST_DETAIL_ALPHA))
        }
        if (actionLabel != null && onAction != null) {
            Box(Modifier.heightIn(min = MinTouch).clip(DsShapes.full).clickable(role = Role.Button, onClick = onAction).padding(horizontal = 12.dp), contentAlignment = Alignment.Center) {
                Text(actionLabel, style = MaterialTheme.typography.labelLarge, color = colors.inversePrimary)
            }
        }
        if (onDismiss != null) DsBareIcon(Lucide.X, "Dismiss", onDismiss, size = 20.dp, tint = colors.inverseOnSurface)
    }
}

private const val TOAST_DETAIL_ALPHA = 0.8f

/** One queued toast. */
data class DsToastItem(
    val id: Long,
    val kind: ToastKind,
    val title: String,
    val detail: String? = null,
    val actionLabel: String? = null,
    val onAction: (() -> Unit)? = null,
)

/** The toasts on screen, newest last (web `toasts`). Each leaves after five seconds unless dismissed first. */
@Stable
class DsToasts {
    val items = mutableStateListOf<DsToastItem>()
    private var next = 0L

    fun show(kind: ToastKind, title: String, detail: String? = null, actionLabel: String? = null, onAction: (() -> Unit)? = null) {
        items += DsToastItem(next++, kind, title, detail, actionLabel, onAction)
    }

    fun dismiss(id: Long) {
        items.removeAll { it.id == id }
    }
}

/** Where toasts appear: each slides up and fades in, and slides away when it goes. */
@Composable
fun DsToastRegion(toasts: DsToasts, modifier: Modifier = Modifier) {
    Column(modifier.semantics { contentDescription = "Notifications" }, verticalArrangement = Arrangement.spacedBy(8.dp), horizontalAlignment = Alignment.CenterHorizontally) {
        toasts.items.forEach { toast ->
            androidx.compose.runtime.key(toast.id) {
                val state = remember { MutableTransitionState(false) }.apply { targetState = true }
                LaunchedEffect(toast.id) {
                    delay(TOAST_MS)
                    toasts.dismiss(toast.id)
                }
                AnimatedVisibility(
                    state,
                    enter = slideInVertically(DsMotion.spatial()) { it } + fadeIn(DsMotion.effects()),
                    exit = slideOutVertically(DsMotion.fast()) { it } + fadeOut(DsMotion.fast()),
                ) {
                    DsToast(
                        toast.kind, toast.title, detail = toast.detail, actionLabel = toast.actionLabel,
                        onAction = toast.onAction?.let { action -> { action(); toasts.dismiss(toast.id) } },
                        onDismiss = { toasts.dismiss(toast.id) },
                    )
                }
            }
        }
    }
}

/** The tooltip's bubble: a label and its shortcut (web TooltipBubble). */
@Composable
fun TooltipBubble(label: String, modifier: Modifier = Modifier, shortcut: String? = null) {
    Row(
        modifier.clip(DsShapes.xs).background(MaterialTheme.colorScheme.inverseSurface).padding(horizontal = 8.dp, vertical = 4.dp),
        horizontalArrangement = Arrangement.spacedBy(8.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(label, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.inverseOnSurface)
        if (shortcut != null) Text(shortcut, style = Ds.label, color = MaterialTheme.colorScheme.inverseOnSurface.copy(alpha = TOAST_DETAIL_ALPHA))
    }
}

/** Says what a control does on hover or long-press (the M3 plain tooltip). */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun DsTooltip(label: String, modifier: Modifier = Modifier, shortcut: String? = null, content: @Composable () -> Unit) {
    TooltipBox(
        positionProvider = TooltipDefaults.rememberTooltipPositionProvider(androidx.compose.material3.TooltipAnchorPosition.Above),
        tooltip = { PlainTooltip { Text(if (shortcut != null) "$label  $shortcut" else label) } },
        state = rememberTooltipState(),
        modifier = modifier,
    ) { content() }
}
