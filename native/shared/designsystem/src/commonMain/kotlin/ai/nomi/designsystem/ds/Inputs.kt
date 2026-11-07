package ai.nomi.designsystem.ds

import ai.nomi.designsystem.component.NomiSwitch
import ai.nomi.designsystem.component.nomiFocusRing
import ai.nomi.designsystem.icon.Lucide
import ai.nomi.designsystem.icon.NomiGlyph
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.Crossfade
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.scaleIn
import androidx.compose.animation.scaleOut
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsFocusedAsState
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.selection.selectable
import androidx.compose.foundation.selection.selectableGroup
import androidx.compose.foundation.selection.toggleable
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.layout.positionInParent
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.error
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

private const val INDICATOR_RADIUS = 7

/** The M3 checkbox box alone (18dp, 7dp corners), for rows that bring their own label. Fill and check animate in. */
@Composable
fun CheckMark(checked: Boolean, onCheckedChange: ((Boolean) -> Unit)?, modifier: Modifier = Modifier, enabled: Boolean = true) {
    val fill by animateColorAsState(if (checked) Ds.accent else Color.Transparent, DsMotion.fast(), label = "check-fill")
    val edge by animateColorAsState(if (checked) Ds.accent else MaterialTheme.colorScheme.outline, DsMotion.fast(), label = "check-edge")
    val shape = RoundedCornerShape(INDICATOR_RADIUS.dp)
    val toggle = if (onCheckedChange != null) {
        Modifier.toggleable(checked, enabled = enabled, role = Role.Checkbox, onValueChange = onCheckedChange)
    } else {
        Modifier
    }
    Box(modifier.then(toggle).size(18.dp).dsDisabled(!enabled), contentAlignment = Alignment.Center) {
        Box(Modifier.size(18.dp).clip(shape).background(fill).border(2.dp, edge, shape), contentAlignment = Alignment.Center) {
            AnimatedVisibility(checked, enter = scaleIn(DsMotion.spatial()) + fadeIn(DsMotion.fast()), exit = scaleOut(DsMotion.fast()) + fadeOut(DsMotion.fast())) {
                DsIcon(Lucide.Check, 14.dp, MaterialTheme.colorScheme.onPrimary)
            }
        }
    }
}

