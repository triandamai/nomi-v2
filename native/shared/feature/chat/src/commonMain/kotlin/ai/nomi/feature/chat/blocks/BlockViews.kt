package ai.nomi.feature.chat.blocks

import ai.nomi.designsystem.brand.brandStyle
import ai.nomi.designsystem.component.NomiButton
import ai.nomi.designsystem.component.NomiButtonVariant
import ai.nomi.designsystem.component.NomiWavyProgress
import ai.nomi.designsystem.icon.NomiIcon
import ai.nomi.designsystem.icon.NomiIcons
import ai.nomi.designsystem.shape.AgentIdentity
import ai.nomi.designsystem.shape.AgentShape
import ai.nomi.designsystem.theme.NomiTheme
import ai.nomi.feature.chat.ApprovalState
import ai.nomi.feature.chat.markdown.NomiMarkdown
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.selection.toggleable
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Checkbox
import androidx.compose.material3.CheckboxDefaults
import androidx.compose.material3.LocalContentColor
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.lerp
import androidx.compose.ui.semantics.LiveRegionMode
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.clearAndSetSemantics
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.liveRegion
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.stateDescription
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

private const val FILL = 1f
private const val FALLBACK_AGENT = "Nomi"

/** The widest a card reads in the conversation column (`.checklist { max-width: 540px }`). */
private val CardMaxWidth = 540.dp
private val BlockGap = 8.dp
private val CardPaddingH = 18.dp
private val CardPaddingTop = 16.dp
private val CardPaddingBottom = 14.dp
private val CardGap = 12.dp
private val SmallGap = 6.dp
private val RowMinHeight = 38.dp
private val TouchTarget = 48.dp

private const val CARD_TITLE_SP = 18f
private const val CARD_TITLE_LINE_HEIGHT = 1.3f
private const val CARD_TITLE_WEIGHT = 650
private const val HEADING_WEIGHT = 700
private const val EMPHASIS_WEIGHT = 650
private const val ITEM_SP = 15f
private const val ITEM_LINE_HEIGHT = 1.35f
private const val QUIET_ALPHA = 0.8f

/**
 * A message's content blocks, in order (`ContentBlockView.svelte`). Any kind this build doesn't draw shows once, as
 * a quiet card carrying the message's own text ([fallbackText]). Views and questions that answer for the person send
 * through [onSend]; [answered] (the person replied since) settles them.
 */
@Composable
fun MessageBlocks(
    blocks: List<ContentBlock>,
    agent: String?,
    identity: AgentIdentity,
    fallbackText: String,
    approval: ApprovalState?,
    onResolveApproval: (approve: Boolean, remember: Boolean) -> Unit,
    modifier: Modifier = Modifier,
    answered: Boolean = false,
    onSend: ((String) -> Unit)? = null,
    timeZone: kotlinx.datetime.TimeZone = kotlinx.datetime.TimeZone.currentSystemDefault(),
) {
    val firstUnknown = blocks.indexOfFirst { it is ContentBlock.Unknown }
    Column(modifier.fillMaxWidth(), verticalArrangement = Arrangement.spacedBy(BlockGap)) {
        blocks.forEachIndexed { index, block ->
            when (block) {
                is ContentBlock.Approval -> ApprovalCard(block, approval ?: ApprovalState(), onResolveApproval)
                is ContentBlock.TodoList -> TodoListCard(block, agent, identity)
                is ContentBlock.Plan -> PlanCard(block, agent, identity)
                is ContentBlock.FileChange -> FileChangeCard(block, identity)
                is ContentBlock.Reasoning -> ReasoningDisclosure(block.text)
                is ContentBlock.View -> ai.nomi.feature.chat.views.ChatViewBlock(block.view, agent, answered, onSend, timeZone)
                is ContentBlock.Questions -> ai.nomi.feature.chat.views.QuestionsBlock(block.block, agent, answered, onSend)
                is ContentBlock.Unknown -> if (index == firstUnknown) FallbackCard(fallbackText)
            }
        }
    }
}

@Composable
private fun cardTitleStyle() =
    brandStyle(CARD_TITLE_SP.sp, (CARD_TITLE_SP * CARD_TITLE_LINE_HEIGHT).sp).copy(fontWeight = FontWeight(CARD_TITLE_WEIGHT))

