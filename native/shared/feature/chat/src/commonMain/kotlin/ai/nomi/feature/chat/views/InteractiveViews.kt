package ai.nomi.feature.chat.views

import ai.nomi.designsystem.component.NomiButton
import ai.nomi.designsystem.component.NomiButtonSize
import ai.nomi.designsystem.component.NomiButtonVariant
import ai.nomi.designsystem.component.NomiTextField
import ai.nomi.designsystem.component.nomiFocusRing
import ai.nomi.designsystem.icon.NomiIcon
import ai.nomi.designsystem.icon.NomiIcons
import ai.nomi.designsystem.shape.AgentShape
import ai.nomi.designsystem.shape.NomiIdentity
import ai.nomi.designsystem.shape.agentIdentity
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.Checkbox
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.selected
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.dp
import kotlinx.datetime.TimeZone

// The views that answer for the person (frontend/src/lib/components/views/*.svelte, blocks/QuestionsCard.svelte): each
// sends its answer as the person's message through [onSend]; once sent — or once the person has replied since
// ([answered]) — it stays as a record of what was chosen and can't be used again.

private const val DISABLED_ALPHA = 0.55f
private const val ENABLED_ALPHA = 1f
private val ChoiceMinHeight = 36.dp
private val ButtonMinHeight = 44.dp
private val HeadAvatar = 28.dp
private const val HIGHLIGHT_SP = 32f
private const val CARD_TITLE_SP = 17f
private const val ANSWERED = "Answered"
private const val TEXTAREA_ROWS = 3
private const val NOTE_ROWS = 2

private val CardFieldPaddingH = 12.dp
private val CardFieldPaddingV = 4.dp
private val CardFieldRow = 8.dp

/** One thing with its key facts, and buttons that answer for the person (each sends its message). */
@OptIn(ExperimentalLayoutApi::class)
@Composable
internal fun CardViewBlock(view: ChatView.Card, timeZone: TimeZone, answered: Boolean, onSend: ((String) -> Unit)?) {
    val colors = MaterialTheme.colorScheme
    val uri = LocalUriHandler.current
    var sent by rememberSaveable { mutableStateOf<String?>(null) }
    val done = answered || sent != null
    ViewFrame(null) {
        Column(verticalArrangement = Arrangement.spacedBy(InsetM)) {
            Column(verticalArrangement = Arrangement.spacedBy(GapS)) {
                view.eyebrow?.let { Text(it, style = smallStyle(), color = colors.onSurfaceVariant) }
                Text(view.title, style = MaterialTheme.typography.bodyLarge.copy(fontSize = CARD_TITLE_SP.sp(), fontWeight = TitleWeight), color = colors.onSurface)
                view.subtitle?.let { Text(it, style = detailStyle(), color = colors.onSurfaceVariant) }
            }
            view.highlight?.let { h ->
                Column(verticalArrangement = Arrangement.spacedBy(GapS)) {
                    Text(h.label, style = smallStyle(), color = colors.onSurfaceVariant)
                    Text(formatValue(h.value, h.format, timeZone), style = brandFigure(HIGHLIGHT_SP), color = toneInk(view.tone))
                }
            }
            if (view.fields.isNotEmpty()) {
                Column(Modifier.fillMaxWidth().clip(MaterialTheme.shapes.large).background(colors.surfaceContainerLow).padding(horizontal = CardFieldPaddingH, vertical = CardFieldPaddingV)) {
                    view.fields.forEachIndexed { i, field ->
                        if (i > 0) Rule()
                        Row(Modifier.fillMaxWidth().padding(vertical = CardFieldRow), horizontalArrangement = Arrangement.spacedBy(GapXl)) {
                            Text(field.label, style = rowStyle(), color = colors.onSurfaceVariant, modifier = Modifier.weight(FILL))
                            Text(formatValue(field.value, field.format, timeZone), style = rowStyle().copy(fontWeight = StrongWeight), color = colors.onSurface)
                        }
                    }
                }
            }
            view.body?.let { Text(it, style = MaterialTheme.typography.bodyLarge, color = colors.onSurface) }
            if (view.link != null || view.actions.isNotEmpty()) {
                FlowRow(horizontalArrangement = Arrangement.spacedBy(GapM), verticalArrangement = Arrangement.spacedBy(GapM)) {
                    view.link?.let { link ->
                        NomiButton(onClick = { uri.openUri(link.url) }, variant = NomiButtonVariant.Filled, size = NomiButtonSize.Small, modifier = Modifier.heightIn(min = ButtonMinHeight)) { Text(link.label) }
                    }
                    view.actions.forEach { action ->
                        val picked = sent == action.message
                        NomiButton(
                            onClick = { if (!done && onSend != null) { onSend(action.message); sent = action.message } },
                            variant = NomiButtonVariant.Tonal,
                            size = NomiButtonSize.Small,
                            enabled = !done && onSend != null || picked,
                            modifier = Modifier.heightIn(min = ButtonMinHeight),
                        ) { Text(action.label) }
                    }
                }
            }
        }
    }
}

