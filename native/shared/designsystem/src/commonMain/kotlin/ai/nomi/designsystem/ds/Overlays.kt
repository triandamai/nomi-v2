package ai.nomi.designsystem.ds

import ai.nomi.designsystem.ds.mobile.PhoneToast
import ai.nomi.designsystem.ds.mobile.PhoneToastKind
import ai.nomi.designsystem.ds.mobile.SheetHeader
import ai.nomi.designsystem.generated.NomiPenShadow
import ai.nomi.designsystem.generated.PenShadow
import ai.nomi.designsystem.icon.Lucide
import ai.nomi.designsystem.icon.NomiGlyph
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.focusable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsFocusedAsState
import androidx.compose.foundation.interaction.collectIsHoveredAsState
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.selection.selectable
import androidx.compose.foundation.text.BasicText
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.ExperimentalMaterial3Api
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
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.draw.dropShadow
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.graphics.shadow.Shadow
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.onPreviewKeyEvent
import androidx.compose.ui.input.key.type
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.semantics.LiveRegionMode
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.liveRegion
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.DpOffset
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.IntRect
import androidx.compose.ui.unit.IntSize
import androidx.compose.ui.unit.LayoutDirection
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import androidx.compose.ui.window.Popup
import androidx.compose.ui.window.PopupPositionProvider
import androidx.compose.ui.window.PopupProperties
import kotlinx.coroutines.delay

// Design System / Nomi → Overlays (frontend-new `ds/overlays`), in dp.
private val DialogWidth = 420.dp
private val DialogWideWidth = 440.dp
private val DialogPad = 20.dp
private val DialogGap = 8.dp
private val DialogFootTop = 11.dp
private val DialogFootBottom = 12.dp
private val MenuWidth = 236.dp
private val MenuWideWidth = 280.dp
private val MenuChatWidth = 260.dp
private val MenuRowWidth = 220.dp
private val MenuPad = 4.dp
private val MenuGap = 2.dp
private val MenuRowHeight = 30.dp
private val MenuWideRowHeight = 32.dp
private val MenuRowPad = 8.dp
private val MenuRowGap = 8.dp
private val MenuWideRowGap = 10.dp
private val MenuGlyph = 14.dp
private val MenuGroupTop = 6.dp
private val MenuGroupBottom = 2.dp
private val MenuFootPad = 6.dp
private val WindowMargin = 8.dp
private val ToastWidth = 400.dp
private val ToastPadX = 14.dp
private val ToastPadY = 12.dp
private val ToastGap = 12.dp
private val ToastLines = 2.dp
private val ToastGlyph = 16.dp
private val ToastClose = 14.dp
private val ToastStack = 8.dp
private val ToastInset = 16.dp
private val TipPadX = 8.dp
private val TipPadY = 4.dp
private val TipGap = 8.dp
private val SheetLiftedBottom = 12.dp
private val SheetBodyBottom = 8.dp
private val ActionSheetInset = 10.dp
private val ActionSheetGap = 8.dp
private val ActionSheetHeadTop = 14.dp
private val ActionSheetHeadBottom = 13.dp
private val ActionSheetHeadPadX = 16.dp
private val ActionSheetHeadGap = 4.dp
private val ActionHeight = 56.dp
private val OptionHeight = 52.dp
private val OptionDetailHeight = 60.dp
private val OptionPadX = 20.dp
private val OptionPadY = 8.dp
private val OptionGap = 14.dp
private val OptionGlyph = 20.dp
private val OptionCheck = 18.dp
private val OptionRing = 2.dp
private val TilesPadX = 16.dp
private val TilesBottom = 10.dp
private val TilesGap = 8.dp
private val TileHeight = 68.dp
private val TileGap = 6.dp
private val TileGlyph = 20.dp
private const val TOAST_MS = 6_000L

/** Finds the overlay boxes in tests. */
const val DS_DIALOG_TAG = "ds-dialog"
const val DS_MENU_TAG = "ds-menu"
const val DS_TOAST_TAG = "ds-toast"

/** The pen's one shadow (`shadow-float`), or another of app.css's, cast in the pen's shadow colour. */
@Composable
fun Modifier.dsShadow(shape: Shape, shadow: PenShadow = NomiPenShadow.float): Modifier {
    val color = if (shadow.color == FOCUS_SHADOW) Ds.focus else Ds.shadow
    return dropShadow(shape, Shadow(radius = shadow.blur.dp, color = color, offset = DpOffset(shadow.x.dp, shadow.y.dp)))
}

