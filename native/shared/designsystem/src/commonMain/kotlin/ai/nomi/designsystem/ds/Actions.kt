package ai.nomi.designsystem.ds

import ai.nomi.designsystem.icon.Lucide
import ai.nomi.designsystem.icon.NomiGlyph
import ai.nomi.designsystem.theme.NomiTheme
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsFocusedAsState
import androidx.compose.foundation.interaction.collectIsHoveredAsState
import androidx.compose.foundation.interaction.collectIsPressedAsState
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.text.BasicText
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.graphics.drawOutline
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.graphics.drawscope.translate
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

// Design System / Nomi → Actions (frontend-new `ds/actions`). Heights and paddings are the web's, in dp.
private val ButtonHeight = 30.dp
private val ButtonPadding = 12.dp
private val ButtonGap = 6.dp
private val ButtonIcon = 14.dp
private val IconButtonSize = 30.dp
private val IconButtonGlyph = 16.dp
private val IconButtonSmallGlyph = 14.dp
private val CoarseTarget = 44.dp
private val ButtonRing = 3.dp
private val IconButtonRing = 2.dp
private val IconButtonRingOffset = 1.dp
private val KbdHeight = 20.dp
private val KbdPadding = 5.dp
private val KeyHintGapX = 16.dp
private val KeyHintGapY = 4.dp
private val LinkGap = 4.dp
private val LinkIcon = 14.dp
private val PillHeight = 40.dp
private val PillPadding = 16.dp
private val PillIcon = 16.dp
private const val DISABLED_OPACITY = 0.4f
private const val PILL_LOADING_OPACITY = 0.6f

/** The web's `h-…` on a fine pointer, 44 (`pointer-coarse:min-h-11`) on a finger. */
@Composable
internal fun targetHeight(fine: Dp): Dp = if (NomiTheme.coarsePointer) maxOf(fine, CoarseTarget) else fine

/**
 * The pen's focus ring: [width] of `focus` drawn [offset] outside [shape] while focused. It takes no layout space and
 * sits outside the fill, so it shows on mint too.
 */
fun Modifier.dsFocusRing(focused: Boolean, shape: Shape, color: Color, width: Dp, offset: Dp = 0.dp): Modifier = drawWithContent {
    drawContent()
    if (focused) {
        val outset = (offset + width / 2).toPx()
        translate(-outset, -outset) {
            val ring = Size(size.width + outset * 2, size.height + outset * 2)
            drawOutline(shape.createOutline(ring, layoutDirection, this), color, style = Stroke(width.toPx()))
        }
    }
}

enum class DsButtonVariant { Primary, Inverse, Secondary, Ghost, GhostDanger, Danger }

private class ButtonLook(val fill: Color, val content: Color, val glyph: Color, val edge: Color?, val weight: FontWeight)

@Composable
private fun buttonLook(variant: DsButtonVariant, hovered: Boolean, pressed: Boolean, focused: Boolean): ButtonLook = when (variant) {
    DsButtonVariant.Primary -> ButtonLook(Ds.accent, Ds.accentInk, Ds.accentInk, null, FontWeight.Medium)
    DsButtonVariant.Inverse -> ButtonLook(Ds.text, Ds.bg, Ds.bg, null, FontWeight.Medium)
    DsButtonVariant.Secondary -> ButtonLook(
        fill = if (pressed) Ds.border else if (hovered) Ds.surface2 else Color.Transparent,
        content = Ds.text,
        glyph = Ds.text2,
        edge = if (focused) Color.Transparent else Ds.borderStrong,
        weight = FontWeight.Normal,
    )
    DsButtonVariant.Ghost -> ButtonLook(
        fill = if (hovered) Ds.surface2 else Color.Transparent,
        content = if (hovered) Ds.text else Ds.text2,
        glyph = Ds.text3,
        edge = null,
        weight = FontWeight.Normal,
    )
    DsButtonVariant.GhostDanger -> ButtonLook(if (hovered) Ds.surface2 else Color.Transparent, Ds.danger, Ds.danger, null, FontWeight.Normal)
    DsButtonVariant.Danger -> ButtonLook(Ds.dangerBg, Ds.danger, Ds.danger, if (hovered) Ds.danger else Ds.dangerBorder, FontWeight.Medium)
}

