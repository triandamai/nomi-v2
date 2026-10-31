package ai.nomi.feature.chat

import ai.nomi.data.Message
import ai.nomi.designsystem.icon.NomiIcon
import ai.nomi.designsystem.icon.NomiIcons
import ai.nomi.designsystem.theme.NomiTheme
import ai.nomi.feature.chat.blocks.ContentBlock
import ai.nomi.feature.chat.blocks.parseContentBlocks
import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateMapOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.drawBehind
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.stateDescription
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.booleanOrNull
import kotlinx.serialization.json.contentOrNull

private const val ROLE_ASSISTANT = "assistant"
/** Older thinking steps were plain text starting with this, with no block (`$lib/reasoning`). */
private const val LEGACY_THOUGHT_PREFIX = "🧠 "
private const val TOOL_DELEGATE = "delegate_to_agent"

/** One thing the crew did on the way to a reply (`ReplyStep`): a tool, what it was used on, whether it worked. */
data class ReplyStep(val tool: String, val detail: String?, val ok: Boolean, val output: String?)

private val lenient = Json { ignoreUnknownKeys = true }

/** A message's steps (its `stepsJson`); unreadable JSON gives none. */
fun parseReplySteps(json: String?): List<ReplyStep> {
    if (json.isNullOrBlank()) return emptyList()
    val array = runCatching { lenient.parseToJsonElement(json) }.getOrNull() as? JsonArray ?: return emptyList()
    return array.mapNotNull { element ->
        val step = element as? JsonObject ?: return@mapNotNull null
        val tool = (step["tool"] as? JsonPrimitive)?.contentOrNull ?: return@mapNotNull null
        ReplyStep(
            tool = tool,
            detail = (step["detail"] as? JsonPrimitive)?.contentOrNull,
            ok = (step["ok"] as? JsonPrimitive)?.booleanOrNull ?: true,
            output = (step["output"] as? JsonPrimitive)?.contentOrNull,
        )
    }
}

/** The thinking text when [message] is a thinking step rather than a reply, else null. */
fun thoughtOf(message: Message): String? {
    if (message.role != ROLE_ASSISTANT) return null
    val first = parseContentBlocks(message.contentJson).firstOrNull()
    if (first is ContentBlock.Reasoning) return first.text
    if (message.contentJson.isNullOrBlank() && message.content.startsWith(LEGACY_THOUGHT_PREFIX)) {
        return message.content.removePrefix(LEGACY_THOUGHT_PREFIX)
    }
    return null
}

/**
 * A message as the thread shows it (`groupReasoning`): an agent's thinking steps fold into the reply they led to.
 * [thinkingOnly]: thinking whose reply hasn't landed (still working, or stopped), shown where it began.
 */
data class ThreadEntry(val message: Message, val thinking: List<String> = emptyList(), val thinkingOnly: Boolean = false)

fun groupThinking(messages: List<Message>): List<ThreadEntry> {
    val entries = mutableListOf<ThreadEntry>()
    // Thinking waiting for its reply, per agent (display name; null is Nomi): the steps and the first one.
    val pending = LinkedHashMap<String?, Pair<MutableList<String>, Message>>()
    for (message in messages) {
        val agent = message.agentDisplayName
        val thought = thoughtOf(message)
        if (thought != null) {
            pending.getOrPut(agent) { mutableListOf<String>() to message }.first += thought
            continue
        }
        val waiting = if (message.role == ROLE_ASSISTANT) pending.remove(agent) else null
        entries += ThreadEntry(message, waiting?.first.orEmpty())
    }
    for ((steps, first) in pending.values) {
        val at = entries.indexOfFirst { it.message.createdAt > first.createdAt }.takeIf { it >= 0 } ?: entries.size
        entries.add(at, ThreadEntry(first, steps, thinkingOnly = true))
    }
    return entries
}

/** What a step did, in the past tense (`stepLabel`): "Searched the web", "Handed it to Money". */
fun stepLabel(step: ReplyStep): String {
    if (step.tool == TOOL_DELEGATE && step.detail != null) return "Handed it to ${agentLabel(step.detail)}"
    return StepLabels[step.tool] ?: "Used ${step.tool.replace('_', ' ')}"
}