private val ApprovalPaddingV = 16.dp
private val ApprovalPaddingH = 18.dp
private val ApprovalGap = 10.dp
private val ResultIconSize = 18.dp
private val StoppedMarkSize = 12.dp
private val StoppedMarkCorner = 3.dp

/**
 * `ApprovalCard.svelte`: ember (tertiary) while it needs the person's OK, so a pending decision is impossible to
 * scroll past; a neutral surface once decided.
 */
@Composable
private fun ApprovalCard(block: ContentBlock.Approval, state: ApprovalState, onResolve: (approve: Boolean, remember: Boolean) -> Unit) {
    val colors = MaterialTheme.colorScheme
    val pending = block.status == ApprovalStatus.Pending
    val container = if (pending) colors.tertiaryContainer else colors.surfaceContainerLowest
    val content = if (pending) colors.onTertiaryContainer else colors.onSurface
    var remember by rememberSaveable(block.id) { mutableStateOf(false) }
    CompositionLocalProvider(LocalContentColor provides content) {
        Column(
            Modifier
                .widthIn(max = CardMaxWidth)
                .fillMaxWidth()
                .background(container, NomiTheme.bubbleShapes.start)
                .padding(horizontal = ApprovalPaddingH, vertical = ApprovalPaddingV),
            verticalArrangement = Arrangement.spacedBy(ApprovalGap),
        ) {
            if (pending) Text("NEEDS YOUR OK", style = NomiTheme.metaStyle, color = content, modifier = Modifier.alpha(QUIET_ALPHA))
            Text(block.description, style = cardTitleStyle(), color = content, modifier = Modifier.semantics { heading() })
            Row(Modifier.alpha(QUIET_ALPHA), horizontalArrangement = Arrangement.spacedBy(SmallGap), verticalAlignment = Alignment.CenterVertically) {
                Text("Tool:", style = MaterialTheme.typography.bodySmall, color = content)
                Text(block.toolName, style = MaterialTheme.typography.bodySmall.copy(fontFamily = NomiTheme.metaStyle.fontFamily), color = content)
            }
            if (pending) {
                state.error?.let { Text(it, style = MaterialTheme.typography.bodySmall, color = colors.error, modifier = Modifier.semantics { liveRegion = LiveRegionMode.Polite }) }
                if (state.expired) {
                    Text(
                        "This request expired. Ask again and the agent will ask for your OK again.",
                        style = MaterialTheme.typography.bodySmall,
                        color = content,
                        modifier = Modifier.semantics { liveRegion = LiveRegionMode.Polite },
                    )
                } else {
                    val busy = state.sending || state.sent
                    Row(
                        Modifier
                            .heightIn(min = TouchTarget)
                            .toggleable(remember, enabled = !busy, role = Role.Checkbox) { remember = it },
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Checkbox(
                            checked = remember,
                            onCheckedChange = null,
                            enabled = !busy,
                            colors = CheckboxDefaults.colors(checkedColor = colors.tertiary, checkmarkColor = colors.onTertiary, uncheckedColor = content),
                        )
                        Text("Remember this decision for next time", style = MaterialTheme.typography.bodySmall, color = content, modifier = Modifier.padding(start = SmallGap))
                    }
                    Row(horizontalArrangement = Arrangement.spacedBy(BlockGap)) {
                        NomiButton(
                            onClick = { onResolve(true, remember) },
                            enabled = !busy,
                            colors = ButtonDefaults.buttonColors(containerColor = colors.tertiary, contentColor = colors.onTertiary),
                        ) { Text("Approve") }
                        NomiButton(
                            onClick = { onResolve(false, remember) },
                            variant = NomiButtonVariant.Outlined,
                            enabled = !busy,
                            colors = ButtonDefaults.outlinedButtonColors(contentColor = content),
                        ) { Text("Deny") }
                    }
                }
            } else {
                ApprovalResult(block.status, content)
            }
        }
    }
}