/**
 * A button: 30 tall, radius-sm, a 13 label. One primary per view; Inverse for the main action inside a panel, so mint
 * stays rare. Loading shows real progress in its label ("Deploying · 00:14") and takes no second press, without
 * looking disabled.
 */
@Composable
fun DsButton(
    label: String,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    variant: DsButtonVariant = DsButtonVariant.Primary,
    iconStart: NomiGlyph? = null,
    iconEnd: NomiGlyph? = null,
    loading: Boolean = false,
    enabled: Boolean = true,
) {
    val interaction = remember { MutableInteractionSource() }
    val active = enabled && !loading
    val hovered by interaction.collectIsHoveredAsState()
    val pressed by interaction.collectIsPressedAsState()
    var focused by remember { mutableStateOf(false) }
    val look = buttonLook(variant, active && hovered, active && pressed, focused)
    val leading = if (loading) Lucide.Loader else iconStart
    Row(
        modifier
            .alpha(if (!enabled && !loading) DISABLED_OPACITY else 1f)
            .height(targetHeight(ButtonHeight))
            .dsFocusRing(focused, DsShapes.sm, Ds.focus, ButtonRing)
            .clip(DsShapes.sm)
            .background(look.fill)
            .then(if (look.edge != null) Modifier.border(Hairline, look.edge, DsShapes.sm) else Modifier)
            .onFocusChanged { focused = it.isFocused }
            .clickable(interaction, indication = null, enabled = active, role = Role.Button, onClick = onClick)
            .padding(horizontal = ButtonPadding),
        horizontalArrangement = Arrangement.spacedBy(ButtonGap),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        if (leading != null) DsIcon(leading, ButtonIcon, look.glyph, if (loading) Modifier.dsSpin() else Modifier)
        BasicText(label, style = Ds.ui.copy(color = look.content, fontWeight = look.weight), maxLines = 1, softWrap = false, overflow = TextOverflow.Clip)
        if (iconEnd != null && !loading) DsIcon(iconEnd, ButtonIcon, look.glyph)
    }
}

enum class DsIconButtonVariant { Outline, Ghost, Send }

/**
 * A 30 square with a 16 glyph ([small]: 14, a card header's tools), always named by [label]. Send is mint, and grey
 * rather than faded when there's nothing to send.
 */
@Composable
fun DsIconButton(
    icon: NomiGlyph,
    label: String,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    variant: DsIconButtonVariant = DsIconButtonVariant.Outline,
    small: Boolean = false,
    enabled: Boolean = true,
) {
    val interaction = remember { MutableInteractionSource() }
    val hovered by interaction.collectIsHoveredAsState()
    var focused by remember { mutableStateOf(false) }
    val hover = enabled && hovered
    val (fill, tint) = when (variant) {
        DsIconButtonVariant.Outline -> (if (hover) Ds.surface2 else Color.Transparent) to (if (hover) Ds.text else Ds.text2)
        DsIconButtonVariant.Ghost -> (if (hover) Ds.surface2 else Color.Transparent) to (if (hover) Ds.text else Ds.text3)
        DsIconButtonVariant.Send -> if (enabled) Ds.accent to Ds.accentInk else Ds.surface2 to Ds.text3
    }
    val side = targetHeight(IconButtonSize)
    Box(
        modifier
            .size(side)
            .dsFocusRing(focused, DsShapes.sm, Ds.focus, IconButtonRing, IconButtonRingOffset)
            .clip(DsShapes.sm)
            .background(fill)
            .then(if (variant == DsIconButtonVariant.Outline) Modifier.border(Hairline, Ds.borderStrong, DsShapes.sm) else Modifier)
            .onFocusChanged { focused = it.isFocused }
            .clickable(interaction, indication = null, enabled = enabled, role = Role.Button, onClick = onClick)
            .semantics { contentDescription = label },
        contentAlignment = Alignment.Center,
    ) { DsIcon(icon, if (small) IconButtonSmallGlyph else IconButtonGlyph, tint) }
}

