package ai.nomi.designsystem.ds

import ai.nomi.designsystem.icon.Lucide
import ai.nomi.designsystem.icon.NomiGlyph
import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.animateDpAsState
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
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.selection.selectable
import androidx.compose.foundation.selection.selectableGroup
import androidx.compose.foundation.selection.toggleable
import androidx.compose.foundation.text.BasicText
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.isShiftPressed
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.onPreviewKeyEvent
import androidx.compose.ui.input.key.type
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.error
import androidx.compose.ui.semantics.role
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp

// Design System / Nomi → Inputs (frontend-new `ds/inputs`). Sizes are the web's, in dp.
private val CheckSize = 16.dp
private val CheckGlyph = 12.dp
private val CheckboxGap = 8.dp
private val ToggleWidth = 32.dp
private val ToggleHeight = 18.dp
private val ToggleKnob = 14.dp
private val TogglePad = 1.dp
private val ChipHeight = 30.dp
private val ChipPadding = 8.dp
private val ChipAgentStart = 6.dp
private val ChipGap = 6.dp
private val ChipMascot = 20.dp
private val ChipIcon = 14.dp
private val ChipChevron = 12.dp
private val ProvisionHeight = 24.dp
private val ProvisionIcon = 12.dp
private val SearchHeight = 30.dp
private val SearchPadding = 12.dp
private val SearchPlainPadding = 10.dp
private val SearchGlyph = 14.dp
private val PhoneSearchHeight = 38.dp
private val PhoneSearchPadding = 12.dp
private val PhoneSearchGlyph = 16.dp
private val FieldGap = 8.dp
private val SegmentGroupPad = 2.dp
private val SegmentHeight = 26.dp
private val SegmentPadding = 12.dp
private val SegmentGap = 6.dp
private val TabHeight = 40.dp
private val TabGap = 20.dp
private val TextFieldHeight = 34.dp
private val TextFieldPadding = 7.dp
private val TextFieldStackGap = 6.dp
private val TextFieldRing = 3.dp
private val ErrorGlyph = 14.dp
private val SelectHeight = 30.dp
private val SelectTallHeight = 34.dp
private val SelectStart = 9.dp
private val SelectLeadStart = 34.dp
private val SelectLeadInset = 7.dp
private val SelectEnd = 10.dp
private val SelectChevron = 12.dp
private val ComposerPadStart = 11.dp
private val ComposerPadTop = 13.dp
private val ComposerPadBottom = 9.dp
private val ComposerGap = 12.dp
private val ComposerToolsGap = 6.dp
private val ComposerFieldInset = 4.dp
private val ComposerMaxField = 240.dp
private val RadioSize = 16.dp
private val RadioRingChosen = 5.dp
private const val DISABLED_OPACITY = 0.4f
private const val FIELD_DISABLED_OPACITY = 0.5f

/** Finds a text field's box in tests: its label belongs to the field itself. */
const val DS_FIELD_TAG = "ds-text-field"

/** The 16 box of Design System / Nomi → Check, for rows that bring their own label: text colour when checked. */
@Composable
fun CheckMark(checked: Boolean, onCheckedChange: ((Boolean) -> Unit)?, modifier: Modifier = Modifier, enabled: Boolean = true) {
    val toggle = if (onCheckedChange != null) {
        Modifier.toggleable(checked, null, indication = null, enabled = enabled, role = Role.Checkbox, onValueChange = onCheckedChange)
    } else {
        Modifier
    }
    Box(
        modifier
            .then(toggle)
            .size(CheckSize)
            .clip(DsShapes.xs)
            .background(if (checked) Ds.text else Ds.surface)
            .border(Hairline, if (checked) Ds.text else Ds.borderStrong, DsShapes.xs),
        contentAlignment = Alignment.Center,
    ) { if (checked) DsIcon(Lucide.Check, CheckGlyph, Ds.bg) }
}

