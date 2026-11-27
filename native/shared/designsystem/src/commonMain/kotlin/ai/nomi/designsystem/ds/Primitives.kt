package ai.nomi.designsystem.ds

import ai.nomi.designsystem.component.NomiAvatar
import ai.nomi.designsystem.component.computeInitials
import androidx.compose.foundation.border
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsHoveredAsState
import androidx.compose.foundation.text.BasicText
import androidx.compose.runtime.remember
import androidx.compose.ui.semantics.hideFromAccessibility
import androidx.compose.ui.text.style.TextOverflow
import kotlin.math.roundToInt
import ai.nomi.designsystem.component.NomiButton
import ai.nomi.designsystem.component.NomiButtonVariant
import ai.nomi.designsystem.component.nomiFocusRing
import ai.nomi.designsystem.icon.Lucide
import ai.nomi.designsystem.icon.NomiGlyph
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.FilledIconButton
import androidx.compose.material3.IconButton
import androidx.compose.material3.IconButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedIconButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.progressBarRangeInfo
import androidx.compose.ui.semantics.ProgressBarRangeInfo
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

private const val PERCENT = 100

/** Web Button variants: primary is the hero (action gradient, one per view). */
/** A small bare icon you can press (a card header's copy, a row's remove): 14dp glyph, 44dp target. */
@Composable
fun DsBareIcon(icon: NomiGlyph, label: String, onClick: () -> Unit, modifier: Modifier = Modifier, size: Dp = 14.dp, tint: Color = Ds.text3, enabled: Boolean = true) {
    Box(
        modifier
            .size(MinTouch)
            .clip(DsShapes.sm)
            .clickable(enabled = enabled, role = Role.Button, onClick = onClick)
            .semantics { contentDescription = label }
            .dsDisabled(!enabled),
        contentAlignment = Alignment.Center,
    ) { DsIcon(icon, size, tint) }
}

// Design System / Nomi → Identity and Status (frontend-new `ds/identity`, `ds/status`), in dp.
private val BylineGap = 8.dp
private val BylineMascot = 20.dp
private val AvatarSize = 28.dp
private val CrewRowMascot = 32.dp
private val CrewRowGap = 12.dp
private val CrewRowPadX = 12.dp
private val CrewRowPadY = 8.dp
private val CrewRowLines = 2.dp
private val MeterTrack = 4.dp
private val MeterGap = 6.dp
private val MeterRoomyGap = 10.dp
private val StatusDot = 6.dp
private val StatusGap = 6.dp
private val StatusMascot = 20.dp
private val TagPadX = 5.dp
private val TagPadY = 1.dp
private val SectionLabelGap = 8.dp

/** Who is speaking (13, medium), with their mascot, and when (mono: "15:37 · 4m 12s"). */
@Composable
fun AgentByline(agent: Crew, name: String, modifier: Modifier = Modifier, meta: String? = null, working: Boolean = false) {
    Row(modifier, horizontalArrangement = Arrangement.spacedBy(BylineGap), verticalAlignment = Alignment.CenterVertically) {
        Mascot(agent, BylineMascot, working = working)
        BasicText(name, style = Ds.title.copy(color = Ds.text))
        if (meta != null) BasicText(meta, style = Ds.label.copy(color = Ds.text3))
    }
}

/** A person: their picture, or their initials in mono, on a 28 tile with a border-strong edge. */
@Composable
fun DsAvatar(name: String, modifier: Modifier = Modifier, size: Dp = AvatarSize, src: String? = null) {
    if (src != null) {
        NomiAvatar(name, modifier.semantics { contentDescription = name }.clip(DsShapes.sm).border(Hairline, Ds.borderStrong, DsShapes.sm), avatarUrl = src, size = size)
        return
    }
    Box(
        modifier
            .semantics { contentDescription = name }
            .size(size)
            .clip(DsShapes.sm)
            .background(Ds.surface2)
            .border(Hairline, Ds.borderStrong, DsShapes.sm),
        contentAlignment = Alignment.Center,
    ) { BasicText(computeInitials(name), style = Ds.label.copy(color = Ds.text2), modifier = Modifier.semantics { hideFromAccessibility() }) }
}

/**
 * A crew member in a list, with what they do (on one line, cut short) and their state. Pressable when [onClick] is
 * given, darkening on hover.
 */
@Composable
fun CrewRow(agent: Crew, name: String, role: String, modifier: Modifier = Modifier, state: String? = null, onClick: (() -> Unit)? = null) {
    val interaction = remember { MutableInteractionSource() }
    val hovered by interaction.collectIsHoveredAsState()
    Row(
        modifier
            .fillMaxWidth()
            .clip(DsShapes.sm)
            .background(if (onClick != null && hovered) Ds.surface2 else Color.Transparent)
            .then(if (onClick != null) Modifier.clickable(interaction, indication = null, role = Role.Button, onClick = onClick) else Modifier)
            .padding(horizontal = CrewRowPadX, vertical = CrewRowPadY),
        horizontalArrangement = Arrangement.spacedBy(CrewRowGap),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Mascot(agent, CrewRowMascot)
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(CrewRowLines)) {
            BasicText(name, style = Ds.title.copy(color = Ds.text))
            BasicText(role, style = Ds.caption.copy(color = Ds.text3), maxLines = 1, overflow = TextOverflow.Ellipsis)
        }
        if (state != null) BasicText(state, style = Ds.label.copy(color = Ds.text3))
    }
}