/** A keyboard key or chord (⌘K): a 20 keycap, mono 11, read as one. */
@Composable
fun Kbd(keys: String, modifier: Modifier = Modifier) {
    Box(
        modifier
            .semantics(mergeDescendants = true) {}
            .height(KbdHeight)
            .clip(DsShapes.xs)
            .border(Hairline, Ds.borderStrong, DsShapes.xs)
            .padding(horizontal = KbdPadding),
        contentAlignment = Alignment.Center,
    ) { BasicText(keys, style = Ds.label.copy(color = Ds.text2)) }
}

data class KeyHintItem(val keys: String, val action: String)

/** What the keys do here, in a quiet mono line: "↵ send   ⇧↵ new line". */
@Composable
fun KeyHint(hints: List<KeyHintItem>, modifier: Modifier = Modifier) {
    FlowRow(modifier, horizontalArrangement = Arrangement.spacedBy(KeyHintGapX), verticalArrangement = Arrangement.spacedBy(KeyHintGapY)) {
        hints.forEach { BasicText("${it.keys} ${it.action}", style = Ds.label.copy(color = Ds.text3)) }
    }
}

/**
 * A link on its own line ("Upgrade to Pro ↗", standalone: 13, 500, with its arrow) or inside a sentence (inline,
 * underlined on hover). [quiet] is a card footer's secondary link, in regular weight.
 */
@Composable
fun DsLink(label: String, onClick: () -> Unit, modifier: Modifier = Modifier, inline: Boolean = false, quiet: Boolean = false) {
    val interaction = remember { MutableInteractionSource() }
    val hovered by interaction.collectIsHoveredAsState()
    val color = when {
        inline || quiet -> if (hovered) Ds.text else Ds.text2
        else -> Ds.text
    }
    val weight = if (inline || quiet) FontWeight.Normal else FontWeight.Medium
    Row(
        modifier.clip(DsShapes.xs).clickable(interaction, indication = null, role = Role.Button, onClick = onClick),
        horizontalArrangement = Arrangement.spacedBy(LinkGap),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        BasicText(
            label,
            style = Ds.ui.copy(color = color, fontWeight = weight, textDecoration = if (hovered) TextDecoration.Underline else TextDecoration.None),
        )
        if (!inline) DsIcon(Lucide.ArrowUpRight, LinkIcon, Ds.text2)
    }
}

enum class DsPhonePillVariant { Inverse, Secondary, Ghost }

/** A phone's pill button: 40 tall, fully round, 14 medium; ghost puts its icon first. */
@Composable
fun DsPhonePill(
    label: String,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    variant: DsPhonePillVariant = DsPhonePillVariant.Secondary,
    icon: NomiGlyph? = null,
    loading: Boolean = false,
) {
    val (fill, content) = when (variant) {
        DsPhonePillVariant.Inverse -> Ds.text to Ds.bg
        DsPhonePillVariant.Secondary -> Color.Transparent to Ds.text
        DsPhonePillVariant.Ghost -> Color.Transparent to Ds.text2
    }
    val iconFirst = variant == DsPhonePillVariant.Ghost
    Row(
        modifier
            .alpha(if (loading) PILL_LOADING_OPACITY else 1f)
            .height(PillHeight)
            .clip(DsShapes.full)
            .background(fill)
            .then(if (variant == DsPhonePillVariant.Secondary) Modifier.border(Hairline, Ds.borderStrong, DsShapes.full) else Modifier)
            .clickable(enabled = !loading, role = Role.Button, onClick = onClick)
            .padding(horizontal = PillPadding),
        horizontalArrangement = Arrangement.spacedBy(ButtonGap),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        if (icon != null && iconFirst) DsIcon(icon, PillIcon, content)
        BasicText(label, style = Ds.phoneSm.copy(color = content, fontWeight = FontWeight.Medium), maxLines = 1, softWrap = false)
        if (icon != null && !iconFirst) DsIcon(icon, PillIcon, content)
    }
}

/** GitHub's mark, in the line style of the other icons. */
@Composable
fun DsGithubMark(size: Dp, tint: Color, modifier: Modifier = Modifier) {
    DsIcon(Lucide.Github, size, tint, modifier)
}