/** A real checkbox with its 13 label: the whole row toggles it. */
@Composable
fun DsCheckbox(label: String, checked: Boolean, onCheckedChange: (Boolean) -> Unit, modifier: Modifier = Modifier, enabled: Boolean = true) {
    Row(
        modifier
            .alpha(if (enabled) 1f else DISABLED_OPACITY)
            .heightIn(min = targetHeight(CheckSize))
            .toggleable(checked, null, indication = null, enabled = enabled, role = Role.Checkbox, onValueChange = onCheckedChange),
        horizontalArrangement = Arrangement.spacedBy(CheckboxGap),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        CheckMark(checked, null)
        BasicText(label, style = Ds.ui.copy(color = Ds.text))
    }
}

/** A radio dot (16), its ring thickening to the accent when chosen. */
@Composable
fun RadioMark(selected: Boolean, modifier: Modifier = Modifier) {
    val ring by animateDpAsState(if (selected) RadioRingChosen else Hairline, DsMotion.spatial(), label = "radio-ring")
    val color by animateColorAsState(if (selected) Ds.accent else Ds.borderStrong, DsMotion.fast(), label = "radio-color")
    Box(modifier.size(RadioSize).clip(DsShapes.full).border(ring, color, DsShapes.full))
}

/** Design System / Nomi → Toggle: a 32 by 18 switch, mint with the knob right when on. Named by [label]. */
@Composable
fun DsToggle(label: String, checked: Boolean, onCheckedChange: (Boolean) -> Unit, modifier: Modifier = Modifier, enabled: Boolean = true) {
    Box(
        modifier
            .alpha(if (enabled) 1f else DISABLED_OPACITY)
            .size(ToggleWidth, ToggleHeight)
            .clip(DsShapes.full)
            .background(if (checked) Ds.accent else Ds.surface2)
            .border(Hairline, if (checked) Ds.accent else Ds.borderStrong, DsShapes.full)
            .toggleable(checked, null, indication = null, enabled = enabled, role = Role.Switch, onValueChange = onCheckedChange)
            .semantics { contentDescription = label }
            .padding(TogglePad + Hairline),
        contentAlignment = if (checked) Alignment.CenterEnd else Alignment.CenterStart,
    ) { Box(Modifier.size(ToggleKnob).clip(DsShapes.full).background(if (checked) Ds.accentInk else Ds.text3)) }
}

enum class ChipKind { Model, Option, Agent, Provision }

/**
 * Design System / Nomi → Chips. Model, option and agent pick something from a menu (a chevron says so); provision
 * names what a project was given (a git repo, a database) and isn't interactive.
 */
@Composable
fun DsChip(
    kind: ChipKind,
    label: String,
    modifier: Modifier = Modifier,
    icon: NomiGlyph? = null,
    agent: Crew? = null,
    expanded: Boolean = false,
    onClick: (() -> Unit)? = null,
) {
    if (kind == ChipKind.Provision) {
        Row(
            modifier.semantics(mergeDescendants = true) {}.height(ProvisionHeight).clip(DsShapes.sm).background(Ds.surface2).padding(horizontal = ChipPadding),
            horizontalArrangement = Arrangement.spacedBy(ChipGap),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            if (icon != null) DsIcon(icon, ProvisionIcon, Ds.text3)
            BasicText(label, style = Ds.label.copy(color = Ds.text2))
        }
        return
    }
    val interaction = remember { MutableInteractionSource() }
    val hovered by interaction.collectIsHoveredAsState()
    val focused by interaction.collectIsFocusedAsState()
    Row(
        modifier
            .height(targetHeight(ChipHeight))
            .dsFocusRing(focused, DsShapes.sm, Ds.focus, ChipFocusRing, Hairline)
            .clip(DsShapes.sm)
            .background(if (hovered) Ds.border else Ds.surface2)
            .clickable(interaction, indication = null, enabled = onClick != null, role = Role.DropdownList) { onClick?.invoke() }
            .padding(start = if (kind == ChipKind.Agent) ChipAgentStart else ChipPadding, end = ChipPadding),
        horizontalArrangement = Arrangement.spacedBy(ChipGap),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        when {
            kind == ChipKind.Agent && agent != null -> Mascot(agent, ChipMascot)
            kind == ChipKind.Option && icon != null -> DsIcon(icon, ChipIcon, Ds.text3)
        }
        val style = when (kind) {
            ChipKind.Model -> Ds.mono
            ChipKind.Option -> Ds.caption
            else -> Ds.ui
        }
        BasicText(label, style = style.copy(color = Ds.text2), maxLines = 1)
        DsIcon(Lucide.ChevronDown, ChipChevron, Ds.text3)
    }
}