private const val FOCUS_SHADOW = "focus"

/** Moves focus into an overlay as it opens, so the keyboard works inside it (Tab reaches its controls next). */
@Composable
private fun Modifier.takesFocus(): Modifier {
    val requester = remember { FocusRequester() }
    LaunchedEffect(Unit) { requester.requestFocus() }
    return focusRequester(requester).focusable()
}

/** Escape closes the overlay this wraps, wherever focus is inside it. */
private fun Modifier.closesOnEscape(close: () -> Unit): Modifier = onPreviewKeyEvent { event ->
    if (event.type == KeyEventType.KeyDown && event.key == Key.Escape) {
        close()
        true
    } else {
        false
    }
}

/** A floating box: surface, a border-strong edge drawn inside, the float shadow. */
@Composable
private fun Modifier.floating(shape: Shape): Modifier = dsShadow(shape).clip(shape).background(Ds.surface).border(Hairline, Ds.borderStrong, shape)

/** A hairline across the top of a box (`border-t border-border`). */
private fun Modifier.topRule(color: Color): Modifier = drawBehind {
    drawLine(color, Offset(0f, Hairline.toPx() / 2), Offset(size.width, Hairline.toPx() / 2), Hairline.toPx())
}

/** A hairline across the bottom of a box (`border-b border-border`). */
private fun Modifier.bottomRule(color: Color): Modifier = drawBehind {
    drawLine(color, Offset(0f, size.height - Hairline.toPx() / 2), Offset(size.width, size.height - Hairline.toPx() / 2), Hairline.toPx())
}

/**
 * Overlays / Dialog, the box: 420 wide ([wide]: 440), its 15 medium title and description over the content, actions
 * in a footer under a hairline.
 */
@Composable
fun DialogPanel(
    title: String,
    modifier: Modifier = Modifier,
    description: String? = null,
    wide: Boolean = false,
    actions: (@Composable () -> Unit)? = null,
    content: (@Composable ColumnScope.() -> Unit)? = null,
) {
    Column(modifier.width(if (wide) DialogWideWidth else DialogWidth).testTag(DS_DIALOG_TAG).floating(DsShapes.md)) {
        Column(Modifier.padding(DialogPad), verticalArrangement = Arrangement.spacedBy(DialogGap)) {
            BasicText(title, style = Ds.body.copy(color = Ds.text, fontWeight = FontWeight.Medium), modifier = Modifier.semantics { heading() })
            if (description != null) BasicText(description, style = Ds.ui.copy(color = Ds.text2, lineHeight = Ds.ui.fontSize * LEADING_NORMAL))
            content?.invoke(this)
        }
        if (actions != null) {
            FlowRow(
                Modifier.fillMaxWidth().topRule(Ds.border).padding(start = DialogPad, end = DialogPad, top = DialogFootTop, bottom = DialogFootBottom),
                horizontalArrangement = Arrangement.spacedBy(DialogGap, Alignment.End),
                verticalArrangement = Arrangement.spacedBy(DialogGap),
                itemVerticalAlignment = Alignment.CenterVertically,
            ) { actions() }
        }
    }
}

private const val LEADING_NORMAL = 1.5f

/**
 * A modal dialog on the overlay scrim, with focus kept inside until it closes (Escape, the scrim, or its own buttons
 * through [onDismiss]). [inline] draws only the panel in place, as the design sheets show it.
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
    DsModal(open, onDismiss) { DialogPanel(title, modifier.widthIn(max = DialogWideWidth), description, wide, actions, content) }
}

/** A modal for a panel that brings its own box (Share, Export): the overlay scrim, focus kept inside, Escape closes. */
@Composable
fun DsModal(open: Boolean, onDismiss: () -> Unit, content: @Composable () -> Unit) {
    if (!open) return
    Dialog(onDismissRequest = onDismiss, properties = DialogProperties(usePlatformDefaultWidth = false)) {
        Box(Modifier.takesFocus().closesOnEscape(onDismiss).padding(ToastInset), contentAlignment = Alignment.Center) { content() }
    }
}

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