private val StepLabels = mapOf(
    "cancel_reminder" to "Cancelled a reminder",
    "create_project" to "Set up the project",
    "create_reminder" to "Set a reminder",
    TOOL_DELEGATE to "Handed it over",
    "delete_file" to "Deleted a file",
    "edit_file" to "Edited a file",
    "list_reminders" to "Checked reminders",
    "list_transactions" to "Checked transactions",
    "read_attachment" to "Read the attachment",
    "read_file" to "Read a file",
    "read_web_page" to "Read a page",
    "run_command" to "Ran",
    "search_files" to "Searched the code",
    "summarize_budget" to "Summed up the budget",
    "web_search" to "Searched the web",
    "write_file" to "Wrote a file",
    "write_files" to "Wrote files",
)

/** What a tool call is doing, fit to follow "is" (`toolActivityLabel`): "checking your transactions". */
fun toolActivity(tool: String): String = ToolActivities[tool] ?: "using ${tool.replace('_', ' ')}"

private val ToolActivities = mapOf(
    "complete_task" to "wrapping up",
    TOOL_DELEGATE to "delegating to another agent",
    "update_todos" to "updating the to-do list",
    "write_plan" to "writing the plan",
    "list_transactions" to "checking your transactions",
    "summarize_budget" to "summarizing your budget",
    "create_project" to "setting up the project",
    "read_file" to "reading a file",
    "write_file" to "writing a file",
    "delete_file" to "deleting a file",
    "list_files" to "listing files",
    "set_personality" to "updating its personality",
    "rollback_personality" to "rolling back its personality",
    "list_personality_versions" to "checking personality history",
    "list_recent_agent_activity" to "checking recent agent activity",
)

/** What the crew's phase reads as after "Nomi is" (`phaseActivity`). */
fun phaseActivity(phase: String, detail: String?): String = when (phase) {
    "thinking" -> "thinking"
    "writing_reply", "writing" -> "finalizing a reply"
    "calling_tool" -> detail?.let(::toolActivity) ?: "using a tool"
    "running_command" -> detail?.let { "running $it" } ?: "running a command"
    "retrying" -> "trying again"
    else -> "working"
}

/** An agent type as a name, for hand-offs (`agentTypeFallbackLabel`). */
private fun agentLabel(agentType: String): String = agentType.replaceFirstChar { it.uppercase() }

private const val LONG_THOUGHT_CHARS = 240
private const val LONG_THOUGHT_LINES = 3
private const val CLAMPED_LINES = 3
private const val CHEVRON_OPEN_DEG = 180f
private const val NO_ROTATION = 0f
private const val SPIN_MS = 900
private const val FULL_TURN = 360f
private const val TOGGLE_WEIGHT = 600
private val ToggleMinHeight = 36.dp
private val TogglePaddingStart = 10.dp
private val TogglePaddingEnd = 12.dp
private val ToggleGap = 6.dp
private val ToggleIcon = 16.dp
private val TimelineIndent = 18.dp
private val TimelineGap = 10.dp
private val DotSize = 7.dp
private val DotTop = 7.dp
private val ItemLineGap = 2.dp
private val RailWidth = 2.dp
private val OutputPadding = 10.dp
private val OutputMaxHeight = 220.dp

/**
 * `ActivityDisclosure.svelte`: how a reply came about, as one quiet pill ("Thought it through · 3 steps") that opens
 * in place into a timeline of the thinking and the steps. [live]: no reply yet, the agent may still be thinking.
 */
@Composable
fun ActivityDisclosure(thinking: List<String>, steps: List<ReplyStep>, live: Boolean, modifier: Modifier = Modifier) {
    if (thinking.isEmpty() && steps.isEmpty() && !live) return
    val colors = MaterialTheme.colorScheme
    var open by rememberSaveable(thinking.size, steps.size) { mutableStateOf(false) }
    val summary = listOfNotNull(
        when {
            live -> "Thinking…"
            thinking.isNotEmpty() -> "Thought it through"
            else -> null
        },
        when (steps.size) {
            0 -> null
            1 -> "1 step"
            else -> "${steps.size} steps"
        },
    )
    Column(modifier, verticalArrangement = Arrangement.spacedBy(ToggleGap)) {
        Row(
            Modifier
                .heightIn(min = ToggleMinHeight)
                .clip(if (open) MaterialTheme.shapes.medium else RoundedCornerShape(percent = 50))
                .background(colors.surfaceContainer)
                .clickable(onClickLabel = if (open) "Hide how it got there" else "Show how it got there") { open = !open }
                .semantics(mergeDescendants = true) { stateDescription = if (open) "expanded" else "collapsed" }
                .padding(start = TogglePaddingStart, end = TogglePaddingEnd),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(ToggleGap),
        ) {
            if (live) Spinner(colors.onSurfaceVariant) else NomiIcon(NomiIcons.Lightbulb, null, size = ToggleIcon, tint = colors.onSurfaceVariant)
            Text(
                summary.joinToString(" · "),
                style = MaterialTheme.typography.labelLarge.copy(fontWeight = FontWeight(TOGGLE_WEIGHT)),
                color = colors.onSurfaceVariant,
            )
            NomiIcon(
                NomiIcons.ChevronDown,
                null,
                size = ToggleIcon,
                tint = colors.onSurfaceVariant,
                modifier = Modifier.rotate(if (open) CHEVRON_OPEN_DEG else NO_ROTATION),
            )
        }
        if (open) Timeline(thinking, steps)
    }
}