private val ChipFocusRing = 2.dp

/** A single-line field with no chrome of its own, for the design system's shaped boxes. */
@Composable
internal fun DsBareField(
    value: String,
    onValueChange: (String) -> Unit,
    placeholder: String,
    style: TextStyle,
    modifier: Modifier = Modifier,
    singleLine: Boolean = true,
    secret: Boolean = false,
    keyboard: KeyboardType = KeyboardType.Text,
    interaction: MutableInteractionSource? = null,
    enabled: Boolean = true,
) {
    BasicTextField(
        value = value,
        onValueChange = onValueChange,
        modifier = modifier,
        enabled = enabled,
        singleLine = singleLine,
        textStyle = style.copy(color = if (enabled) Ds.text else Ds.text3),
        cursorBrush = SolidColor(Ds.accent),
        keyboardOptions = KeyboardOptions(keyboardType = keyboard),
        visualTransformation = if (secret) PasswordVisualTransformation() else VisualTransformation.None,
        interactionSource = interaction,
        decorationBox = { inner ->
            Box(contentAlignment = Alignment.CenterStart) {
                if (value.isEmpty()) BasicText(placeholder, style = style.copy(color = Ds.text3), maxLines = 1)
                inner()
            }
        },
    )
}

/**
 * Design System / Nomi → Search: a quiet 30 box on the plain border (not -strong) with its shortcut key, mint when
 * focused. [plain] is the settings nav's: 10 in, the shortcut as plain mono text.
 */
@Composable
fun DsSearch(
    value: String,
    onValueChange: (String) -> Unit,
    placeholder: String,
    modifier: Modifier = Modifier,
    shortcut: String? = null,
    plain: Boolean = false,
    label: String? = null,
) {
    val interaction = remember { MutableInteractionSource() }
    val focused by interaction.collectIsFocusedAsState()
    Row(
        modifier
            .fillMaxWidth()
            .height(SearchHeight)
            .dsFocusRing(focused, DsShapes.sm, Ds.focus, TextFieldRing)
            .clip(DsShapes.sm)
            .border(Hairline, if (focused) Ds.accent else Ds.border, DsShapes.sm)
            .semantics { contentDescription = label ?: placeholder }
            .padding(horizontal = if (plain) SearchPlainPadding else SearchPadding),
        horizontalArrangement = Arrangement.spacedBy(FieldGap),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        DsIcon(Lucide.Search, SearchGlyph, Ds.text3)
        DsBareField(value, onValueChange, placeholder, Ds.ui, Modifier.weight(1f), interaction = interaction)
        when {
            shortcut != null && plain -> BasicText(shortcut, style = Ds.label.copy(color = Ds.text3))
            shortcut != null -> Kbd(shortcut)
        }
    }
}

/** A phone list's search: 38 tall on surface-2 with no stroke and 12 corners, 16 text so nothing zooms, mint when focused. */
@Composable
fun DsPhoneSearch(value: String, onValueChange: (String) -> Unit, placeholder: String, modifier: Modifier = Modifier, label: String? = null) {
    val interaction = remember { MutableInteractionSource() }
    val focused by interaction.collectIsFocusedAsState()
    Row(
        modifier
            .fillMaxWidth()
            .height(PhoneSearchHeight)
            .dsFocusRing(focused, DsShapes.lg, Ds.focus, TextFieldRing)
            .clip(DsShapes.lg)
            .background(Ds.surface2)
            .border(Hairline, if (focused) Ds.accent else Color.Transparent, DsShapes.lg)
            .semantics { contentDescription = label ?: placeholder }
            .padding(horizontal = PhoneSearchPadding),
        horizontalArrangement = Arrangement.spacedBy(FieldGap),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        DsIcon(Lucide.Search, PhoneSearchGlyph, Ds.text3)
        DsBareField(value, onValueChange, placeholder, Ds.phone, Modifier.weight(1f), interaction = interaction)
    }
}