/**
 * Overlays / Menu Item: 30 tall (wide: 32, 10 between its parts), icon, label, shortcut or meta; danger in red; on
 * surface-2 while hovered or focused.
 */
@Composable
fun DsMenuRow(item: MenuEntry.Item, modifier: Modifier = Modifier, wide: Boolean = false, onPick: (MenuEntry.Item) -> Unit = { it.onSelect() }) {
    val interaction = remember { MutableInteractionSource() }
    val hovered by interaction.collectIsHoveredAsState()
    val focused by interaction.collectIsFocusedAsState()
    val ink = if (item.danger) Ds.danger else Ds.text
    Row(
        modifier
            .then(if (wide) Modifier.fillMaxWidth() else Modifier.width(MenuRowWidth))
            .height(targetHeight(if (wide) MenuWideRowHeight else MenuRowHeight))
            .clip(DsShapes.xs)
            .background(if (hovered || focused) Ds.surface2 else Color.Transparent)
            .clickable(interaction, indication = null, role = Role.Button) { onPick(item) }
            .padding(horizontal = MenuRowPad),
        horizontalArrangement = Arrangement.spacedBy(if (wide) MenuWideRowGap else MenuRowGap),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        if (item.icon != null) DsIcon(item.icon, MenuGlyph, if (item.danger) Ds.danger else Ds.text2)
        BasicText(item.label, style = Ds.ui.copy(color = ink), modifier = Modifier.weight(1f), maxLines = 1, overflow = TextOverflow.Ellipsis)
        if (item.meta != null) BasicText(item.meta, style = Ds.label.copy(color = Ds.text3))
        if (item.shortcut != null) BasicText(item.shortcut, style = Ds.label.copy(color = Ds.text3))
    }
}

/**
 * Overlays / Menu, the panel: surface, a border-strong edge, the one shadow; 236 wide ([wide]: 280, the chat's menu
 * [chatWidth]: 260). Arrow keys, Home and End move between its rows, wrapping round.
 */
@Composable
fun MenuPanel(
    items: List<MenuEntry>,
    modifier: Modifier = Modifier,
    title: String? = null,
    footer: String? = null,
    wide: Boolean = false,
    chatWidth: Boolean = false,
    focusFirst: Boolean = false,
    onPick: (MenuEntry.Item) -> Unit = { it.onSelect() },
) {
    val rows = items.filterIsInstance<MenuEntry.Item>()
    val requesters = remember(rows.size) { List(rows.size) { FocusRequester() } }
    var at by remember { mutableStateOf(0) }
    fun move(to: Int) {
        if (rows.isEmpty()) return
        at = (to + rows.size) % rows.size
        requesters[at].requestFocus()
    }
    LaunchedEffect(focusFirst) { if (focusFirst && rows.isNotEmpty()) requesters[0].requestFocus() }
    val width = when {
        chatWidth -> MenuChatWidth
        wide -> MenuWideWidth
        else -> MenuWidth
    }
    Column(
        modifier.width(width).testTag(DS_MENU_TAG).floating(DsShapes.md).padding(MenuPad),
        verticalArrangement = Arrangement.spacedBy(MenuGap),
    ) {
        if (title != null) BasicText(title.uppercase(), style = Ds.labelCaps.copy(color = Ds.text3))
        var index = 0
        items.forEach { entry ->
            when (entry) {
                is MenuEntry.Item -> {
                    val mine = index++
                    DsMenuRow(
                        entry,
                        Modifier
                            .focusRequester(requesters[mine])
                            .onPreviewKeyEvent { event ->
                                if (event.type != KeyEventType.KeyDown) return@onPreviewKeyEvent false
                                when (event.key) {
                                    Key.DirectionDown -> { move(mine + 1); true }
                                    Key.DirectionUp -> { move(mine - 1); true }
                                    Key.MoveHome -> { move(0); true }
                                    Key.MoveEnd -> { move(rows.size - 1); true }
                                    else -> false
                                }
                            },
                        wide = wide || chatWidth,
                        onPick = onPick,
                    )
                }
                MenuEntry.Divider -> DsRule()
                is MenuEntry.Group -> BasicText(
                    entry.label.uppercase(),
                    style = Ds.labelCaps.copy(color = Ds.text3),
                    modifier = Modifier.padding(start = MenuRowPad, end = MenuRowPad, top = MenuGroupTop, bottom = MenuGroupBottom),
                )
            }
        }
        if (footer != null) {
            DsRule()
            BasicText(footer, style = Ds.label.copy(color = Ds.text3), modifier = Modifier.padding(horizontal = MenuRowPad, vertical = MenuFootPad))
        }
    }
}