/**
 * How much of something is used, as a thin text-2 bar and in words ("21.8K / 1M tokens this month"): the real figure,
 * never more than full. [roomy] spaces it 10 apart, as in the sidebar's usage card.
 */
@Composable
fun Meter(label: String, value: Float, modifier: Modifier = Modifier, caption: String? = null, roomy: Boolean = false) {
    val clamped = value.coerceIn(0f, 1f)
    val percent = (clamped * PERCENT).roundToInt()
    Column(
        modifier.fillMaxWidth().semantics(mergeDescendants = true) { progressBarRangeInfo = ProgressBarRangeInfo(clamped, 0f..1f) },
        verticalArrangement = Arrangement.spacedBy(if (roomy) MeterRoomyGap else MeterGap),
    ) {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = if (roomy) Alignment.CenterVertically else Alignment.Top) {
            BasicText(label, style = Ds.title.copy(color = Ds.text))
            BasicText("$percent%", style = Ds.label.copy(color = Ds.text3))
        }
        Box(Modifier.fillMaxWidth().height(MeterTrack).clip(DsShapes.full).background(Ds.surface2)) {
            Box(Modifier.fillMaxWidth(clamped).fillMaxHeight().clip(DsShapes.full).background(Ds.text2))
        }
        if (caption != null) BasicText(caption, style = Ds.label.copy(color = Ds.text3))
    }
}

/** The only uppercase style: a section's name in mono capitals, in text-3, and how many are in it. */
@Composable
fun SectionLabel(label: String, modifier: Modifier = Modifier, count: String? = null) {
    Row(modifier, horizontalArrangement = Arrangement.spacedBy(SectionLabelGap), verticalAlignment = Alignment.CenterVertically) {
        BasicText(label.uppercase(), style = Ds.labelCaps.copy(color = Ds.text3))
        if (count != null) BasicText(count, style = Ds.label.copy(color = Ds.text3))
    }
}

enum class StatusKind { Live, Ready, Idle, Failed, Attention, Working }

/**
 * A 6 dot plus a word, so colour never carries meaning alone. Who is working shows their mascot instead of a dot.
 * States, not decoration: no pulsing. [small] sets the word at 11 where the pen does (a build's "Live").
 */
@Composable
fun DsStatus(kind: StatusKind, label: String, modifier: Modifier = Modifier, agent: Crew? = null, small: Boolean = false) {
    val (dot, ink) = when (kind) {
        StatusKind.Live -> Ds.accent to Ds.text
        StatusKind.Ready -> Ds.accent to Ds.text2
        StatusKind.Idle -> Ds.text3 to Ds.text3
        StatusKind.Failed -> Ds.danger to Ds.danger
        StatusKind.Attention -> Ds.warning to Ds.warning
        StatusKind.Working -> Ds.text to Ds.text
    }
    val style = when {
        kind == StatusKind.Ready || kind == StatusKind.Idle -> Ds.label
        small -> Ds.ui.copy(fontSize = Ds.label.fontSize, lineHeight = Ds.label.lineHeight)
        else -> Ds.caption
    }
    Row(modifier.semantics(mergeDescendants = true) {}, horizontalArrangement = Arrangement.spacedBy(StatusGap), verticalAlignment = Alignment.CenterVertically) {
        if (kind == StatusKind.Working && agent != null) {
            Mascot(agent, StatusMascot, working = true)
        } else {
            Box(Modifier.size(StatusDot).clip(CircleShape).background(dot))
        }
        BasicText(label, style = style.copy(color = ink))
    }
}

enum class TagKind { Go, NoGo, Neutral }

/** A go / no-go / standby verdict in small mono capitals, 18 tall: written in sentence case, shown in capitals. */
@Composable
fun DsTag(kind: TagKind, label: String, modifier: Modifier = Modifier) {
    val (fill, edge, ink) = when (kind) {
        TagKind.Go -> Triple(Ds.accentBg, Ds.accentBg, Ds.accent)
        TagKind.NoGo -> Triple(Ds.dangerBg, Ds.dangerBorder, Ds.danger)
        TagKind.Neutral -> Triple(Ds.surface2, Ds.borderStrong, Ds.text2)
    }
    Box(
        modifier
            .semantics(mergeDescendants = true) {}
            .clip(DsShapes.xs)
            .background(fill)
            .border(Hairline, edge, DsShapes.xs)
            .padding(horizontal = TagPadX + Hairline, vertical = TagPadY + Hairline),
        contentAlignment = Alignment.Center,
    ) { BasicText(label.uppercase(), style = Ds.labelCaps.copy(color = ink)) }
}

/** A vertical hairline between groups in a row of tools. */
@Composable
fun DsVerticalRule(modifier: Modifier = Modifier, height: Dp = 14.dp, color: Color = Ds.borderStrong) {
    Box(modifier.width(Hairline).height(height).background(color))
}
