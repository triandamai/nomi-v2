package ai.nomi.designsystem.ds.chat

import ai.nomi.designsystem.ds.CheckMark
import ai.nomi.designsystem.ds.Crew
import ai.nomi.designsystem.ds.Ds
import ai.nomi.designsystem.ds.DsBareIcon
import ai.nomi.designsystem.ds.DsButton
import ai.nomi.designsystem.ds.DsButtonVariant
import ai.nomi.designsystem.ds.DsCard
import ai.nomi.designsystem.ds.DsCardTitle
import ai.nomi.designsystem.ds.DsFold
import ai.nomi.designsystem.ds.DsIcon
import ai.nomi.designsystem.ds.DsMotion
import ai.nomi.designsystem.ds.DsShapes
import ai.nomi.designsystem.ds.DsTag
import ai.nomi.designsystem.ds.DsText
import ai.nomi.designsystem.ds.DsTextAction
import ai.nomi.designsystem.ds.Hairline
import ai.nomi.designsystem.ds.Mascot
import ai.nomi.designsystem.ds.MinTouch
import ai.nomi.designsystem.ds.RadioMark
import ai.nomi.designsystem.ds.SectionLabel
import ai.nomi.designsystem.ds.TagKind
import ai.nomi.designsystem.ds.dsOutline
import ai.nomi.designsystem.ds.DsBareField
import ai.nomi.designsystem.icon.Lucide
import ai.nomi.designsystem.icon.NomiGlyph
import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideOutHorizontally
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.selection.selectable
import androidx.compose.foundation.selection.toggleable
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp

enum class ApprovalStatus { Pending, Editing, Approved, Rejected }
data class KeyValue(val key: String, val value: String)

/**
 * Something the crew wants to do that changes the world, waiting for you: who and what, the fields and draft, and
 * approve (with "don't ask again"), decline or edit. Editing lets you rewrite the draft; after you act it shrinks to
 * what happened and how to undo it.
 */