@Composable
private fun ApprovalResult(status: ApprovalStatus, color: Color) {
    Row(horizontalArrangement = Arrangement.spacedBy(BlockGap), verticalAlignment = Alignment.CenterVertically) {
        when (status) {
            ApprovalStatus.Approved -> NomiIcon(NomiIcons.Check, contentDescription = null, tint = color, size = ResultIconSize)
            ApprovalStatus.Denied -> NomiIcon(NomiIcons.Close, contentDescription = null, tint = color, size = ResultIconSize)
            else -> Box(Modifier.size(ResultIconSize), contentAlignment = Alignment.Center) {
                Box(Modifier.size(StoppedMarkSize).background(color, RoundedCornerShape(StoppedMarkCorner)))
            }
        }
        val label = when (status) {
            ApprovalStatus.Approved -> "Approved"
            ApprovalStatus.Denied -> "Denied"
            else -> "Stopped before you decided"
        }
        Text(label, style = MaterialTheme.typography.bodyLarge.copy(fontWeight = FontWeight(EMPHASIS_WEIGHT)), color = color)
    }
}

/** `.checklist`'s background: the agent's tone at 16% over the lowest surface. */
private const val TINT_SHARE = 0.16f
private val ChecklistAvatar = 26.dp
private val MarkSize = 22.dp
private val MarkCorner = 7.dp
private val MarkBorder = 2.dp
private val MarkIcon = 14.dp
private const val PENDING_MARK_ALPHA = 0.45f

private fun possessive(agent: String?) = "${agent ?: FALLBACK_AGENT}’s"

@Composable
private fun TodoListCard(block: ContentBlock.TodoList, agent: String?, identity: AgentIdentity) {
    ChecklistCard(identity, "${possessive(agent)} to-do", block.items)
}

private const val PLAN_TITLE_SP = 15f

/** `PlanBlock.svelte`'s bubble: the plan's title and version (its steps open on the web). */
@Composable
private fun PlanCard(block: ContentBlock.Plan, agent: String?, identity: AgentIdentity) {
    ChecklistCard(identity, "${possessive(agent)} draft", emptyList()) {
        Text(
            block.title,
            style = MaterialTheme.typography.bodyLarge.copy(fontSize = PLAN_TITLE_SP.sp, fontWeight = FontWeight(EMPHASIS_WEIGHT)),
            color = MaterialTheme.colorScheme.onSurface,
        )
        Text("VERSION ${block.version}", style = NomiTheme.metaStyle, color = MaterialTheme.colorScheme.onSurfaceVariant)
    }
}

/** `ChecklistBubble.svelte`: the posting agent's shape and name, a wavy progress line and the steps, in its tint. */
@Composable
private fun ChecklistCard(identity: AgentIdentity, heading: String, items: List<TodoItem>, extra: @Composable ColumnScope.() -> Unit = {}) {
    val colors = MaterialTheme.colorScheme
    val gradient = NomiTheme.gradients.all[identity.tone]
    val tint = gradient?.stops?.lastOrNull()?.second?.let(::Color) ?: colors.primaryContainer
    val done = items.count { it.status == TodoStatus.Done }
    val working = items.any { it.status == TodoStatus.InProgress }
    Column(
        Modifier
            .widthIn(max = CardMaxWidth)
            .fillMaxWidth()
            .background(lerp(colors.surfaceContainerLowest, tint, TINT_SHARE), NomiTheme.bubbleShapes.start)
            .padding(start = CardPaddingH, end = CardPaddingH, top = CardPaddingTop, bottom = CardPaddingBottom),
        verticalArrangement = Arrangement.spacedBy(CardGap),
    ) {
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(ApprovalGap)) {
            AgentShape(identity, ChecklistAvatar, working = working, face = true)
            Text(
                heading,
                style = cardTitleStyle().copy(fontWeight = FontWeight(HEADING_WEIGHT)),
                color = colors.onSurface,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.weight(FILL).semantics { heading() },
            )
            if (items.isNotEmpty()) Text("$done / ${items.size}", style = NomiTheme.metaStyle, color = colors.onSurface)
        }
        if (items.isNotEmpty()) {
            val share = done.toFloat() / items.size
            NomiWavyProgress(progress = { share }, contentDescription = "$done of ${items.size} done", modifier = Modifier.fillMaxWidth())
        }
        extra()
        if (items.isNotEmpty()) {
            Column {
                items.forEach { ChecklistRow(it) }
            }
        }
    }
}