/** Opens below its anchor, its start edge on the anchor's ([alignEnd]: end on end), above when there's no room, 8 inside the window. */
private class BelowAnchor(private val alignEnd: Boolean, private val margin: Int) : PopupPositionProvider {
    override fun calculatePosition(anchorBounds: IntRect, windowSize: IntSize, layoutDirection: LayoutDirection, popupContentSize: IntSize): IntOffset {
        val x = if (alignEnd) anchorBounds.right - popupContentSize.width else anchorBounds.left
        val below = anchorBounds.bottom + margin
        val y = if (below + popupContentSize.height > windowSize.height - margin) anchorBounds.top - margin - popupContentSize.height else below
        return IntOffset(
            x.coerceIn(margin, (windowSize.width - popupContentSize.width - margin).coerceAtLeast(margin)),
            y.coerceAtLeast(margin),
        )
    }
}

/**
 * Overlays / Menu: floats the panel off the button [trigger] draws (it's given `open` and whether it's open). The first
 * row takes focus; Escape or a click elsewhere closes it, choosing nothing. [inline] shows the panel in place.
 */
@Composable
fun DsMenu(
    items: List<MenuEntry>,
    modifier: Modifier = Modifier,
    title: String? = null,
    footer: String? = null,
    wide: Boolean = false,
    inline: Boolean = false,
    alignEnd: Boolean = false,
    trigger: @Composable (open: () -> Unit, expanded: Boolean) -> Unit = { _, _ -> },
) {
    if (inline) {
        MenuPanel(items, modifier, title, footer, wide)
        return
    }
    var expanded by remember { mutableStateOf(false) }
    val margin = with(androidx.compose.ui.platform.LocalDensity.current) { WindowMargin.roundToPx() }
    Box(modifier) {
        trigger({ expanded = !expanded }, expanded)
        if (expanded) {
            Popup(
                popupPositionProvider = remember(alignEnd, margin) { BelowAnchor(alignEnd, margin) },
                onDismissRequest = { expanded = false },
                properties = PopupProperties(focusable = true),
            ) {
                MenuPanel(items, Modifier.closesOnEscape { expanded = false }, title = title, footer = footer, wide = wide, focusFirst = true) { item ->
                    expanded = false
                    item.onSelect()
                }
            }
        }
    }
}

enum class ToastKind { Success, Error, Info }

/**
 * Overlays / Toast: what just happened (13 medium), its detail in mono, one action and a close, on a floating surface
 * box. Errors interrupt; everything else waits its turn.
 */
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
    dismissLabel: String = "Dismiss",
) {
    val glyph = icon ?: when (kind) {
        ToastKind.Success -> Lucide.Check
        ToastKind.Error -> Lucide.Alert
        ToastKind.Info -> Lucide.Info
    }
    val mark = when (kind) {
        ToastKind.Success -> Ds.accent
        ToastKind.Error -> Ds.danger
        ToastKind.Info -> Ds.text2
    }
    Row(
        modifier
            .fillMaxWidth()
            .testTag(DS_TOAST_TAG)
            .floating(DsShapes.md)
            .semantics { liveRegion = if (kind == ToastKind.Error) LiveRegionMode.Assertive else LiveRegionMode.Polite }
            .padding(horizontal = ToastPadX, vertical = ToastPadY),
        horizontalArrangement = Arrangement.spacedBy(ToastGap),
        verticalAlignment = Alignment.Top,
    ) {
        DsIcon(glyph, ToastGlyph, mark, Modifier.padding(top = Hairline))
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(ToastLines)) {
            BasicText(title, style = Ds.title.copy(color = Ds.text))
            if (detail != null) BasicText(detail, style = Ds.label.copy(color = Ds.text3))
        }
        if (actionLabel != null && onAction != null) {
            BasicText(actionLabel, style = Ds.ui.copy(color = Ds.text2), modifier = Modifier.clip(DsShapes.xs).clickable(role = Role.Button, onClick = onAction))
        }
        if (onDismiss != null) {
            Box(
                Modifier.clip(DsShapes.xs).clickable(role = Role.Button, onClick = onDismiss).semantics { contentDescription = dismissLabel },
            ) { DsIcon(Lucide.X, ToastClose, Ds.text3) }
        }
    }
}