data class SegmentOption(val value: String, val label: String, val count: String? = null)

enum class SegmentedSurface { Filled, Bg, Surface }

/** Moves to the next (+1) or previous (-1) option on the arrow keys, wrapping round, and focuses it. */
private fun Modifier.arrowKeys(previous: Set<Key>, next: Set<Key>, move: (Int) -> Unit): Modifier = onPreviewKeyEvent { event ->
    if (event.type != KeyEventType.KeyDown) return@onPreviewKeyEvent false
    when (event.key) {
        in next -> { move(1); true }
        in previous -> { move(-1); true }
        else -> false
    }
}

/**
 * Design System / Nomi → Segmented: a few mutually exclusive views of one list ("All 4 · Live 2 · Building 1"). The
 * chosen one sits on bg with a border-strong edge; arrow keys move the choice. [surface]: filled (surface-2) on its own,
 * or outlined on bg or surface inside a card.
 */
@Composable
fun DsSegmented(
    label: String,
    options: List<SegmentOption>,
    value: String,
    onChange: (String) -> Unit,
    modifier: Modifier = Modifier,
    surface: SegmentedSurface = SegmentedSurface.Filled,
) {
    val requesters = remember(options.size) { List(options.size) { FocusRequester() } }
    fun step(by: Int) {
        val at = options.indexOfFirst { it.value == value }.coerceAtLeast(0)
        val next = (at + by + options.size) % options.size
        onChange(options[next].value)
        requesters[next].requestFocus()
    }
    val (fill, outline) = when (surface) {
        SegmentedSurface.Filled -> Ds.surface2 to null
        SegmentedSurface.Bg -> Ds.bg to Ds.border
        SegmentedSurface.Surface -> Ds.surface to Ds.border
    }
    FlowRow(
        modifier
            .clip(DsShapes.sm)
            .background(fill)
            .then(if (outline != null) Modifier.border(Hairline, outline, DsShapes.sm) else Modifier)
            .selectableGroup()
            .semantics { contentDescription = label }
            .padding(SegmentGroupPad),
        horizontalArrangement = Arrangement.spacedBy(SegmentGroupPad),
        verticalArrangement = Arrangement.spacedBy(SegmentGroupPad),
    ) {
        options.forEachIndexed { index, option ->
            val checked = option.value == value
            val interaction = remember { MutableInteractionSource() }
            val hovered by interaction.collectIsHoveredAsState()
            val focused by interaction.collectIsFocusedAsState()
            Row(
                Modifier
                    .height(targetHeight(SegmentHeight))
                    .dsFocusRing(focused, DsShapes.sm, Ds.focus, ChipFocusRing, Hairline)
                    .focusRequester(requesters[index])
                    .arrowKeys(setOf(Key.DirectionLeft, Key.DirectionUp), setOf(Key.DirectionRight, Key.DirectionDown), ::step)
                    .clip(DsShapes.sm)
                    .background(if (checked) Ds.bg else Color.Transparent)
                    .border(Hairline, if (checked) Ds.borderStrong else Color.Transparent, DsShapes.sm)
                    .selectable(checked, interaction, indication = null, role = Role.RadioButton) { onChange(option.value) }
                    .padding(horizontal = SegmentPadding),
                horizontalArrangement = Arrangement.spacedBy(SegmentGap),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                val ink = if (checked || hovered) Ds.text else Ds.text2
                BasicText(option.label, style = Ds.ui.copy(color = ink, fontWeight = if (checked) FontWeight.Medium else FontWeight.Normal))
                if (option.count != null) BasicText(option.count, style = Ds.label.copy(color = Ds.text3))
            }
        }
    }
}

