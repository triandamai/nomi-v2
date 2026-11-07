package ai.nomi.designsystem.ds.chat

import ai.nomi.designsystem.ds.Crew
import ai.nomi.designsystem.ds.Ds
import ai.nomi.designsystem.ds.footerNote
import ai.nomi.designsystem.ds.DsButton
import ai.nomi.designsystem.ds.DsButtonVariant
import ai.nomi.designsystem.ds.DsCard
import ai.nomi.designsystem.ds.DsCardTitle
import ai.nomi.designsystem.ds.DsChevron
import ai.nomi.designsystem.ds.DsFold
import ai.nomi.designsystem.ds.DsIcon
import ai.nomi.designsystem.ds.DsMotion
import ai.nomi.designsystem.ds.DsShapes
import ai.nomi.designsystem.ds.DsSpinner
import ai.nomi.designsystem.ds.DsTag
import ai.nomi.designsystem.ds.DsText
import ai.nomi.designsystem.ds.DsBareIcon
import ai.nomi.designsystem.ds.Hairline
import ai.nomi.designsystem.ds.Mascot
import ai.nomi.designsystem.ds.MinTouch
import ai.nomi.designsystem.ds.StepMark
import ai.nomi.designsystem.ds.StepState
import ai.nomi.designsystem.ds.TagKind
import ai.nomi.designsystem.ds.dsOutline
import ai.nomi.designsystem.ds.stepNumber
import ai.nomi.designsystem.icon.Lucide
import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.animateContentSize
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.stateDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp

private const val PARAGRAPH_BREAK = "\n\n"
private const val DRAGGING_ALPHA = 0.4f

/** Content set off by a hairline on its left, as the web's `border-l`. */
@Composable
internal fun LeftRuled(color: androidx.compose.ui.graphics.Color = Ds.border, content: @Composable () -> Unit) {
    Row(Modifier.fillMaxWidth().height(androidx.compose.foundation.layout.IntrinsicSize.Min).padding(start = 7.dp)) {
        androidx.compose.foundation.layout.Box(Modifier.width(Hairline).fillMaxHeight().background(color))
        androidx.compose.foundation.layout.Box(Modifier.padding(start = 12.dp)) { content() }
    }
}