/** One queued toast; [durationMs] null means it stays until dismissed. */
data class DsToastItem(
    val id: Long,
    val kind: ToastKind,
    val title: String,
    val detail: String? = null,
    val actionLabel: String? = null,
    val onAction: (() -> Unit)? = null,
    val icon: NomiGlyph? = null,
    val durationMs: Long? = null,
)

/**
 * The toasts on screen, newest last (web `toasts`). They go on their own after six seconds ([durationMs], say ten for
 * an Undo), except errors, which stay until dismissed unless given a time.
 */
@Stable
class DsToasts {
    val items = mutableStateListOf<DsToastItem>()
    private var next = 0L

    fun show(
        kind: ToastKind,
        title: String,
        detail: String? = null,
        actionLabel: String? = null,
        onAction: (() -> Unit)? = null,
        icon: NomiGlyph? = null,
        durationMs: Long? = null,
    ): Long {
        val id = next++
        items += DsToastItem(id, kind, title, detail, actionLabel, onAction, icon, durationMs ?: if (kind == ToastKind.Error) null else TOAST_MS)
        return id
    }

    fun dismiss(id: Long) {
        items.removeAll { it.id == id }
    }
}

/** Lets a toast go after its time. */
@Composable
private fun Expiry(toast: DsToastItem, toasts: DsToasts) {
    val duration = toast.durationMs ?: return
    LaunchedEffect(toast.id) {
        delay(duration)
        toasts.dismiss(toast.id)
    }
}

/** Where toasts appear on a desktop: 400 wide, stacked at the bottom right, 16 from the edges. */
@Composable
fun DsToastRegion(toasts: DsToasts, modifier: Modifier = Modifier, label: String = "Notifications") {
    Column(
        modifier.semantics { contentDescription = label }.padding(ToastInset),
        verticalArrangement = Arrangement.spacedBy(ToastStack),
        horizontalAlignment = Alignment.End,
    ) {
        toasts.items.forEach { toast ->
            androidx.compose.runtime.key(toast.id) {
                Expiry(toast, toasts)
                DsToast(
                    toast.kind, toast.title, Modifier.widthIn(max = ToastWidth), detail = toast.detail, icon = toast.icon,
                    actionLabel = toast.actionLabel,
                    onAction = toast.onAction?.let { action -> { action(); toasts.dismiss(toast.id) } },
                    onDismiss = { toasts.dismiss(toast.id) },
                )
            }
        }
    }
}

/** Where toasts appear on a phone: across the top, 16 from each side; a tap on one lets it go. */
@Composable
fun DsPhoneToastRegion(toasts: DsToasts, modifier: Modifier = Modifier, label: String = "Notifications") {
    Column(
        modifier.fillMaxWidth().semantics { contentDescription = label }.padding(horizontal = ToastInset).padding(top = DialogGap),
        verticalArrangement = Arrangement.spacedBy(ToastStack),
    ) {
        toasts.items.forEach { toast ->
            androidx.compose.runtime.key(toast.id) {
                Expiry(toast, toasts)
                val kind = when (toast.kind) {
                    ToastKind.Success -> PhoneToastKind.Success
                    ToastKind.Error -> PhoneToastKind.Error
                    ToastKind.Info -> PhoneToastKind.Info
                }
                Box(Modifier.clickable(interactionSource = null, indication = null) { toasts.dismiss(toast.id) }) {
                    PhoneToast(
                        kind, toast.title, detail = toast.detail, actionLabel = toast.actionLabel,
                        onAction = toast.onAction?.let { action -> { action(); toasts.dismiss(toast.id) } },
                    )
                }
            }
        }
    }
}