data class TabItem(val value: String, val label: String)

/**
 * Design System / Nomi → Tabs: 40 tall, the open one in text colour with a 1 line under it, on a border rule. Arrow
 * keys, Home and End move between tabs and open them. [panel] draws the open tab's content.
 */
@Composable
fun DsTabs(
    label: String,
    tabs: List<TabItem>,
    value: String,
    onChange: (String) -> Unit,
    modifier: Modifier = Modifier,
    panel: (@Composable (String) -> Unit)? = null,
) {
    val requesters = remember(tabs.size) { List(tabs.size) { FocusRequester() } }
    fun open(index: Int) {
        val next = (index + tabs.size) % tabs.size
        onChange(tabs[next].value)
        requesters[next].requestFocus()
    }
    val at = tabs.indexOfFirst { it.value == value }.coerceAtLeast(0)
    val rule = Ds.border
    Column(modifier.fillMaxWidth()) {
        Row(
            Modifier
                .fillMaxWidth()
                .drawBehind { drawLine(rule, Offset(0f, size.height - Hairline.toPx() / 2), Offset(size.width, size.height - Hairline.toPx() / 2), Hairline.toPx()) }
                .selectableGroup()
                .semantics { contentDescription = label },
            horizontalArrangement = Arrangement.spacedBy(TabGap),
        ) {
            tabs.forEachIndexed { index, tab ->
                val selected = index == at
                val interaction = remember { MutableInteractionSource() }
                val hovered by interaction.collectIsHoveredAsState()
                val focused by interaction.collectIsFocusedAsState()
                val line = Ds.text
                Box(
                    Modifier
                        .height(TabHeight)
                        .dsFocusRing(focused, DsShapes.xs, Ds.focus, ChipFocusRing, Hairline)
                        .focusRequester(requesters[index])
                        .onPreviewKeyEvent { event ->
                            if (event.type != KeyEventType.KeyDown) return@onPreviewKeyEvent false
                            when (event.key) {
                                Key.DirectionRight -> { open(at + 1); true }
                                Key.DirectionLeft -> { open(at - 1); true }
                                Key.MoveHome -> { open(0); true }
                                Key.MoveEnd -> { open(tabs.size - 1); true }
                                else -> false
                            }
                        }
                        .selectable(selected, interaction, indication = null, role = Role.Tab) { onChange(tab.value) }
                        .drawBehind {
                            if (selected) drawLine(line, Offset(0f, size.height - Hairline.toPx() / 2), Offset(size.width, size.height - Hairline.toPx() / 2), Hairline.toPx())
                        },
                    contentAlignment = Alignment.Center,
                ) {
                    val ink = when {
                        selected -> Ds.text
                        hovered -> Ds.text2
                        else -> Ds.text3
                    }
                    BasicText(tab.label, style = Ds.ui.copy(color = ink, fontWeight = if (selected) FontWeight.Medium else FontWeight.Normal))
                }
            }
        }
        if (panel != null) panel(value)
    }
}

/**
 * Design System / Nomi → Inputs: label · field · helper. The 34 field sits on surface with border-strong; focus turns
 * the edge mint and adds the 3 focus ring; an error swaps the edge and takes the helper's place to say what to do next.
 * [smallHelper] sets the helper at 11 where the pen does (the first-run name).
 */