@Composable
fun ApprovalCard(
    agent: Crew,
    title: String,
    modifier: Modifier = Modifier,
    status: ApprovalStatus = ApprovalStatus.Pending,
    fields: List<KeyValue> = emptyList(),
    body: String? = null,
    attachment: String? = null,
    rememberLabel: String? = null,
    approveLabel: String = "Approve",
    approveIcon: NomiGlyph = Lucide.Check,
    rejectLabel: String = "Decline",
    outcome: String? = null,
    actionLabel: String? = null,
    onAction: (() -> Unit)? = null,
    onApprove: ((remember: Boolean) -> Unit)? = null,
    onReject: (() -> Unit)? = null,
    onEdit: (() -> Unit)? = null,
    onSave: ((String) -> Unit)? = null,
    onCancel: (() -> Unit)? = null,
) {
    var rememberChoice by remember { mutableStateOf(false) }
    var draft by remember(body) { mutableStateOf(body ?: "") }
    val open = status == ApprovalStatus.Pending || status == ApprovalStatus.Editing
    DsCard(
        modifier,
        strong = open,
        header = {
            Mascot(agent, 20.dp)
            DsCardTitle(title, if (status == ApprovalStatus.Rejected) Ds.text2 else Ds.text)
            if (status == ApprovalStatus.Pending) DsTag(TagKind.Neutral, "Needs you")
            if (status == ApprovalStatus.Editing) DsTag(TagKind.Neutral, "Editing")
        },
        footer = when (status) {
            ApprovalStatus.Editing -> {
                {
                    Box(Modifier.weight(1f))
                    if (onCancel != null) DsButton("Cancel", onCancel, variant = DsButtonVariant.Ghost)
                    if (onSave != null) DsButton(approveLabel, { onSave(draft.trim()) }, iconEnd = approveIcon, enabled = draft.isNotBlank())
                }
            }
            ApprovalStatus.Pending -> {
                {
                    Box(Modifier.weight(1f)) {
                        if (rememberLabel != null) {
                            Row(
                                Modifier.heightIn(min = MinTouch).toggleable(rememberChoice, role = Role.Checkbox) { rememberChoice = it },
                                horizontalArrangement = Arrangement.spacedBy(8.dp),
                                verticalAlignment = Alignment.CenterVertically,
                            ) {
                                CheckMark(rememberChoice, null)
                                Text(rememberLabel, style = Ds.caption, color = Ds.text3)
                            }
                        }
                    }
                    if (onEdit != null) DsButton("Edit", onEdit, variant = DsButtonVariant.Ghost, iconStart = Lucide.Pencil)
                    if (onReject != null) DsButton(rejectLabel, onReject, variant = DsButtonVariant.Secondary)
                    if (onApprove != null) DsButton(approveLabel, { onApprove(rememberChoice) }, iconEnd = approveIcon)
                }
            }
            else -> null
        },
    ) {
        AnimatedContent(status, label = "approval-status") { now ->
            when (now) {
                ApprovalStatus.Editing -> Column(Modifier.fillMaxWidth().padding(horizontal = 14.dp, vertical = 12.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
                    KeyValues(fields)
                    BasicTextField(
                        draft, { draft = it },
                        Modifier.fillMaxWidth().clip(DsShapes.sm).background(Ds.surface).border(Hairline, Ds.accent, DsShapes.sm).padding(horizontal = 11.dp, vertical = 9.dp),
                        textStyle = Ds.caption.copy(color = Ds.text),
                        cursorBrush = SolidColor(Ds.accent),
                    )
                    Text("Your edits are sent as written. ${agent.displayName} won't rewrite them.", style = Ds.label, color = Ds.text3)
                }
                ApprovalStatus.Pending -> Column(Modifier.fillMaxWidth().padding(horizontal = 14.dp, vertical = 12.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
                    KeyValues(fields)
                    if (body != null) LeftRuled(Ds.borderStrong) { Text(body, style = Ds.caption, color = Ds.text2) }
                    if (attachment != null) {
                        Row(Modifier.padding(top = 4.dp), horizontalArrangement = Arrangement.spacedBy(6.dp), verticalAlignment = Alignment.CenterVertically) {
                            DsIcon(Lucide.Paperclip, 12.dp, Ds.text3)
                            Text(attachment, style = Ds.label, color = Ds.text3)
                        }
                    }
                }
                else -> Row(Modifier.fillMaxWidth().padding(horizontal = 14.dp, vertical = 10.dp), horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
                    DsIcon(if (now == ApprovalStatus.Approved) Lucide.Check else Lucide.X, 14.dp, if (now == ApprovalStatus.Approved) Ds.accent else Ds.text3)
                    Text(outcome ?: "", style = Ds.caption, color = Ds.text2, modifier = Modifier.weight(1f))
                    if (actionLabel != null && onAction != null) DsTextAction(actionLabel, onAction)
                }
            }
        }
    }
}

@Composable
private fun KeyValues(fields: List<KeyValue>) {
    if (fields.isEmpty()) return
    Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
        fields.forEach { field ->
            Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                Text(field.key, style = Ds.label, color = Ds.text3, modifier = Modifier.width(56.dp))
                Text(field.value, style = Ds.caption, color = Ds.text2, modifier = Modifier.weight(1f))
            }
        }
    }
}

enum class ChoiceControl { Start, End }

/**
 * One option in a question: its title, detail and tag ("Recommended"), its key number, and a radio or checkbox.
 * A chosen radio fills in the accent container. [content] sits under it (the "keep planning" field).
 */