/** Overlays / Tooltip, the tip itself: inverted (12 bg on text), with the shortcut in mono. */
@Composable
fun TooltipBubble(label: String, modifier: Modifier = Modifier, shortcut: String? = null) {
    Row(
        modifier.clip(DsShapes.sm).background(Ds.text).padding(horizontal = TipPadX, vertical = TipPadY),
        horizontalArrangement = Arrangement.spacedBy(TipGap),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        BasicText(label, style = Ds.caption.copy(color = Ds.bg))
        if (shortcut != null) BasicText(shortcut, style = Ds.label.copy(color = Ds.text3))
    }
}

/** Says what a control does, with its shortcut, on hover or keyboard focus (a long press on a phone). */
@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun DsTooltip(label: String, modifier: Modifier = Modifier, shortcut: String? = null, content: @Composable () -> Unit) {
    TooltipBox(
        positionProvider = TooltipDefaults.rememberTooltipPositionProvider(androidx.compose.material3.TooltipAnchorPosition.Above),
        tooltip = { TooltipBubble(label, shortcut = shortcut) },
        state = rememberTooltipState(),
        modifier = modifier,
    ) { content() }
}

/** A phone sheet's frame: the overlay scrim (a tap on it closes), the sheet at the bottom. */
@Composable
private fun SheetFrame(open: Boolean, onDismiss: () -> Unit, label: String, content: @Composable ColumnScope.() -> Unit) {
    if (!open) return
    Dialog(onDismissRequest = onDismiss, properties = DialogProperties(usePlatformDefaultWidth = false)) {
        Box(
            Modifier.fillMaxSize().takesFocus().closesOnEscape(onDismiss).background(Ds.overlay).clickable(interactionSource = null, indication = null, onClick = onDismiss),
            contentAlignment = Alignment.BottomCenter,
        ) {
            Column(
                Modifier.fillMaxWidth().clickable(interactionSource = null, indication = null) {}.semantics { contentDescription = label },
                content = content,
            )
        }
    }
}

/**
 * A phone's bottom sheet: 22 top corners on surface over the scrim, its header (title, subtitle, close) or [header],
 * then its rows. [lifted] floats above the sheet's top right (a camera button).
 */
@Composable
fun DsBottomSheet(
    open: Boolean,
    onDismiss: () -> Unit,
    title: String,
    modifier: Modifier = Modifier,
    subtitle: String? = null,
    header: (@Composable (close: () -> Unit) -> Unit)? = null,
    lifted: (@Composable () -> Unit)? = null,
    content: @Composable ColumnScope.() -> Unit,
) {
    SheetFrame(open, onDismiss, title) {
        if (lifted != null) {
            Row(Modifier.fillMaxWidth().padding(horizontal = ToastInset).padding(bottom = SheetLiftedBottom), horizontalArrangement = Arrangement.End) { lifted() }
        }
        Column(
            modifier
                .fillMaxWidth()
                .clip(DsShapes.sheet)
                .background(Ds.surface)
                .topRule(Ds.borderStrong)
                .verticalScroll(rememberScrollState()),
        ) {
            if (header != null) header(onDismiss) else SheetHeader(title, onDismiss, subtitle = subtitle)
            Column(Modifier.padding(bottom = SheetBodyBottom), content = content)
        }
    }
}

/** One action in an action sheet; [danger] in red. */
data class SheetAction(val label: String, val danger: Boolean = false, val onClick: () -> Unit)

/**
 * A phone's action sheet: a title (and [body]) over 56 tall actions in a 16 rounded group, with [cancel] on its own
 * below. Picking an action closes the sheet first.
 */