@Composable
fun DsTextField(
    label: String,
    value: String,
    onValueChange: (String) -> Unit,
    modifier: Modifier = Modifier,
    helper: String? = null,
    error: String? = null,
    placeholder: String = "",
    mono: Boolean = false,
    enabled: Boolean = true,
    secret: Boolean = false,
    keyboard: KeyboardType = KeyboardType.Text,
    showFocus: Boolean = false,
    smallHelper: Boolean = false,
) {
    val interaction = remember { MutableInteractionSource() }
    val focused by interaction.collectIsFocusedAsState()
    val active = (focused || showFocus) && enabled
    val edge = when {
        error != null -> Ds.danger
        !enabled -> Ds.border
        active -> Ds.accent
        else -> Ds.borderStrong
    }
    val note = error ?: helper
    Column(modifier.fillMaxWidth().alpha(if (enabled) 1f else FIELD_DISABLED_OPACITY), verticalArrangement = Arrangement.spacedBy(TextFieldStackGap)) {
        BasicText(label, style = Ds.title.copy(color = Ds.text))
        Row(
            Modifier
                .fillMaxWidth()
                .height(TextFieldHeight)
                .testTag(DS_FIELD_TAG)
                .dsFocusRing(active, DsShapes.sm, Ds.focus, TextFieldRing)
                .clip(DsShapes.sm)
                .background(Ds.surface)
                .border(Hairline, edge, DsShapes.sm)
                .padding(horizontal = TextFieldPadding),
            horizontalArrangement = Arrangement.spacedBy(FieldGap),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            DsBareField(
                value, onValueChange, placeholder,
                if (mono) Ds.ui.copy(fontFamily = Ds.mono.fontFamily) else Ds.ui,
                Modifier
                    .weight(1f)
                    .semantics {
                        contentDescription = label
                        if (error != null) error(error)
                    },
                secret = secret, keyboard = keyboard, interaction = interaction, enabled = enabled,
            )
            if (error != null) DsIcon(Lucide.Alert, ErrorGlyph, Ds.danger)
        }
        if (note != null) {
            AnimatedContent(note, label = "field-note") { text ->
                val style = if (smallHelper && error == null) Ds.label else Ds.caption
                BasicText(text, style = style.copy(color = if (error != null) Ds.danger else Ds.text3))
            }
        }
    }
}

data class SelectOption(val value: String, val label: String)

/**
 * App / Settings → Select: one choice from a list, as a 30 field (border-strong on bg, the value at 12 and a chevron)
 * that opens the list as a menu. [tall] is the 34 field on surface that Models & providers draws, with [lead] (a
 * mascot) before the value. It fills the width it's given.
 */