@Composable
private fun Timeline(thinking: List<String>, steps: List<ReplyStep>) {
    val colors = MaterialTheme.colorScheme
    val rail = colors.outlineVariant
    val expanded = remember { mutableStateMapOf<Int, Boolean>() }
    val outputs = remember { mutableStateMapOf<Int, Boolean>() }
    Column(
        // The rail runs through the dots' centres.
        Modifier.drawBehind {
            val x = TimelineIndent.toPx() / 2
            drawLine(rail, Offset(x, DotTop.toPx()), Offset(x, size.height), RailWidth.toPx())
        },
        verticalArrangement = Arrangement.spacedBy(TimelineGap),
    ) {
        thinking.forEachIndexed { i, text ->
            val long = text.length > LONG_THOUGHT_CHARS || text.lines().size > LONG_THOUGHT_LINES
            val open = expanded[i] == true
            TimelineItem(colors.outline) {
                Text(
                    text,
                    style = MaterialTheme.typography.bodyMedium,
                    color = colors.onSurfaceVariant,
                    maxLines = if (long && !open) CLAMPED_LINES else Int.MAX_VALUE,
                    overflow = TextOverflow.Ellipsis,
                )
                if (long) MoreToggle(if (open) "Show less" else "Show more") { expanded[i] = !open }
            }
        }
        steps.forEachIndexed { i, step ->
            TimelineItem(if (step.ok) colors.primary else colors.error) {
                Text(stepLabel(step), style = MaterialTheme.typography.bodyMedium, color = colors.onSurface)
                if (step.detail != null && step.tool != TOOL_DELEGATE) {
                    Text(step.detail, style = NomiTheme.metaStyle, color = colors.onSurfaceVariant, maxLines = 2, overflow = TextOverflow.Ellipsis)
                }
                if (!step.ok) Text("didn't work", style = NomiTheme.metaStyle, color = colors.error)
                step.output?.let { output ->
                    val shown = outputs[i] == true
                    MoreToggle(if (shown) "Hide output" else "Show output") { outputs[i] = !shown }
                    if (shown) {
                        Text(
                            output,
                            style = NomiTheme.metaStyle,
                            color = colors.onSurface,
                            modifier = Modifier
                                .heightIn(max = OutputMaxHeight)
                                .clip(MaterialTheme.shapes.small)
                                .background(colors.surfaceContainerHigh)
                                .horizontalScroll(rememberScrollState())
                                .padding(OutputPadding),
                        )
                    }
                }
            }
        }
    }
}

@Composable
private fun TimelineItem(dot: Color, content: @Composable () -> Unit) {
    Row {
        Box(Modifier.width(TimelineIndent).padding(top = DotTop), contentAlignment = Alignment.TopCenter) {
            Box(Modifier.size(DotSize).background(dot, CircleShape))
        }
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(ItemLineGap)) { content() }
    }
}

@Composable
private fun MoreToggle(label: String, onClick: () -> Unit) {
    Text(
        label,
        style = MaterialTheme.typography.labelMedium,
        color = MaterialTheme.colorScheme.primary,
        modifier = Modifier.clickable(onClick = onClick).padding(vertical = ItemLineGap),
    )
}

/** A small turning arc, for "still at it". */
@Composable
internal fun Spinner(tint: Color, size: Dp = ToggleIcon) {
    val turn by rememberInfiniteTransition().animateFloat(
        initialValue = NO_ROTATION,
        targetValue = FULL_TURN,
        animationSpec = infiniteRepeatable(tween(SPIN_MS, easing = LinearEasing), RepeatMode.Restart),
    )
    NomiIcon(NomiIcons.Refresh, null, size = size, tint = tint, modifier = Modifier.rotate(turn))
}