/** A row you press to fold something open: its icon, label, meta and a turning chevron. */
@Composable
private fun FoldButton(open: Boolean, onToggle: () -> Unit, label: String, icon: ai.nomi.designsystem.icon.NomiGlyph, meta: String? = null) {
    Row(
        Modifier
            .heightIn(min = 32.dp)
            .clip(DsShapes.xs)
            .clickable(role = Role.Button, onClick = onToggle)
            .semantics { stateDescription = if (open) "Expanded" else "Collapsed" },
        horizontalArrangement = Arrangement.spacedBy(6.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        DsIcon(icon, 14.dp, Ds.text3)
        Text(label, style = Ds.ui, color = Ds.text2)
        if (meta != null) Text(meta, style = Ds.label, color = Ds.text3)
        DsChevron(open, 12.dp, Ds.text3)
    }
}

/**
 * How the model thought: while [live] its words stream under a turning loader with the time so far; after, it
 * folds behind "Thought for 12s" and opens on a press.
 */
@Composable
fun Reasoning(label: String, text: String, open: Boolean, onToggle: () -> Unit, modifier: Modifier = Modifier, live: Boolean = false, elapsed: String? = null) {
    val paragraphs = text.split(PARAGRAPH_BREAK).map { it.trim() }.filter { it.isNotEmpty() }
    val body: @Composable () -> Unit = {
        LeftRuled {
            Column(Modifier.fillMaxWidth().animateContentSize(DsMotion.spatial()), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                paragraphs.forEach { Text(it, style = Ds.ui, color = Ds.text3) }
            }
        }
    }
    Column(modifier.fillMaxWidth(), verticalArrangement = Arrangement.spacedBy(8.dp)) {
        if (live) {
            Row(horizontalArrangement = Arrangement.spacedBy(6.dp), verticalAlignment = Alignment.CenterVertically) {
                DsSpinner(14.dp, Ds.text2)
                Text(label, style = Ds.ui, color = Ds.text2)
                if (elapsed != null) Text(elapsed, style = Ds.label, color = Ds.text3)
            }
            body()
        } else {
            FoldButton(open, onToggle, label, Lucide.Brain)
            DsFold(open) { body() }
        }
    }
}

/** One decision the crew made on the way, said quietly. */
@Composable
fun Thought(text: String, modifier: Modifier = Modifier) {
    Row(modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        DsIcon(Lucide.Lightbulb, 14.dp, Ds.text3, Modifier.padding(top = 3.dp))
        Text(text, style = Ds.ui, color = Ds.text3)
    }
}

data class ThoughtStep(val label: String, val state: StepState, val meta: String? = null)

/** The steps the crew took, folding under "6 steps"; the one running turns and reads in full colour. */
@Composable
fun ThoughtSteps(label: String, steps: List<ThoughtStep>, open: Boolean, onToggle: () -> Unit, modifier: Modifier = Modifier, meta: String? = null) {
    Column(modifier.fillMaxWidth(), verticalArrangement = Arrangement.spacedBy(10.dp)) {
        FoldButton(open, onToggle, label, Lucide.ListTree, meta)
        DsFold(open) {
            Column(Modifier.fillMaxWidth().padding(start = 7.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
                steps.forEach { step ->
                    Row(horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                        AnimatedContent(step.state, Modifier.padding(top = 2.dp), label = "thought-step") { StepMark(it) }
                        Column(verticalArrangement = Arrangement.spacedBy(2.dp)) {
                            Text(step.label, style = Ds.ui, color = if (step.state == StepState.Running) Ds.text else Ds.text2)
                            if (step.meta != null) Text(step.meta, style = Ds.label, color = Ds.text3)
                        }
                    }
                }
            }
        }
    }
}

data class StepOwner(val agent: Crew, val name: String)
data class PlanStep(val label: String, val state: StepState, val owner: StepOwner, val meta: String? = null)

/** A plan the crew is running: each step numbered, its state, and the crew member who owns it. */
@Composable
fun PlanCard(subject: String, steps: List<PlanStep>, modifier: Modifier = Modifier, onStop: (() -> Unit)? = null) {
    val done = steps.count { it.state == StepState.Done }
    DsCard(
        modifier,
        header = {
            DsIcon(Lucide.ListChecks, 14.dp, Ds.text2)
            Text("Plan", style = Ds.ui.copy(fontWeight = FontWeight.Medium), color = Ds.text)
            DsText(subject, Ds.ui, Ds.text2, Modifier.weight(1f), singleLine = true)
            AnimatedContent(done, label = "plan-done") { Text("$it of ${steps.size} done", style = Ds.label, color = Ds.text3) }
        },
        footer = {
            Text("Each step goes to the crew member who owns it", style = Ds.caption, color = Ds.text3, modifier = Modifier.footerNote(this))
            if (onStop != null) DsButton("Stop plan", onStop, variant = DsButtonVariant.Ghost, iconStart = Lucide.Square)
        },
    ) {
        steps.forEachIndexed { index, step ->
            if (index > 0) HorizontalDivider(thickness = Hairline, color = Ds.border)
            Row(
                Modifier.fillMaxWidth().padding(start = 14.dp, end = 14.dp, top = if (index > 0) 9.dp else 10.dp, bottom = 10.dp),
                horizontalArrangement = Arrangement.spacedBy(12.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(stepNumber(index), style = Ds.label, color = Ds.text3)
                AnimatedContent(step.state, label = "plan-step") { StepMark(it) }
                Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
                    Text(step.label, style = Ds.ui, color = if (step.state == StepState.Waiting) Ds.text2 else Ds.text)
                    if (step.meta != null) Text(step.meta, style = Ds.label, color = Ds.text3)
                }
                Row(horizontalArrangement = Arrangement.spacedBy(6.dp), verticalAlignment = Alignment.CenterVertically) {
                    Mascot(step.owner.agent, 20.dp, working = step.state == StepState.Running)
                    Text(step.owner.name, style = Ds.caption, color = Ds.text2)
                }
            }
        }
    }
}

data class DraftStep(val label: String, val owner: StepOwner)

/**
 * A plan still being shaped: reorder its steps (up and down on the grip), give a step to someone else, remove or add
 * one, then run it, ask for changes, or discard it.
 */
@Composable
fun DraftPlanCard(
    intro: String,
    steps: List<DraftStep>,
    estimate: String,
    modifier: Modifier = Modifier,
    onMove: ((from: Int, to: Int) -> Unit)? = null,
    onRemove: ((Int) -> Unit)? = null,
    onAssign: ((Int) -> Unit)? = null,
    onAdd: (() -> Unit)? = null,
    onRun: () -> Unit = {},
    onChanges: () -> Unit = {},
    onDiscard: () -> Unit = {},
    dragging: Int? = null,
) {
    DsCard(
        modifier,
        header = {
            DsIcon(Lucide.FilePen, 14.dp, Ds.text2)
            DsCardTitle("Draft plan")
            DsTag(TagKind.Neutral, "Draft")
        },
        footer = {
            Text(estimate, style = Ds.label, color = Ds.text3, modifier = Modifier.footerNote(this))
            DsButton("Discard", onDiscard, variant = DsButtonVariant.Ghost, iconStart = Lucide.X)
            DsButton("Ask for changes", onChanges, variant = DsButtonVariant.Secondary)
            DsButton("Run plan", onRun, iconEnd = Lucide.Play)
        },
    ) {
        Text(intro, style = Ds.ui, color = Ds.text2, modifier = Modifier.padding(start = 14.dp, end = 14.dp, top = 12.dp, bottom = 4.dp))
        Column(Modifier.padding(horizontal = 14.dp, vertical = 4.dp)) {
            steps.forEachIndexed { index, step ->
                val alpha by animateFloatAsState(if (dragging == index) DRAGGING_ALPHA else 1f, DsMotion.fast(), label = "draft-drag")
                Row(
                    Modifier.fillMaxWidth().alpha(alpha).padding(vertical = 4.dp),
                    horizontalArrangement = Arrangement.spacedBy(6.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Column {
                        DsBareIcon(Lucide.ChevronUp, "Move step ${index + 1} up", { onMove?.invoke(index, index - 1) }, size = 12.dp, enabled = index > 0, modifier = Modifier.height(22.dp))
                        DsBareIcon(Lucide.ChevronDown, "Move step ${index + 1} down", { onMove?.invoke(index, index + 1) }, size = 12.dp, enabled = index < steps.lastIndex, modifier = Modifier.height(22.dp))
                    }
                    Text(stepNumber(index), style = Ds.label, color = Ds.text3)
                    Text(step.label, style = Ds.ui, color = Ds.text, modifier = Modifier.weight(1f))
                    Row(
                        Modifier.height(26.dp).clip(DsShapes.sm).dsOutline(Ds.borderStrong).clickable(role = Role.DropdownList) { onAssign?.invoke(index) }
                            .padding(start = 3.dp, end = 7.dp),
                        horizontalArrangement = Arrangement.spacedBy(6.dp),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Mascot(step.owner.agent, 20.dp)
                        Text(step.owner.name, style = Ds.caption, color = Ds.text2)
                        DsIcon(Lucide.ChevronDown, 12.dp, Ds.text3)
                    }
                    DsBareIcon(Lucide.X, "Remove step ${index + 1}", { onRemove?.invoke(index) })
                }
            }
            Row(
                Modifier.fillMaxWidth().heightIn(min = MinTouch).clip(DsShapes.xs).clickable(role = Role.Button) { onAdd?.invoke() },
                horizontalArrangement = Arrangement.spacedBy(8.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                DsIcon(Lucide.Plus, 14.dp, Ds.text3)
                Text("Add a step", style = Ds.ui, color = Ds.text3)
            }
        }
    }
}

/** The turn's work pinned above the composer's segmented progress: done, the one on now, still to come. */
@Composable
fun SegmentedProgress(total: Int, done: Int, modifier: Modifier = Modifier, now: Int? = null) {
    Row(modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(4.dp)) {
        repeat(total) { index ->
            val color = when {
                index < done -> Ds.accent
                index == now -> Ds.accent.copy(alpha = NOW_ALPHA)
                else -> Ds.border
            }
            val fill by androidx.compose.animation.animateColorAsState(color, DsMotion.effects(), label = "segment")
            androidx.compose.foundation.layout.Box(Modifier.weight(1f).height(4.dp).clip(DsShapes.full).background(fill))
        }
    }
}

private const val NOW_ALPHA = 0.45f