private fun Float.sp() = androidx.compose.ui.unit.TextUnit(this, androidx.compose.ui.unit.TextUnitType.Sp)

/** A choice chip: outlined, filled with the secondary container (and a check) when picked. */
@Composable
private fun ChoiceChip(label: String, picked: Boolean, enabled: Boolean, onClick: () -> Unit, leadingArrow: Boolean = false) {
    val colors = MaterialTheme.colorScheme
    val shape = MaterialTheme.shapes.small
    Row(
        Modifier
            .heightIn(min = ChoiceMinHeight)
            .alpha(if (enabled || picked) ENABLED_ALPHA else DISABLED_ALPHA)
            .nomiFocusRing(shape)
            .clip(shape)
            .background(if (picked) colors.secondaryContainer else colors.surfaceContainerLowest)
            .border(Hairline, if (picked) colors.secondaryContainer else colors.outlineVariant, shape)
            .semantics { selected = picked }
            .clickable(enabled = enabled, role = Role.Button, onClick = onClick)
            .padding(start = GapMd, end = InsetM, top = GapSm, bottom = GapSm),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(GapSm),
    ) {
        when {
            picked -> NomiIcon(NomiIcons.Check, null, tint = colors.onSecondaryContainer, size = GapXl)
            leadingArrow -> NomiIcon(NomiIcons.ArrowForward, null, tint = colors.primary, size = GapXl)
        }
        Text(label, style = rowStyle(), color = if (picked) colors.onSecondaryContainer else colors.onSurface)
    }
}

/** Next things the person might ask, as chips: tapping one sends it. Once the conversation moved on, they go. */
@OptIn(ExperimentalLayoutApi::class)
@Composable
internal fun SuggestionsViewBlock(view: ChatView.Suggestions, answered: Boolean, onSend: ((String) -> Unit)?) {
    var sent by rememberSaveable { mutableStateOf<String?>(null) }
    if (answered && sent == null) return
    val done = answered || sent != null
    Column(verticalArrangement = Arrangement.spacedBy(GapM)) {
        view.title?.let { Text(it, style = detailStyle(), color = MaterialTheme.colorScheme.onSurfaceVariant) }
        FlowRow(horizontalArrangement = Arrangement.spacedBy(GapM), verticalArrangement = Arrangement.spacedBy(GapM)) {
            view.items.forEach { item ->
                ChoiceChip(item, picked = sent == item, enabled = !done && onSend != null, leadingArrow = true, onClick = {
                    onSend?.invoke(item)
                    sent = item
                })
            }
        }
    }
}

private val OptionMinWidth = 200.dp
private val OptionPaddingH = 14.dp