@Composable
fun ChoiceOption(
    multiple: Boolean,
    checked: Boolean,
    title: String,
    onChange: (Boolean) -> Unit,
    modifier: Modifier = Modifier,
    detail: String? = null,
    tag: String? = null,
    key: Int? = null,
    control: ChoiceControl = ChoiceControl.End,
    content: (@Composable ColumnScope.() -> Unit)? = null,
) {
    val chosen = !multiple && checked
    val edge by animateColorAsState(if (chosen) Ds.accent else Ds.borderStrong, DsMotion.fast(), label = "choice-edge")
    val fill by animateColorAsState(if (chosen) Ds.accentBg else Color.Transparent, DsMotion.fast(), label = "choice-fill")
    val mark: @Composable () -> Unit = { if (multiple) CheckMark(checked, null) else RadioMark(checked) }
    Column(modifier.fillMaxWidth().clip(DsShapes.sm).background(fill).border(Hairline, edge, DsShapes.sm)) {
        Row(
            Modifier
                .fillMaxWidth()
                .heightIn(min = MinTouch)
                .then(
                    if (multiple) Modifier.toggleable(checked, role = Role.Checkbox, onValueChange = onChange)
                    else Modifier.selectable(checked, role = Role.RadioButton) { onChange(true) },
                )
                .padding(start = 11.dp, end = 11.dp, top = 9.dp, bottom = if (content == null) 9.dp else 0.dp),
            horizontalArrangement = Arrangement.spacedBy(12.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            if (control == ChoiceControl.Start) mark()
            if (key != null) KeyBadge(key)
            Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
                FlowRow(horizontalArrangement = Arrangement.spacedBy(8.dp), itemVerticalAlignment = Alignment.CenterVertically) {
                    Text(title, style = Ds.ui.copy(fontWeight = FontWeight.Medium), color = Ds.text)
                    if (tag != null) DsTag(TagKind.Go, tag)
                }
                if (detail != null) Text(detail, style = Ds.caption, color = Ds.text3)
            }
            if (control == ChoiceControl.End) mark()
        }
        if (content != null) Column(Modifier.padding(start = 11.dp, end = 11.dp, top = 8.dp, bottom = 9.dp), content = content)
    }
}

@Composable
private fun KeyBadge(key: Int) {
    Box(Modifier.size(20.dp).clip(DsShapes.xs).background(Ds.surface2), contentAlignment = Alignment.Center) {
        Text(key.toString(), style = Ds.label, color = Ds.text2)
    }
}

data class ClarifyOption(val title: String, val detail: String? = null, val recommended: Boolean = false)
data class ClarifyQuestion(
    val topic: String,
    val title: String,
    val options: List<ClarifyOption>,
    val helper: String? = null,
    val multiple: Boolean = false,
    val other: Boolean = false,
)

/** An answer: one choice, several, something written, or skipped (null). */
sealed interface ClarifyAnswer {
    data class Choice(val index: Int) : ClarifyAnswer
    data class Choices(val indices: List<Int>) : ClarifyAnswer
    data class Other(val text: String) : ClarifyAnswer
}

fun initialAnswer(question: ClarifyQuestion): ClarifyAnswer? = when {
    question.multiple -> ClarifyAnswer.Choices(emptyList())
    else -> question.options.indexOfFirst { it.recommended }.takeIf { it >= 0 }?.let { ClarifyAnswer.Choice(it) }
}

fun isAnswered(answer: ClarifyAnswer?): Boolean = when (answer) {
    null -> false
    is ClarifyAnswer.Choices -> answer.indices.isNotEmpty()
    is ClarifyAnswer.Other -> answer.text.isNotBlank()
    is ClarifyAnswer.Choice -> true
}

/**
 * A few questions before the crew starts, one at a time: topics along the top (checked once answered), the question
 * with its options, "something else", and skip / back / next. Questions slide in the way you're going.
 */
