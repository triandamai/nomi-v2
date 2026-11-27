package ai.nomi.designsystem.ds

import ai.nomi.designsystem.component.NomiAvatar
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

/** Who is speaking, and when (mono: "15:37 · 4m 12s"). */
@Composable
fun AgentByline(agent: Crew, name: String, modifier: Modifier = Modifier, meta: String? = null, working: Boolean = false) {
    Row(modifier, horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
        Mascot(agent, 20.dp, working = working)
        Text(name, style = MaterialTheme.typography.titleSmall, color = Ds.text)
        if (meta != null) Text(meta, style = Ds.label, color = Ds.text2)
    }
}

/** A person: their initials on the secondary container (32dp). */
@Composable
fun DsAvatar(name: String, modifier: Modifier = Modifier, size: Dp = 32.dp) {
    NomiAvatar(name, modifier.semantics { contentDescription = name }, size = size)
}

/** A crew member in a list, with what they do and their state. */
@Composable
fun CrewRow(agent: Crew, name: String, role: String, modifier: Modifier = Modifier, state: String? = null, onClick: (() -> Unit)? = null) {
    Row(
        modifier
            .fillMaxWidth()
            .heightIn(min = 56.dp)
            .clip(MaterialTheme.shapes.large)
            .then(if (onClick != null) Modifier.clickable(role = Role.Button, onClick = onClick) else Modifier)
            .padding(horizontal = 16.dp, vertical = 8.dp),
        horizontalArrangement = Arrangement.spacedBy(16.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Mascot(agent, 32.dp)
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
            Text(name, style = Ds.body, color = Ds.text)
            Text(role, style = Ds.ui, color = Ds.text2)
        }
        if (state != null) Text(state, style = Ds.label, color = Ds.text2)
    }
}

/**
 * How much of something is used, as a bar and in words ("21.8K / 1M tokens this month"): never more than full.
 * The fill grows to its value. [roomy] spaces it out, as in the drawer's usage card.
 */
@Composable
fun Meter(label: String, value: Float, modifier: Modifier = Modifier, caption: String? = null, roomy: Boolean = false) {
    val clamped = value.coerceIn(0f, 1f)
    val percent = (clamped * PERCENT).toInt()
    val fill by animateFloatAsState(clamped, DsMotion.spatial(), label = "meter-fill")
    Column(
        modifier.fillMaxWidth().semantics { progressBarRangeInfo = ProgressBarRangeInfo(clamped, 0f..1f) },
        verticalArrangement = Arrangement.spacedBy(if (roomy) 10.dp else 6.dp),
    ) {
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
            Text(label, style = MaterialTheme.typography.titleSmall, color = Ds.text)
            Text("$percent%", style = Ds.label, color = Ds.text2)
        }
        Box(Modifier.fillMaxWidth().height(4.dp).clip(DsShapes.full).background(MaterialTheme.colorScheme.secondaryContainer)) {
            Box(Modifier.fillMaxWidth(fill).fillMaxHeight().clip(DsShapes.full).background(Ds.accent))
        }
        if (caption != null) Text(caption, style = Ds.label, color = Ds.text2)
    }
}

/** A section's name in the mono meta face, and how many are in it. */
@Composable
fun SectionLabel(label: String, modifier: Modifier = Modifier, count: String? = null) {
    Row(modifier, horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
        Text(label, style = Ds.labelCaps, color = Ds.text2)
        if (count != null) Text(count, style = Ds.label, color = Ds.text2)
    }
}

enum class StatusKind { Live, Ready, Idle, Failed, Attention, Working }

/**
 * A dot plus a word, so colour never carries meaning alone. Who is working shows their shape (moving) instead of a
 * dot. [small] sets the word smaller.
 */
@Composable
fun DsStatus(kind: StatusKind, label: String, modifier: Modifier = Modifier, agent: Crew? = null, small: Boolean = false) {
    val color = when (kind) {
        StatusKind.Failed -> Ds.danger
        StatusKind.Attention -> Ds.warning
        StatusKind.Idle, StatusKind.Ready -> Ds.text2
        else -> Ds.text
    }
    val dot = if (kind == StatusKind.Live || kind == StatusKind.Ready) Ds.accent else color
    val quiet = small || kind == StatusKind.Ready || kind == StatusKind.Idle
    Row(modifier, horizontalArrangement = Arrangement.spacedBy(6.dp), verticalAlignment = Alignment.CenterVertically) {
        if (kind == StatusKind.Working && agent != null) {
            Mascot(agent, 20.dp, working = true)
        } else {
            Box(Modifier.size(8.dp).clip(CircleShape).background(dot))
        }
        Text(label, style = if (quiet) MaterialTheme.typography.labelSmall else MaterialTheme.typography.labelLarge, color = color)
    }
}

enum class TagKind { Go, NoGo, Neutral }

/** A go / no-go / standby verdict, in the meta face, upper case. */
@Composable
fun DsTag(kind: TagKind, label: String, modifier: Modifier = Modifier) {
    val (bg, fg) = when (kind) {
        TagKind.Go -> MaterialTheme.colorScheme.primaryContainer to MaterialTheme.colorScheme.onPrimaryContainer
        TagKind.NoGo -> MaterialTheme.colorScheme.errorContainer to MaterialTheme.colorScheme.onErrorContainer
        TagKind.Neutral -> MaterialTheme.colorScheme.surfaceContainerHigh to MaterialTheme.colorScheme.onSurfaceVariant
    }
    Box(modifier.height(24.dp).clip(DsShapes.sm).background(bg).padding(horizontal = 8.dp), contentAlignment = Alignment.Center) {
        Text(label.uppercase(), style = Ds.labelCaps.copy(fontWeight = FontWeight.Medium), color = fg)
    }
}

/** A vertical hairline between groups in a row of tools. */
@Composable
fun DsVerticalRule(modifier: Modifier = Modifier, height: Dp = 14.dp, color: Color = Ds.borderStrong) {
    Box(modifier.width(Hairline).height(height).background(color))
}