/** A real checkbox with its text label: the whole row toggles it. */
@Composable
fun DsCheckbox(label: String, checked: Boolean, onCheckedChange: (Boolean) -> Unit, modifier: Modifier = Modifier, enabled: Boolean = true) {
    Row(
        modifier.heightIn(min = MinTouch).toggleable(checked, enabled = enabled, role = Role.Checkbox, onValueChange = onCheckedChange).dsDisabled(!enabled),
        horizontalArrangement = Arrangement.spacedBy(12.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        CheckMark(checked, null)
        Text(label, style = Ds.body, color = Ds.text)
    }
}

/** A radio dot (16dp), its ring thickening to the accent when chosen. */
@Composable
fun RadioMark(selected: Boolean, modifier: Modifier = Modifier) {
    val ring by animateDpAsState(if (selected) 5.dp else Hairline, DsMotion.spatial(), label = "radio-ring")
    val color by animateColorAsState(if (selected) Ds.accent else Ds.borderStrong, DsMotion.fast(), label = "radio-color")
    Box(modifier.size(16.dp).clip(DsShapes.full).border(ring, color, DsShapes.full))
}

/** A switch with its label said (web Toggle on the M3 switch). */
@Composable
fun DsToggle(label: String, checked: Boolean, onCheckedChange: (Boolean) -> Unit, modifier: Modifier = Modifier, enabled: Boolean = true) {
    NomiSwitch(checked, onCheckedChange, modifier.semantics { contentDescription = label }, enabled)
}

enum class ChipKind { Model, Option, Agent, Provision }

/**
 * frontend-new's Chip. Model, option and agent are assist chips that open a menu (a chevron says so); provision
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
            modifier.height(28.dp).clip(DsShapes.sm).background(Ds.surface2).padding(horizontal = 10.dp),
            horizontalArrangement = Arrangement.spacedBy(6.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            if (icon != null) DsIcon(icon, 14.dp, Ds.text2)
            Text(label, style = Ds.mono, color = Ds.text2)
        }
        return
    }
    val shape = MaterialTheme.shapes.small
    Row(
        modifier
            .height(32.dp)
            .clip(shape)
            .border(Hairline, MaterialTheme.colorScheme.outline, shape)
            .clickable(enabled = onClick != null, role = Role.DropdownList) { onClick?.invoke() }
            .nomiFocusRing(shape)
            .padding(start = if (kind == ChipKind.Model) 12.dp else 8.dp, end = 8.dp),
        horizontalArrangement = Arrangement.spacedBy(8.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        when {
            kind == ChipKind.Agent && agent != null -> Mascot(agent, 20.dp)
            kind == ChipKind.Option && icon != null -> DsIcon(icon, 18.dp, Ds.accent)
        }
        Text(label, style = if (kind == ChipKind.Model) Ds.mono else MaterialTheme.typography.labelLarge, color = Ds.text2)
        val turn by animateFloatAsState(if (expanded) HALF_TURN else 0f, DsMotion.spatial(), label = "chip-chevron")
        DsIcon(Lucide.ChevronDown, 18.dp, Ds.text2, Modifier.rotate(turn))
    }
}

private const val HALF_TURN = 180f

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
        textStyle = style.copy(color = Ds.text),
        cursorBrush = SolidColor(Ds.accent),
        keyboardOptions = KeyboardOptions(keyboardType = keyboard),
        visualTransformation = if (secret) PasswordVisualTransformation() else VisualTransformation.None,
        interactionSource = interaction,
        decorationBox = { inner ->
            Box(contentAlignment = Alignment.CenterStart) {
                if (value.isEmpty()) Text(placeholder, style = style, color = Ds.text3)
                inner()
            }
        },
    )
}

/**
 * frontend-new's Search on the M3 search bar: a full pill with the icon and its shortcut key. [plain] is the
 * settings nav's, shorter, with the shortcut as plain mono text.
 */
@Composable
fun DsSearch(value: String, onValueChange: (String) -> Unit, placeholder: String, modifier: Modifier = Modifier, shortcut: String? = null, plain: Boolean = false) {
    val interaction = remember { MutableInteractionSource() }
    val focused by interaction.collectIsFocusedAsState()
    val ring by animateColorAsState(if (focused) Ds.accent else Color.Transparent, DsMotion.fast(), label = "search-ring")
    Row(
        modifier
            .fillMaxWidth()
            .height(if (plain) 40.dp else 48.dp)
            .clip(DsShapes.full)
            .background(Ds.surface2)
            .border(2.dp, ring, DsShapes.full)
            .padding(horizontal = if (plain) 12.dp else 16.dp),
        horizontalArrangement = Arrangement.spacedBy(12.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        DsIcon(Lucide.Search, 20.dp, Ds.text2)
        DsBareField(value, onValueChange, placeholder, Ds.body, Modifier.weight(1f), interaction = interaction)
        when {
            shortcut != null && plain -> Text(shortcut, style = Ds.label, color = Ds.text2)
            shortcut != null -> Kbd(shortcut)
        }
    }
}

data class SegmentOption(val value: String, val label: String, val count: String? = null)

enum class SegmentedSurface { Filled, Bg, Surface }

/**
 * frontend-new's Segmented: an M3 segmented button for a few mutually exclusive views. The chosen one fills in
 * secondary-container with a check that grows in.
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
    val shape = DsShapes.full
    val bg = when (surface) {
        SegmentedSurface.Filled -> Color.Transparent
        SegmentedSurface.Bg -> Ds.bg
        SegmentedSurface.Surface -> Ds.surface
    }
    Row(
        modifier
            .height(IntrinsicSize.Min)
            .clip(shape)
            .background(bg)
            .border(Hairline, MaterialTheme.colorScheme.outline, shape)
            .selectableGroup()
            .semantics { contentDescription = label },
    ) {
        options.forEachIndexed { index, option ->
            if (index > 0) Box(Modifier.width(Hairline).fillMaxHeight().background(MaterialTheme.colorScheme.outline))
            val checked = option.value == value
            val fill by animateColorAsState(if (checked) MaterialTheme.colorScheme.secondaryContainer else Color.Transparent, DsMotion.fast(), label = "segment-fill")
            val ink = if (checked) MaterialTheme.colorScheme.onSecondaryContainer else Ds.text
            Row(
                Modifier
                    .heightIn(min = 40.dp)
                    .background(fill)
                    .selectable(checked, role = Role.RadioButton) { onChange(option.value) }
                    .padding(horizontal = 16.dp),
                horizontalArrangement = Arrangement.spacedBy(8.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                AnimatedVisibility(checked, enter = scaleIn(DsMotion.spatial()) + fadeIn(DsMotion.fast()), exit = scaleOut(DsMotion.fast()) + fadeOut(DsMotion.fast())) {
                    DsIcon(Lucide.Check, 18.dp, ink)
                }
                Text(option.label, style = MaterialTheme.typography.labelLarge, color = ink)
                if (option.count != null) Text(option.count, style = Ds.label, color = ink.copy(alpha = COUNT_ALPHA))
            }
        }
    }
}

private const val COUNT_ALPHA = 0.72f

data class TabItem(val value: String, val label: String)

/**
 * frontend-new's Tabs on M3 primary tabs: the open one in primary, its indicator sliding under the label. [panel]
 * draws the open tab's content, cross-fading as tabs change.
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
    val density = LocalDensity.current
    val bounds = remember { mutableStateMapOf<String, Pair<Dp, Dp>>() }
    val target = bounds[value]
    val left by animateDpAsState(target?.first ?: 0.dp, DsMotion.spatial(), label = "tab-left")
    val width by animateDpAsState(target?.second ?: 0.dp, DsMotion.spatial(), label = "tab-width")
    Column(modifier.fillMaxWidth()) {
        Box(Modifier.fillMaxWidth().horizontalScroll(rememberScrollState())) {
            Row(Modifier.selectableGroup().semantics { contentDescription = label }) {
                tabs.forEach { tab ->
                    val selected = tab.value == value
                    val color by animateColorAsState(if (selected) Ds.accent else Ds.text2, DsMotion.fast(), label = "tab-color")
                    Box(
                        Modifier
                            .height(48.dp)
                            .onGloballyPositioned { coords ->
                                with(density) { bounds[tab.value] = coords.positionInParent().x.toDp() to coords.size.width.toDp() }
                            }
                            .selectable(selected, role = Role.Tab) { onChange(tab.value) }
                            .padding(horizontal = 16.dp),
                        contentAlignment = Alignment.Center,
                    ) { Text(tab.label, style = MaterialTheme.typography.titleSmall, color = color) }
                }
            }
            if (target != null) {
                Box(
                    Modifier
                        .align(Alignment.BottomStart)
                        .offset(x = left + TAB_INSET)
                        .width((width - TAB_INSET * 2).coerceAtLeast(0.dp))
                        .height(3.dp)
                        .clip(RoundedCornerShape(topStart = 3.dp, topEnd = 3.dp))
                        .background(Ds.accent),
                )
            }
        }
        DsRule(color = MaterialTheme.colorScheme.surfaceContainerHighest)
        if (panel != null) {
            Crossfade(value, animationSpec = DsMotion.effects(), label = "tab-panel") { panel(it) }
        }
    }
}

private val TAB_INSET = 16.dp

/**
 * frontend-new's TextField as the M3 outlined field: label · field · helper. Focus thickens the outline to the
 * accent; an error swaps it for the error colour and takes the helper's place to say what to do next.
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
) {
    val interaction = remember { MutableInteractionSource() }
    val focused by interaction.collectIsFocusedAsState()
    val active = focused || showFocus
    val edge by animateColorAsState(
        when {
            error != null -> Ds.danger
            active -> Ds.accent
            else -> MaterialTheme.colorScheme.outline
        },
        DsMotion.fast(),
        label = "field-edge",
    )
    val thickness by animateDpAsState(if (active) 2.dp else 1.5.dp, DsMotion.fast(), label = "field-thickness")
    val shape = MaterialTheme.shapes.large
    val note = error ?: helper
    Column(modifier.fillMaxWidth().dsDisabled(!enabled), verticalArrangement = Arrangement.spacedBy(4.dp)) {
        Text(label, style = Ds.caption, color = Ds.text2)
        Row(
            Modifier
                .fillMaxWidth()
                .height(48.dp)
                .clip(shape)
                .background(MaterialTheme.colorScheme.surface)
                .border(thickness, edge, shape)
                .padding(horizontal = 14.dp)
                .then(if (error != null) Modifier.semantics { error(error) } else Modifier),
            horizontalArrangement = Arrangement.spacedBy(8.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            DsBareField(
                value, onValueChange, placeholder,
                if (mono) Ds.body.copy(fontFamily = Ds.label.fontFamily) else Ds.body,
                Modifier.weight(1f), secret = secret, keyboard = keyboard, interaction = interaction, enabled = enabled,
            )
            if (error != null) DsIcon(Lucide.Alert, 20.dp, Ds.danger)
        }
        if (note != null) {
            AnimatedContent(note, label = "field-note") { text ->
                Text(text, style = Ds.caption, color = if (error != null) Ds.danger else Ds.text2)
            }
        }
    }
}