@Composable
private fun ChecklistRow(item: TodoItem) {
    val colors = MaterialTheme.colorScheme
    val state = when (item.status) {
        TodoStatus.Done -> "done"
        TodoStatus.InProgress -> "current step"
        TodoStatus.Pending -> "not started"
    }
    Row(
        Modifier.fillMaxWidth().heightIn(min = RowMinHeight).semantics(mergeDescendants = true) { stateDescription = state },
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(CardGap),
    ) {
        val markShape = RoundedCornerShape(MarkCorner)
        val mark = when (item.status) {
            TodoStatus.Done -> Modifier.background(colors.primary, markShape)
            TodoStatus.InProgress -> Modifier.border(MarkBorder, colors.primary, markShape)
            TodoStatus.Pending -> Modifier.border(MarkBorder, colors.primary.copy(alpha = PENDING_MARK_ALPHA), markShape)
        }
        Box(Modifier.size(MarkSize).then(mark), contentAlignment = Alignment.Center) {
            if (item.status == TodoStatus.Done) NomiIcon(NomiIcons.Check, contentDescription = null, tint = colors.onPrimary, size = MarkIcon)
        }
        Text(
            item.text,
            style = MaterialTheme.typography.bodyLarge.copy(
                fontSize = ITEM_SP.sp,
                lineHeight = (ITEM_SP * ITEM_LINE_HEIGHT).sp,
                fontWeight = if (item.status == TodoStatus.InProgress) FontWeight(EMPHASIS_WEIGHT) else null,
            ),
            color = if (item.status == TodoStatus.Pending) colors.onSurfaceVariant else colors.onSurface,
            modifier = Modifier.weight(FILL),
        )
    }
}

private val FileAvatar = 20.dp
private val FileRowPaddingH = 12.dp
private val FilePaddingV = 6.dp
private val CodePadding = 12.dp
private const val CHEVRON_OPEN_DEG = 90f
private const val NO_ROTATION = 0f
private const val PREVIEW_MAX_LINES = 40
private const val BADGE_MAX_CHARS = 3
private const val NO_LINES = 0

/**
 * `FileChangeBlock.svelte`: a file the agent created, changed or deleted, collapsed to its name and line counts; it
 * opens onto the file (the new text, or what was deleted).
 */
@Composable
private fun FileChangeCard(block: ContentBlock.FileChange, identity: AgentIdentity) {
    val colors = MaterialTheme.colorScheme
    val mono = NomiTheme.metaStyle.fontFamily
    val body = block.content ?: block.previousContent
    val stats = remember(block) {
        if (block.action == FileAction.Deleted) diffStats(block.previousContent, null) else diffStats(block.previousContent, block.content)
    }
    var open by rememberSaveable(block.path) { mutableStateOf(false) }
    val name = block.path.substringAfterLast('/')
    val folder = block.path.substringBeforeLast('/', missingDelimiterValue = "")
    val action = when (block.action) {
        FileAction.Created -> "Created"
        FileAction.Updated -> "Updated"
        FileAction.Deleted -> "Deleted"
    }
    Column(
        Modifier
            .widthIn(max = CardMaxWidth)
            .fillMaxWidth()
            .background(colors.surfaceContainer, MaterialTheme.shapes.large),
    ) {
        val toggle = if (body != null) Modifier.clickable(onClickLabel = if (open) "Hide file" else "Show file") { open = !open } else Modifier
        Row(
            Modifier
                .fillMaxWidth()
                .heightIn(min = TouchTarget)
                .then(toggle)
                .padding(horizontal = FileRowPaddingH, vertical = FilePaddingV)
                .semantics(mergeDescendants = true) { if (body != null) stateDescription = if (open) "expanded" else "collapsed" },
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(BlockGap),
        ) {
            AgentShape(identity, FileAvatar)
            Text(
                block.path.substringAfterLast('.', missingDelimiterValue = "").take(BADGE_MAX_CHARS).uppercase(),
                style = NomiTheme.metaStyle,
                color = colors.primary,
                modifier = Modifier.clearAndSetSemantics { },
            )
            Column(Modifier.weight(FILL)) {
                Text(name, style = MaterialTheme.typography.bodyMedium.copy(fontFamily = mono, fontWeight = FontWeight(EMPHASIS_WEIGHT)), color = colors.onSurface, maxLines = 1, overflow = TextOverflow.Ellipsis)
                if (folder.isNotEmpty()) {
                    Text(folder, style = NomiTheme.metaStyle, color = colors.onSurfaceVariant, maxLines = 1, overflow = TextOverflow.Ellipsis)
                }
            }
            Text(action.uppercase(), style = NomiTheme.metaStyle, color = colors.onSurfaceVariant)
            if (stats.added > NO_LINES) Text("+${stats.added}", style = NomiTheme.metaStyle, color = colors.primary)
            if (stats.removed > NO_LINES) Text("−${stats.removed}", style = NomiTheme.metaStyle, color = colors.error)
            if (body != null) {
                NomiIcon(NomiIcons.ChevronRight, contentDescription = null, modifier = Modifier.rotate(if (open) CHEVRON_OPEN_DEG else NO_ROTATION))
            }
        }
        if (open && body != null) CodePreview(body, deleted = block.action == FileAction.Deleted)
    }
}