@Composable
fun DsSelect(
    label: String,
    options: List<SelectOption>,
    value: String,
    onChange: (String) -> Unit,
    modifier: Modifier = Modifier,
    placeholder: String? = null,
    mono: Boolean = false,
    tall: Boolean = false,
    enabled: Boolean = true,
    lead: (@Composable () -> Unit)? = null,
) {
    val chosen = options.firstOrNull { it.value == value }
    DsMenu(
        items = options.map { option -> MenuEntry.Item(option.label, icon = if (option.value == value) Lucide.Check else null) { onChange(option.value) } },
        modifier = modifier,
        trigger = { open, _ ->
            val interaction = remember { MutableInteractionSource() }
            val hovered by interaction.collectIsHoveredAsState()
            val focused by interaction.collectIsFocusedAsState()
            val rest = if (tall) Ds.surface else Ds.bg
            val hover = if (tall) Ds.surface2 else Ds.surface
            Row(
                Modifier
                    .fillMaxWidth()
                    .alpha(if (enabled) 1f else FIELD_DISABLED_OPACITY)
                    .height(targetHeight(if (tall) SelectTallHeight else SelectHeight))
                    .dsFocusRing(focused, DsShapes.sm, Ds.focus, ChipFocusRing, Hairline)
                    .clip(DsShapes.sm)
                    .background(if (hovered && enabled) hover else rest)
                    .border(Hairline, Ds.borderStrong, DsShapes.sm)
                    .clickable(interaction, indication = null, enabled = enabled, role = Role.DropdownList, onClick = open)
                    .semantics { contentDescription = label }
                    .padding(start = if (lead != null) SelectLeadInset else SelectStart, end = SelectEnd),
                horizontalArrangement = Arrangement.spacedBy(FieldGap),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                if (lead != null) lead()
                val base = if (tall) Ds.ui else Ds.caption
                BasicText(
                    chosen?.label ?: placeholder.orEmpty(),
                    style = (if (mono) base.copy(fontFamily = Ds.mono.fontFamily) else base).copy(color = if (chosen != null) Ds.text else Ds.text3),
                    modifier = Modifier.weight(1f),
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                DsIcon(Lucide.ChevronDown, SelectChevron, Ds.text3)
            }
        },
    )
}

/** Who a composer's message goes to. */
data class ComposerAgent(val crew: Crew, val name: String)

/**
 * Design System / Nomi → Composer: one component for chat, home and project; the agent chip and placeholder change.
 * Enter sends and Shift+Enter starts a new line; Send stays off until there's something to send. The field grows with
 * what's written, up to 240, then scrolls. The copy defaults are the web's English strings.
 */
@Composable
fun DsComposer(
    placeholder: String,
    agent: ComposerAgent,
    model: String,
    modifier: Modifier = Modifier,
    hints: List<KeyHintItem> = listOf(KeyHintItem("↵", "send"), KeyHintItem("⇧↵", "new line"), KeyHintItem("@", "address crew")),
    showHints: Boolean = !targetIsCoarse(),
    fieldLabel: String = "Message",
    attachLabel: String = "Attach a file",
    recordLabel: String = "Record a voice note",
    sendLabel: String = "Send",
    onSend: (String) -> Unit = {},
    onAttach: () -> Unit = {},
    onRecord: () -> Unit = {},
    onPickAgent: () -> Unit = {},
    onPickModel: () -> Unit = {},
) {
    var text by remember { mutableStateOf("") }
    val canSend = text.isNotBlank()
    fun send() {
        if (!canSend) return
        onSend(text.trim())
        text = ""
    }
    Column(
        modifier
            .fillMaxWidth()
            .clip(DsShapes.md)
            .background(Ds.surface)
            .border(Hairline, Ds.borderStrong, DsShapes.md)
            .padding(start = ComposerPadStart, end = ComposerPadStart, top = ComposerPadTop, bottom = ComposerPadBottom),
        verticalArrangement = Arrangement.spacedBy(ComposerGap),
    ) {
        BasicTextField(
            value = text,
            onValueChange = { text = it },
            modifier = Modifier
                .fillMaxWidth()
                .heightIn(max = ComposerMaxField)
                .padding(horizontal = ComposerFieldInset)
                .semantics { contentDescription = fieldLabel }
                .onPreviewKeyEvent { event ->
                    if (event.type == KeyEventType.KeyDown && event.key == Key.Enter && !event.isShiftPressed) {
                        send()
                        true
                    } else {
                        false
                    }
                },
            textStyle = Ds.body.copy(color = Ds.text),
            cursorBrush = SolidColor(Ds.accent),
            decorationBox = { inner ->
                Box {
                    if (text.isEmpty()) BasicText(placeholder, style = Ds.body.copy(color = Ds.text3))
                    inner()
                }
            },
        )
        Row(horizontalArrangement = Arrangement.spacedBy(ComposerToolsGap), verticalAlignment = Alignment.CenterVertically) {
            DsIconButton(Lucide.Plus, attachLabel, onAttach)
            DsChip(ChipKind.Agent, agent.name, agent = agent.crew, onClick = onPickAgent)
            DsChip(ChipKind.Model, model, onClick = onPickModel)
            Spacer(Modifier.weight(1f))
            if (showHints) KeyHint(hints)
            DsIconButton(Lucide.Mic, recordLabel, onRecord, variant = DsIconButtonVariant.Ghost)
            DsIconButton(Lucide.ArrowUp, sendLabel, ::send, variant = DsIconButtonVariant.Send, enabled = canSend)
        }
    }
}

/** The web hides the composer's key hints below md and on touch; native hides them on a coarse pointer. */
@Composable
private fun targetIsCoarse(): Boolean = ai.nomi.designsystem.theme.NomiTheme.coarsePointer