/** A choice between a few options with what speaks for and against each; picking one sends it. */
@Composable
internal fun DecisionViewBlock(view: ChatView.Decision, agent: String?, answered: Boolean, onSend: ((String) -> Unit)?) {
    val colors = MaterialTheme.colorScheme
    var picked by rememberSaveable { mutableStateOf<String?>(null) }
    val done = answered || picked != null
    ViewFrame(null) {
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(GapMd)) {
            val identity = agent?.let { agentIdentity(it) } ?: NomiIdentity
            AgentShape(identity, HeadAvatar, face = true)
            Text(view.question, style = rowStyle().copy(fontWeight = TitleWeight), color = colors.onSurface)
        }
        BoxWithConstraints(Modifier.fillMaxWidth()) {
            val columns = (maxWidth / OptionMinWidth).toInt().coerceIn(1, maxOf(1, view.options.size))
            val width = (maxWidth - GapM * (columns - 1)) / columns
            @OptIn(ExperimentalLayoutApi::class)
            FlowRow(horizontalArrangement = Arrangement.spacedBy(GapM), verticalArrangement = Arrangement.spacedBy(GapM)) {
                view.options.forEach { option ->
                    val chosen = picked == option.label
                    val shape = MaterialTheme.shapes.large
                    Column(
                        Modifier
                            .width(width)
                            .alpha(if (!done || chosen) ENABLED_ALPHA else DISABLED_ALPHA)
                            .nomiFocusRing(shape)
                            .clip(shape)
                            .background(if (chosen) colors.secondaryContainer else colors.surfaceContainerLow)
                            .border(Hairline, when { chosen -> colors.secondaryContainer; option.recommended -> colors.primary; else -> colors.outlineVariant }, shape)
                            .clickable(enabled = !done && onSend != null, role = Role.Button) {
                                onSend?.invoke(option.label)
                                picked = option.label
                            }
                            .padding(horizontal = OptionPaddingH, vertical = GapL),
                        verticalArrangement = Arrangement.spacedBy(GapSm),
                    ) {
                        val ink = if (chosen) colors.onSecondaryContainer else colors.onSurface
                        Row(horizontalArrangement = Arrangement.spacedBy(GapM), verticalAlignment = Alignment.CenterVertically) {
                            Text(option.label, style = rowStyle().copy(fontWeight = TitleWeight), color = ink)
                            if (option.recommended) RecommendedBadge()
                        }
                        option.detail?.let { Text(it, style = detailStyle(), color = if (chosen) ink else colors.onSurfaceVariant) }
                        option.pros.forEach { Point("+", colors.primary, it, ink) }
                        option.cons.forEach { Point("−", colors.error, it, ink) }
                    }
                }
            }
        }
        if (done) Text(ANSWERED, style = detailStyle(), color = colors.onSurfaceVariant)
    }
}

@Composable
private fun Point(mark: String, markColor: androidx.compose.ui.graphics.Color, text: String, ink: androidx.compose.ui.graphics.Color) {
    Row(horizontalArrangement = Arrangement.spacedBy(GapSm)) {
        Text(mark, style = detailStyle().copy(fontWeight = TitleWeight), color = markColor, modifier = Modifier.width(InsetM))
        Text(text, style = detailStyle(), color = ink)
    }
}