@Composable
fun ClarifyCard(
    agent: Crew,
    title: String,
    questions: List<ClarifyQuestion>,
    onFinish: (List<ClarifyAnswer?>) -> Unit,
    onSkipAll: () -> Unit,
    modifier: Modifier = Modifier,
    startStep: Int = 0,
    given: List<ClarifyAnswer?>? = null,
) {
    var step by remember { mutableIntStateOf(startStep) }
    val answers = remember { mutableStateListOf<ClarifyAnswer?>().apply { addAll(questions.mapIndexed { i, q -> given?.getOrNull(i) ?: initialAnswer(q) }) } }
    val visited = remember { mutableStateListOf<Boolean>().apply { addAll(questions.indices.map { it < startStep }) } }
    val question = questions[step]
    val answer = answers[step]
    val last = step == questions.lastIndex
    val keys = question.options.size + if (question.other && !question.multiple) 1 else 0
    fun go(index: Int) {
        visited[step] = true
        step = index
    }
    fun next() = if (last) onFinish(answers.toList()) else go(step + 1)
    DsCard(
        modifier,
        header = {
            Mascot(agent, 20.dp)
            DsCardTitle(title)
            AnimatedContent(step, label = "clarify-progress") { Text("${it + 1} of ${questions.size}", style = Ds.label, color = Ds.text3) }
        },
        footer = {
            val selected = (answer as? ClarifyAnswer.Choices)?.indices?.size ?: 0
            Text(
                if (question.multiple) "$selected selected · ↵ review" else "1–$keys to pick · ↵ next",
                style = Ds.label, color = Ds.text3, modifier = Modifier.weight(1f),
            )
            DsButton("Skip", { if (!question.multiple) answers[step] = null; next() }, variant = DsButtonVariant.Ghost)
            DsButton("Back", { go(step - 1) }, variant = DsButtonVariant.Secondary, enabled = step > 0)
            DsButton(if (last) "Review answers" else "Next", { next() }, iconEnd = Lucide.ArrowRight)
        },
    ) {
        FlowRow(Modifier.padding(start = 14.dp, end = 14.dp, top = 12.dp), horizontalArrangement = Arrangement.spacedBy(6.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
            questions.forEachIndexed { index, item ->
                val current = index == step
                val done = !current && visited[index] && isAnswered(answers[index])
                val bg by animateColorAsState(if (current) Ds.surface2 else Color.Transparent, DsMotion.fast(), label = "topic-bg")
                Row(
                    Modifier.height(28.dp).clip(DsShapes.full).background(bg).border(Hairline, if (current) Ds.borderStrong else Ds.border, DsShapes.full)
                        .clickable(role = Role.Tab) { go(index) }.padding(horizontal = 9.dp),
                    horizontalArrangement = Arrangement.spacedBy(6.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    DsIcon(if (done) Lucide.Check else Lucide.Circle, 12.dp, if (done) Ds.accent else Ds.text3)
                    Text(item.topic, style = Ds.caption, color = if (current) Ds.text else if (done) Ds.text2 else Ds.text3)
                }
            }
        }
        val slide = DsMotion.spatial<androidx.compose.ui.unit.IntOffset>()
        val fadeInSpec = DsMotion.effects<Float>()
        val fadeOutSpec = DsMotion.fast<Float>()
        AnimatedContent(
            step,
            transitionSpec = {
                val forward = targetState > initialState
                (slideInHorizontally(slide) { if (forward) it / SLIDE_FRACTION else -it / SLIDE_FRACTION } + fadeIn(fadeInSpec))
                    .togetherWith(slideOutHorizontally(slide) { if (forward) -it / SLIDE_FRACTION else it / SLIDE_FRACTION } + fadeOut(fadeOutSpec))
            },
            label = "clarify-question",
        ) { at ->
            val q = questions[at]
            val a = answers[at]
            Column {
                Column(Modifier.padding(start = 14.dp, end = 14.dp, top = 14.dp, bottom = 6.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                    Text(q.title, style = Ds.body.copy(fontWeight = FontWeight.Medium), color = Ds.text)
                    if (q.helper != null) Text(q.helper, style = Ds.caption, color = Ds.text3)
                }
                Column(Modifier.padding(start = 14.dp, end = 14.dp, top = 8.dp, bottom = 14.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
                    q.options.forEachIndexed { index, option ->
                        val checked = if (q.multiple) (a as? ClarifyAnswer.Choices)?.indices?.contains(index) == true else (a as? ClarifyAnswer.Choice)?.index == index
                        ChoiceOption(
                            multiple = q.multiple, checked = checked, title = option.title, detail = option.detail,
                            tag = if (option.recommended) "Recommended" else null, key = index + 1,
                            onChange = { on ->
                                answers[at] = if (q.multiple) {
                                    val current = (a as? ClarifyAnswer.Choices)?.indices ?: emptyList()
                                    ClarifyAnswer.Choices(if (on) (current + index).distinct().sorted() else current - index)
                                } else {
                                    ClarifyAnswer.Choice(index)
                                }
                            },
                        )
                    }
                    if (q.other && !q.multiple) {
                        Row(
                            Modifier.fillMaxWidth().clip(DsShapes.sm).border(Hairline, Ds.borderStrong, DsShapes.sm).padding(start = 11.dp, end = 5.dp, top = 5.dp, bottom = 5.dp),
                            horizontalArrangement = Arrangement.spacedBy(12.dp),
                            verticalAlignment = Alignment.CenterVertically,
                        ) {
                            KeyBadge(q.options.size + 1)
                            DsBareField((a as? ClarifyAnswer.Other)?.text ?: "", { answers[at] = ClarifyAnswer.Other(it) }, "Something else…", Ds.ui, Modifier.weight(1f).height(30.dp))
                            DsIcon(Lucide.CornerDownLeft, 14.dp, Ds.text3, Modifier.padding(end = 6.dp))
                        }
                    }
                }
            }
        }
    }
}

private const val SLIDE_FRACTION = 6

data class ReviewAnswer(val topic: String, val value: String, val isDefault: Boolean = false)

/** The answers before the plan is made: each topic and its answer (a pencil to change it), and why defaults were picked. */
@Composable
fun ClarifyReview(answers: List<ReviewAnswer>, onEdit: (Int) -> Unit, onBack: () -> Unit, onConfirm: () -> Unit, modifier: Modifier = Modifier, assumption: String? = null) {
    DsCard(
        modifier,
        header = {
            DsIcon(Lucide.ListChecks, 14.dp, Ds.text2)
            DsCardTitle("Your answers")
            Text("${answers.size} of ${answers.size}", style = Ds.label, color = Ds.text3)
        },
        footer = {
            Text("Nothing runs yet. You’ll see the plan first.", style = Ds.caption, color = Ds.text3, modifier = Modifier.weight(1f))
            DsButton("Back", onBack, variant = DsButtonVariant.Secondary)
            DsButton("Make the plan", onConfirm, iconEnd = Lucide.Sparkles)
        },
    ) {
        Column(Modifier.padding(horizontal = 14.dp, vertical = 4.dp)) {
            answers.forEachIndexed { index, answer ->
                if (index > 0) HorizontalDivider(thickness = Hairline, color = Ds.border)
                Row(Modifier.fillMaxWidth().padding(vertical = 4.dp), horizontalArrangement = Arrangement.spacedBy(12.dp), verticalAlignment = Alignment.CenterVertically) {
                    Text(answer.topic, style = Ds.caption, color = Ds.text3, modifier = Modifier.width(110.dp))
                    Text(answer.value, style = Ds.ui, color = Ds.text, modifier = Modifier.weight(1f))
                    if (answer.isDefault) DsTag(TagKind.Neutral, "Default")
                    DsBareIcon(Lucide.Pencil, "Change ${answer.topic}", { onEdit(index) })
                }
            }
        }
        if (assumption != null) {
            Row(Modifier.padding(start = 14.dp, end = 14.dp, bottom = 12.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                DsIcon(Lucide.Info, 12.dp, Ds.text3, Modifier.padding(top = 2.dp))
                Text(assumption, style = Ds.caption, color = Ds.text3)
            }
        }
    }
}

/** The questions, answered and folded to one line; press to see them again. */
@Composable
fun ClarifyAnswered(count: Int, summary: String, onOpen: () -> Unit, modifier: Modifier = Modifier) {
    Row(
        modifier.fillMaxWidth().heightIn(min = MinTouch).clip(DsShapes.sm).border(Hairline, Ds.border, DsShapes.sm).clickable(role = Role.Button, onClick = onOpen)
            .padding(horizontal = 11.dp, vertical = 7.dp),
        horizontalArrangement = Arrangement.spacedBy(8.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        DsIcon(Lucide.Check, 14.dp, Ds.accent)
        Text(if (count == 1) "Answered 1 question" else "Answered $count questions", style = Ds.caption, color = Ds.text2)
        DsText(summary, Ds.label, Ds.text3, Modifier.weight(1f), singleLine = true)
        DsIcon(Lucide.ChevronRight, 12.dp, Ds.text3)
    }
}

data class DecisionOption(val title: String, val detail: String? = null, val suggested: Boolean = false)

/** One question with a few answers; the suggestion starts picked, and the button says what it'll use. */
@Composable
fun DecisionCard(question: String, options: List<DecisionOption>, onConfirm: (Int) -> Unit, modifier: Modifier = Modifier) {
    var chosen by remember { mutableIntStateOf(-1) }
    val value = if (chosen >= 0) chosen else maxOf(0, options.indexOfFirst { it.suggested })
    DsCard(
        modifier,
        header = {
            DsIcon(Lucide.Split, 14.dp, Ds.text2)
            DsCardTitle(question)
        },
        footer = {
            Text("Or type your own answer", style = Ds.caption, color = Ds.text3, modifier = Modifier.weight(1f))
            DsButton("Use ${options.getOrNull(value)?.title ?: ""}", { onConfirm(value) }, iconEnd = Lucide.Check)
        },
    ) {
        Column(Modifier.padding(14.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
            options.forEachIndexed { index, option ->
                ChoiceOption(
                    multiple = false, checked = index == value, title = option.title, detail = option.detail,
                    tag = if (option.suggested) "Suggested" else null, control = ChoiceControl.Start,
                    onChange = { if (it) chosen = index },
                )
            }
        }
    }
}

/** A short form the crew prefilled ("prefilled from chat"), with what happens after you save. */
@Composable
fun FormCard(
    agent: Crew,
    title: String,
    submitLabel: String,
    onSubmit: () -> Unit,
    modifier: Modifier = Modifier,
    meta: String? = null,
    note: String? = null,
    onCancel: (() -> Unit)? = null,
    content: @Composable ColumnScope.() -> Unit,
) {
    DsCard(
        modifier,
        header = {
            Mascot(agent, 20.dp)
            DsCardTitle(title)
            if (meta != null) Text(meta, style = Ds.label, color = Ds.text3)
        },
        footer = {
            DsText(note ?: "", Ds.caption, Ds.text3, Modifier.weight(1f))
            if (onCancel != null) DsButton("Cancel", onCancel, variant = DsButtonVariant.Ghost)
            DsButton(submitLabel, onSubmit, iconEnd = Lucide.Check)
        },
    ) { Column(Modifier.padding(14.dp), verticalArrangement = Arrangement.spacedBy(14.dp), content = content) }
}

data class RunOption(val title: String, val detail: String? = null, val recommended: Boolean = false)

/** How to run a plan, with "keep planning" last; picking it opens a field for what to change. */
@Composable
fun RunOptions(options: List<RunOption>, who: String, value: Int, onValue: (Int) -> Unit, changes: String, onChanges: (String) -> Unit, modifier: Modifier = Modifier) {
    val chosen = if (value >= 0) value else maxOf(0, options.indexOfFirst { it.recommended })
    val keep = chosen == options.size
    Column(modifier.fillMaxWidth(), verticalArrangement = Arrangement.spacedBy(6.dp)) {
        options.forEachIndexed { index, option ->
            ChoiceOption(
                multiple = false, checked = index == chosen, title = option.title, detail = option.detail,
                tag = if (option.recommended) "Recommended" else null, key = index + 1, onChange = { if (it) onValue(index) },
            )
        }
        ChoiceOption(multiple = false, checked = keep, title = "Keep planning", key = options.size + 1, onChange = { if (it) onValue(options.size) }) {
            DsFold(keep) {
                DsBareField(
                    changes, { onChanges(it); onValue(options.size) }, "Tell $who what to change…", Ds.ui,
                    Modifier.fillMaxWidth().clip(DsShapes.sm).background(Ds.surface).border(Hairline, Ds.borderStrong, DsShapes.sm).padding(horizontal = 10.dp, vertical = 8.dp),
                )
            }
        }
    }
}

/** Ready to run this plan? Pick how, or keep planning and say what to change. */
@Composable
fun PlanApproval(options: List<RunOption>, onRun: (choice: Int?, keepPlanning: String?) -> Unit, modifier: Modifier = Modifier, who: String = "Nomi") {
    var value by remember { mutableIntStateOf(-1) }
    var changes by rememberSaveable { mutableStateOf("") }
    val chosen = if (value >= 0) value else maxOf(0, options.indexOfFirst { it.recommended })
    val keep = chosen == options.size
    DsCard(
        modifier,
        header = {
            DsIcon(Lucide.Play, 14.dp, Ds.text2)
            DsCardTitle("Ready to run this plan?")
        },
        footer = {
            Text("1–${options.size + 1} to pick · ↵ confirm", style = Ds.label, color = Ds.text3, modifier = Modifier.weight(1f))
            AnimatedContent(keep, label = "plan-approval-action") { k ->
                DsButton(
                    if (k) "Send changes" else "Run plan",
                    { if (k) onRun(null, changes.trim()) else onRun(chosen, null) },
                    iconEnd = if (k) Lucide.Send else Lucide.Play,
                    enabled = !k || changes.isNotBlank(),
                )
            }
        },
    ) {
        RunOptions(options, who, value, { value = it }, changes, { changes = it }, Modifier.padding(14.dp))
    }
}

/** Short answers you can send with one press. */
@Composable
fun QuickReplies(replies: List<String>, onPick: (String) -> Unit, modifier: Modifier = Modifier) {
    FlowRow(modifier, horizontalArrangement = Arrangement.spacedBy(8.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
        replies.forEach { reply ->
            Box(
                Modifier.heightIn(min = 32.dp).clip(DsShapes.full).border(Hairline, Ds.borderStrong, DsShapes.full).clickable(role = Role.Button) { onPick(reply) }
                    .padding(horizontal = 12.dp, vertical = 6.dp),
                contentAlignment = Alignment.Center,
            ) { Text(reply, style = Ds.ui, color = Ds.text) }
        }
    }
}

/** What you might ask next, each a line you can press. */
@Composable
fun FollowUps(items: List<String>, onPick: (String) -> Unit, modifier: Modifier = Modifier) {
    Column(modifier.fillMaxWidth(), verticalArrangement = Arrangement.spacedBy(2.dp)) {
        SectionLabel("Follow up")
        items.forEach { item ->
            Row(
                Modifier.fillMaxWidth().heightIn(min = MinTouch).clip(DsShapes.xs).clickable(role = Role.Button) { onPick(item) },
                horizontalArrangement = Arrangement.spacedBy(10.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                DsIcon(Lucide.CornerDownRight, 14.dp, Ds.text3)
                Text(item, style = Ds.ui, color = Ds.text2)
            }
        }
    }
}

data class SelectionItem(val name: String, val detail: String? = null, val amount: String? = null)

/** Pick some of a list (subscriptions to track, files to keep): select all, a running count and total, then confirm. */
@Composable
fun SelectionCard(
    agent: Crew,
    question: String,
    items: List<SelectionItem>,
    confirmLabel: String,
    onConfirm: (List<Int>) -> Unit,
    modifier: Modifier = Modifier,
    initial: List<Boolean>? = null,
    total: String? = null,
    onChange: ((List<Boolean>) -> Unit)? = null,
    onSkip: (() -> Unit)? = null,
) {
    val picked = remember { mutableStateListOf<Boolean>().apply { addAll(items.indices.map { initial?.getOrNull(it) ?: false }) } }
    val count = picked.count { it }
    val all = items.isNotEmpty() && count == items.size
    fun set(next: List<Boolean>) {
        picked.clear(); picked.addAll(next); onChange?.invoke(next)
    }
    DsCard(
        modifier,
        header = {
            Mascot(agent, 20.dp)
            DsCardTitle(question)
            DsTextAction(if (all) "Clear" else "Select all", { set(items.map { !all }) })
        },
        footer = {
            AnimatedContent(count, Modifier.weight(1f), label = "selection-count") {
                Text("$it selected${if (total != null) " · $total" else ""}", style = Ds.label, color = Ds.text3)
            }
            if (onSkip != null) DsButton("Skip", onSkip, variant = DsButtonVariant.Ghost)
            DsButton(confirmLabel, { onConfirm(picked.indices.filter { picked[it] }) }, iconEnd = Lucide.Check, enabled = count > 0)
        },
    ) {
        items.forEachIndexed { index, item ->
            if (index > 0) HorizontalDivider(thickness = Hairline, color = Ds.border)
            val on = picked[index]
            Row(
                Modifier.fillMaxWidth().heightIn(min = MinTouch).toggleable(on, role = Role.Checkbox) { set(picked.mapIndexed { at, v -> if (at == index) it else v }) }
                    .padding(horizontal = 14.dp, vertical = 9.dp),
                horizontalArrangement = Arrangement.spacedBy(12.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                CheckMark(on, null)
                Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
                    Text(item.name, style = Ds.ui, color = if (on) Ds.text else Ds.text2)
                    if (item.detail != null) Text(item.detail, style = Ds.label, color = Ds.text3)
                }
                if (item.amount != null) Text(item.amount, style = Ds.mono, color = if (on) Ds.text else Ds.text3)
            }
        }
    }
}