@Composable
private fun CodePreview(text: String, deleted: Boolean) {
    val colors = MaterialTheme.colorScheme
    val lines = text.removeSuffix("\n").split('\n')
    val shown = lines.take(PREVIEW_MAX_LINES).joinToString("\n")
    val hidden = lines.size - PREVIEW_MAX_LINES
    Column(
        Modifier
            .fillMaxWidth()
            .background(colors.surfaceContainerHighest, MaterialTheme.shapes.large)
            .padding(CodePadding),
        verticalArrangement = Arrangement.spacedBy(SmallGap),
    ) {
        Text(
            shown,
            style = MaterialTheme.typography.bodySmall.copy(fontFamily = NomiTheme.metaStyle.fontFamily),
            color = if (deleted) colors.onSurfaceVariant else colors.onSurface,
            softWrap = false,
            modifier = Modifier.horizontalScroll(rememberScrollState()),
        )
        if (hidden > NO_LINES) Text("+ $hidden more lines", style = NomiTheme.metaStyle, color = colors.onSurfaceVariant)
    }
}

/** `ReasoningDisclosure.svelte`: the model's thinking, collapsed to one quiet line. */
@Composable
private fun ReasoningDisclosure(text: String) {
    val colors = MaterialTheme.colorScheme
    var open by rememberSaveable(text) { mutableStateOf(false) }
    Column(Modifier.fillMaxWidth()) {
        Row(
            Modifier
                .heightIn(min = TouchTarget)
                .clickable(onClickLabel = if (open) "Hide thinking" else "Show thinking") { open = !open }
                .semantics(mergeDescendants = true) { stateDescription = if (open) "expanded" else "collapsed" },
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(SmallGap),
        ) {
            NomiIcon(NomiIcons.ChevronRight, contentDescription = null, tint = colors.onSurfaceVariant, modifier = Modifier.rotate(if (open) CHEVRON_OPEN_DEG else NO_ROTATION))
            Text("Thinking", style = MaterialTheme.typography.labelLarge, color = colors.onSurfaceVariant)
        }
        if (open) {
            NomiMarkdown(text, MaterialTheme.typography.bodyMedium, colors.onSurfaceVariant, Modifier.padding(start = CardPaddingH))
        }
    }
}

private val FallbackPadding = 14.dp

/** A block this build can't draw: the message's own text in a quiet card, and where to see the rest. */
@Composable
private fun FallbackCard(text: String) {
    val colors = MaterialTheme.colorScheme
    Column(
        Modifier
            .widthIn(max = CardMaxWidth)
            .fillMaxWidth()
            .background(colors.surfaceContainerLow, MaterialTheme.shapes.large)
            .padding(FallbackPadding),
        verticalArrangement = Arrangement.spacedBy(SmallGap),
    ) {
        if (text.isNotBlank()) NomiMarkdown(text, MaterialTheme.typography.bodyMedium, colors.onSurface)
        Text("SHOWN IN FULL ON THE WEB", style = NomiTheme.metaStyle, color = colors.onSurfaceVariant)
    }
}