/** A few fields to fill in, sent as one message ("Label: value" lines under the form's title). */
@OptIn(ExperimentalLayoutApi::class)
@Composable
internal fun FormViewBlock(view: ChatView.Form, answered: Boolean, onSend: ((String) -> Unit)?) {
    val colors = MaterialTheme.colorScheme
    fun initial(f: ChatView.FormField) = when (val d = f.default) {
        is Scalar.Text -> d.value
        is Scalar.Num -> formatValue(d, null, TimeZone.UTC).replace(",", "")
        else -> ""
    }
    val values = remember(view) { mutableStateMapOf<String, String>().apply { view.fields.forEach { put(it.key, initial(it)) } } }
    val picks = remember(view) {
        mutableStateMapOf<String, List<String>>().apply {
            view.fields.forEach { f -> put(f.key, if (f.type == ChatView.FieldType.Select && f.default is Scalar.Text) listOf((f.default as Scalar.Text).value) else emptyList()) }
        }
    }
    val checks = remember(view) { mutableStateMapOf<String, Boolean>().apply { view.fields.forEach { put(it.key, it.default == Scalar.Bool(true)) } } }
    var sent by rememberSaveable { mutableStateOf(false) }
    val done = answered || sent
    fun answer(f: ChatView.FormField) = when (f.type) {
        ChatView.FieldType.Select, ChatView.FieldType.Multiselect -> picks[f.key].orEmpty().joinToString(", ")
        ChatView.FieldType.Checkbox -> if (checks[f.key] == true) "Yes" else ""
        else -> values[f.key].orEmpty()
    }
    val missing = view.fields.any { it.required && answer(it).isBlank() }
    val message = composeForm(view.title, view.fields.map { it.key to it.label }, view.fields.associate { it.key to answer(it) })

    ViewFrame(view.title) {
        view.intro?.let { Text(it, style = rowStyle(), color = colors.onSurfaceVariant) }
        view.fields.forEach { field ->
            val label = if (field.required) "${field.label} *" else field.label
            when (field.type) {
                ChatView.FieldType.Select, ChatView.FieldType.Multiselect -> Column(verticalArrangement = Arrangement.spacedBy(GapSm)) {
                    Text(label, style = MaterialTheme.typography.bodySmall, color = colors.onSurfaceVariant)
                    FlowRow(horizontalArrangement = Arrangement.spacedBy(GapM), verticalArrangement = Arrangement.spacedBy(GapM)) {
                        field.options.forEach { option ->
                            val on = picks[field.key].orEmpty().contains(option)
                            ChoiceChip(option, picked = on, enabled = !done, onClick = {
                                val current = picks[field.key].orEmpty()
                                picks[field.key] = when {
                                    on -> current - option
                                    field.type == ChatView.FieldType.Multiselect -> current + option
                                    else -> listOf(option)
                                }
                            })
                        }
                    }
                }
                ChatView.FieldType.Checkbox -> Row(verticalAlignment = Alignment.CenterVertically) {
                    Checkbox(checked = checks[field.key] == true, onCheckedChange = { checks[field.key] = it }, enabled = !done)
                    Text(label, style = rowStyle(), color = colors.onSurface)
                }
                else -> NomiTextField(
                    value = values[field.key].orEmpty(),
                    onValueChange = { values[field.key] = it },
                    label = label,
                    placeholder = field.placeholder ?: when (field.type) { ChatView.FieldType.Date -> "YYYY-MM-DD"; ChatView.FieldType.Time -> "HH:MM"; else -> null },
                    enabled = !done,
                    singleLine = field.type != ChatView.FieldType.Textarea,
                    minLines = TEXTAREA_ROWS,
                    keyboardOptions = KeyboardOptions(keyboardType = if (field.type == ChatView.FieldType.Number) KeyboardType.Decimal else KeyboardType.Text),
                    modifier = Modifier.fillMaxWidth(),
                )
            }
        }
        if (done) {
            Text(ANSWERED, style = detailStyle(), color = colors.onSurfaceVariant)
        } else {
            Box(Modifier.fillMaxWidth(), contentAlignment = Alignment.CenterEnd) {
                NomiButton(
                    onClick = { if (onSend != null && message.isNotEmpty()) { onSend(message); sent = true } },
                    variant = NomiButtonVariant.Filled,
                    size = NomiButtonSize.Small,
                    enabled = !missing && message.isNotEmpty() && onSend != null,
                    modifier = Modifier.heightIn(min = ButtonMinHeight),
                ) { Text(view.submitLabel ?: "Send") }
            }
        }
    }
}

/** `questions` block: a few questions with options to tap (some allow several), an optional note, one send. */
data class Questions(val intro: String?, val questions: List<Question>)
data class Question(val question: String, val options: List<Pair<String, String?>>, val multiple: Boolean)