@Composable
fun DsActionSheet(open: Boolean, onDismiss: () -> Unit, title: String, actions: List<SheetAction>, cancel: String, modifier: Modifier = Modifier, body: String? = null) {
    SheetFrame(open, onDismiss, title) {
        Column(modifier.fillMaxWidth().padding(horizontal = ActionSheetInset), verticalArrangement = Arrangement.spacedBy(ActionSheetGap)) {
            Column(Modifier.fillMaxWidth().clip(DsShapes.xl2).background(Ds.surface)) {
                Column(
                    Modifier
                        .fillMaxWidth()
                        .bottomRule(Ds.border)
                        .padding(start = ActionSheetHeadPadX, end = ActionSheetHeadPadX, top = ActionSheetHeadTop, bottom = ActionSheetHeadBottom),
                    verticalArrangement = Arrangement.spacedBy(ActionSheetHeadGap),
                    horizontalAlignment = Alignment.CenterHorizontally,
                ) {
                    BasicText(title, style = Ds.ui.copy(color = Ds.text2, fontWeight = FontWeight.SemiBold, textAlign = TextAlign.Center))
                    if (body != null) BasicText(body, style = Ds.ui.copy(color = Ds.text3, textAlign = TextAlign.Center))
                }
                actions.forEachIndexed { index, action ->
                    Box(
                        Modifier
                            .fillMaxWidth()
                            .height(ActionHeight)
                            .then(if (index < actions.size - 1) Modifier.bottomRule(Ds.border) else Modifier)
                            .clickable(role = Role.Button) { onDismiss(); action.onClick() },
                        contentAlignment = Alignment.Center,
                    ) { BasicText(action.label, style = Ds.phoneAction.copy(color = if (action.danger) Ds.danger else Ds.text)) }
                }
            }
            Box(
                Modifier
                    .fillMaxWidth()
                    .padding(bottom = ActionSheetGap)
                    .height(ActionHeight)
                    .clip(DsShapes.xl2)
                    .background(Ds.surface)
                    .clickable(role = Role.Button, onClick = onDismiss),
                contentAlignment = Alignment.Center,
            ) { BasicText(cancel, style = Ds.phoneAction.copy(color = Ds.text, fontWeight = FontWeight.SemiBold)) }
        }
    }
}

/**
 * One choice in a phone sheet: 52 tall (60 with a [detail]), its icon or [lead], the label at 16 and a mint check when
 * [chosen].
 */
@Composable
fun DsSheetOption(
    label: String,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    detail: String? = null,
    icon: NomiGlyph? = null,
    chosen: Boolean = false,
    lead: (@Composable () -> Unit)? = null,
) {
    var focused by remember { mutableStateOf(false) }
    Row(
        modifier
            .fillMaxWidth()
            .dsFocusRing(focused, DsShapes.xs, Ds.focus, OptionRing, -OptionRing)
            .onFocusChanged { focused = it.isFocused }
            .then(if (detail != null) Modifier.heightIn(min = OptionDetailHeight) else Modifier.height(OptionHeight))
            .selectable(chosen, role = Role.Button, onClick = onClick)
            .padding(horizontal = OptionPadX, vertical = if (detail != null) OptionPadY else 0.dp),
        horizontalArrangement = Arrangement.spacedBy(OptionGap),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        when {
            lead != null -> lead()
            icon != null -> DsIcon(icon, OptionGlyph, Ds.text2)
        }
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(ToastLines)) {
            BasicText(label, style = Ds.phone.copy(color = Ds.text), maxLines = 1, overflow = TextOverflow.Ellipsis)
            if (detail != null) BasicText(detail, style = Ds.ui.copy(color = Ds.text3), maxLines = 1, overflow = TextOverflow.Ellipsis)
        }
        if (chosen) DsIcon(Lucide.Check, OptionCheck, Ds.accent)
    }
}

/** A tile in a sheet's row of quick actions; [pressed] (a toggle that's on) draws it in mint. */
data class SheetTile(val icon: NomiGlyph, val label: String, val pressed: Boolean? = null, val onClick: () -> Unit)

/** A phone sheet's row of quick actions: 68 tall tiles on surface-2, the icon over the label. */
@Composable
fun DsSheetTiles(tiles: List<SheetTile>, modifier: Modifier = Modifier) {
    Row(modifier.fillMaxWidth().padding(start = TilesPadX, end = TilesPadX, bottom = TilesBottom), horizontalArrangement = Arrangement.spacedBy(TilesGap)) {
        tiles.forEach { tile ->
            val on = tile.pressed == true
            Column(
                Modifier
                    .weight(1f)
                    .height(TileHeight)
                    .clip(DsShapes.xl)
                    .background(Ds.surface2)
                    .clickable(role = Role.Button, onClick = tile.onClick),
                verticalArrangement = Arrangement.spacedBy(TileGap, Alignment.CenterVertically),
                horizontalAlignment = Alignment.CenterHorizontally,
            ) {
                DsIcon(tile.icon, TileGlyph, if (on) Ds.accent else Ds.text)
                BasicText(tile.label, style = Ds.ui.copy(color = if (on) Ds.accent else Ds.text2))
            }
        }
    }
}