@OptIn(ExperimentalLayoutApi::class)
@Composable
internal fun QuestionsBlock(block: Questions, agent: String?, answered: Boolean, onSend: ((String) -> Unit)?) {
    val colors = MaterialTheme.colorScheme
    val picked = remember(block) { mutableStateListOf<List<String>>().apply { repeat(block.questions.size) { add(emptyList()) } } }
    var note by rememberSaveable { mutableStateOf("") }
    var sent by rememberSaveable { mutableStateOf(false) }
    val done = answered || sent
    val message = composeAnswers(block.questions.map { it.question }, picked, note)
    ViewFrame(null) {
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(GapMd)) {
            val identity = agent?.let { agentIdentity(it) } ?: NomiIdentity
            AgentShape(identity, HeadAvatar, face = true)
            Text(block.intro ?: "A few questions first", style = rowStyle().copy(fontWeight = TitleWeight), color = colors.onSurface)
        }
        block.questions.forEachIndexed { q, question ->
            Column(verticalArrangement = Arrangement.spacedBy(GapSm)) {
                Text(question.question, style = rowStyle().copy(fontWeight = StrongWeight), color = colors.onSurface)
                if (question.multiple) Text("Pick any that apply", style = smallStyle(), color = colors.onSurfaceVariant)
                FlowRow(horizontalArrangement = Arrangement.spacedBy(GapM), verticalArrangement = Arrangement.spacedBy(GapM)) {
                    question.options.forEach { (label, _) ->
                        val on = picked[q].contains(label)
                        ChoiceChip(label, picked = on, enabled = !done, onClick = {
                            val current = picked[q]
                            picked[q] = when {
                                on -> current - label
                                question.multiple -> current + label
                                else -> listOf(label)
                            }
                        })
                    }
                }
            }
        }
        if (done) {
            Text(ANSWERED, style = detailStyle(), color = colors.onSurfaceVariant)
        } else {
            NomiTextField(value = note, onValueChange = { note = it }, label = "Anything else", placeholder = "Anything else, or your own answer…", singleLine = false, minLines = NOTE_ROWS, modifier = Modifier.fillMaxWidth())
            Box(Modifier.fillMaxWidth(), contentAlignment = Alignment.CenterEnd) {
                NomiButton(
                    onClick = { if (onSend != null && message.isNotEmpty()) { onSend(message); sent = true } },
                    variant = NomiButtonVariant.Filled,
                    size = NomiButtonSize.Small,
                    enabled = message.isNotEmpty() && onSend != null,
                    modifier = Modifier.heightIn(min = ButtonMinHeight),
                ) { Text("Send answers") }
            }
        }
    }
}

/** Draws a parsed view (`ViewBlock.svelte` + the registry). */
@Composable
fun ChatViewBlock(view: ChatView, agent: String?, answered: Boolean, onSend: ((String) -> Unit)?, timeZone: TimeZone) {
    when (view) {
        is ChatView.List -> ListViewBlock(view, timeZone)
        is ChatView.Table -> TableViewBlock(view.title, view.columns, view.rows, timeZone)
        is ChatView.Comparison -> ComparisonViewBlock(view, timeZone)
        is ChatView.Stepper -> StepperViewBlock(view)
        is ChatView.Timeline -> TimelineViewBlock(view, timeZone)
        is ChatView.Card -> CardViewBlock(view, timeZone, answered, onSend)
        is ChatView.Chart -> ChartViewBlock(view)
        is ChatView.Answer -> AnswerViewBlock(view, timeZone)
        is ChatView.Suggestions -> SuggestionsViewBlock(view, answered, onSend)
        is ChatView.Decision -> DecisionViewBlock(view, agent, answered, onSend)
        is ChatView.Form -> FormViewBlock(view, answered, onSend)
        is ChatView.LinkPreview -> LinkViewBlock(view)
        is ChatView.Task -> TaskViewBlock(view, agent)
        is ChatView.Shell -> ShellViewBlock(view)
    }
}
