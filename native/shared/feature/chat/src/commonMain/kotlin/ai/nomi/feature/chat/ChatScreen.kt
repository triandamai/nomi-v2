package ai.nomi.feature.chat

import ai.nomi.data.ActivityRepository
import ai.nomi.data.AgentActivity
import ai.nomi.data.ChatState
import ai.nomi.data.MESSAGE_STATUS_FAILED
import ai.nomi.data.MESSAGE_STATUS_SENDING
import ai.nomi.data.Message
import ai.nomi.designsystem.brand.brandStyle
import ai.nomi.designsystem.component.Composer
import ai.nomi.designsystem.component.NomiButton
import ai.nomi.designsystem.component.NomiButtonSize
import ai.nomi.designsystem.component.NomiButtonVariant
import ai.nomi.designsystem.component.NomiLoadingIndicator
import ai.nomi.designsystem.component.NomiSuggestionChip
import ai.nomi.designsystem.icon.NomiIcon
import ai.nomi.designsystem.icon.NomiIcons
import ai.nomi.designsystem.shape.AgentIdentity
import ai.nomi.designsystem.shape.AgentShape
import ai.nomi.designsystem.shape.NomiIdentity
import ai.nomi.designsystem.shape.agentIdentity
import ai.nomi.designsystem.theme.NomiTheme
import ai.nomi.feature.chat.blocks.ContentBlock
import ai.nomi.feature.chat.blocks.MessageBlocks
import ai.nomi.feature.chat.blocks.parseContentBlocks
import ai.nomi.feature.chat.markdown.NomiMarkdown
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawingPadding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.text.AnnotatedString
import kotlinx.coroutines.delay
import androidx.compose.ui.semantics.LiveRegionMode
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.liveRegion
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.em
import androidx.compose.ui.unit.sp
import androidx.lifecycle.compose.LifecycleResumeEffect
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import kotlinx.datetime.TimeZone
import kotlinx.datetime.toLocalDateTime
import kotlin.time.Instant

const val CHAT_SCREEN_TAG = "chat-screen"

private const val ROLE_USER = "user"
private const val FILL = 1f
private const val NO_ITEMS = 0
private const val LAST_OFFSET = 1
private const val FALLBACK_AGENT = "Nomi"
private const val UNTITLED = "New chat"
private const val PLACEHOLDER = "Message Nomi"

/** `.thread__column` / `.composer`: the conversation reads in a 780px column. */
private val ColumnMaxWidth = 780.dp
private val PhoneBreakpoint = 600.dp
private val AvatarSize = 36.dp
private val MessageGap = 12.dp
private val ChainGap = 6.dp
private val TurnGap = 22.dp
private val UserBubbleShare = 0.78f
private val UserBubblePaddingH = 16.dp
private val UserBubblePaddingV = 14.dp
private val ThreadPaddingH = 16.dp
private val DockPaddingBottom = 14.dp
private const val SENDER_WEIGHT = 700
private const val TITLE_WEIGHT = 700
private const val TITLE_WIDE_SP = 24f
private const val TITLE_PHONE_SP = 19f
private const val TITLE_TRACKING_EM = -0.025f
private const val TITLE_LINE_HEIGHT = 1.15f
private const val BODY_SP = 16f
private const val BODY_LINE_HEIGHT = 1.6f
private const val USER_SP = 15f
private const val USER_LINE_HEIGHT = 1.5f
private const val MINUTES_PAD = 2
private val ActionIconSize = 24.dp
private val ActionGlyphSize = 14.dp
private val ActionGap = 6.dp
private const val COPIED_RESET_MS = 2000L
/** A message still waiting in the outbox reads as not there yet. */
private const val SENDING_ALPHA = 0.6f
private const val OPAQUE = 1f

private val Suggestions = listOf("Plan my week", "What did I spend on food?", "Remind me to call mom tomorrow")

/**
 * Hands text to the platform's share sheet; null where there is none, and a shared link goes to the clipboard
 * instead.
 */
val LocalShareText = staticCompositionLocalOf<((String) -> Unit)?> { null }

/** [onBack] adds a back action for shells without a system back gesture (desktop); null hides it. */
@Composable
fun ChatScreen(
    viewModel: ChatViewModel,
    modifier: Modifier = Modifier,
    onBack: (() -> Unit)? = null,
    /** Inside a project's workspace, whose own bar names it: no app bar, and the composer asks for project work. */
    embedded: Boolean = false,
    placeholder: String = PLACEHOLDER,
) {
    val chat by viewModel.chat.collectAsStateWithLifecycle()
    val composer by viewModel.composer.collectAsStateWithLifecycle()
    val title by viewModel.title.collectAsStateWithLifecycle()
    val approvals by viewModel.approvals.collectAsStateWithLifecycle()
    val feedback by viewModel.feedback.collectAsStateWithLifecycle()
    val shareText = LocalShareText.current
    val clipboard = LocalClipboardManager.current
    val replies = remember(viewModel, shareText) {
        ReplyHandlers(
            rate = viewModel::rate,
            sendDislike = viewModel::sendDislike,
            closeDislike = viewModel::closeDislike,
            delete = viewModel::deleteReply,
            regenerate = viewModel::regenerate,
            share = { messageId, done ->
                viewModel.shareReply(messageId) { link ->
                    if (link != null) {
                        if (shareText != null) shareText(link) else clipboard.setText(AnnotatedString(link))
                    }
                    done(link != null)
                }
            },
        )
    }
    LifecycleResumeEffect(viewModel) {
        viewModel.onVisible()
        onPauseOrDispose { viewModel.onHidden() }
    }
    ChatContent(
        title = title,
        chat = chat,
        composer = composer,
        onDraftChange = viewModel::onDraftChange,
        onSend = viewModel::send,
        onSuggestion = viewModel::useSuggestion,
        onBack = onBack,
        modifier = modifier,
        approvals = approvals,
        onResolveApproval = viewModel::resolveApproval,
        onSendAnswer = viewModel::sendAnswer,
        onRetrySend = viewModel::retrySend,
        onDiscardSend = viewModel::discardSend,
        showAppBar = !embedded,
        placeholder = placeholder,
        feedback = feedback,
        replies = replies,
    )
}

/** One chat (`ChatThread.svelte`): app bar, the conversation in a centred column, the floating composer. */
@Composable
fun ChatContent(
    title: String,
    chat: ChatState?,
    composer: ChatComposerState,
    onDraftChange: (String) -> Unit,
    onSend: () -> Unit,
    onSuggestion: (String) -> Unit,
    onBack: (() -> Unit)?,
    modifier: Modifier = Modifier,
    timeZone: TimeZone = TimeZone.currentSystemDefault(),
    approvals: Map<String, ApprovalState> = emptyMap(),
    onResolveApproval: (messageId: String, approve: Boolean, remember: Boolean) -> Unit = { _, _, _ -> },
    /** Sends what a view answered for the person (a suggestion, decision, form…); null leaves views read-only. */
    onSendAnswer: ((String) -> Unit)? = null,
    /** Sends a message that wasn't sent again, by its id. */
    onRetrySend: (String) -> Unit = {},
    /** Drops a message that wasn't sent, by its id. */
    onDiscardSend: (String) -> Unit = {},
    showAppBar: Boolean = true,
    placeholder: String = PLACEHOLDER,
    /** Each reply's feedback beyond its stored rating, by message id. */
    feedback: Map<String, ReplyFeedbackState> = emptyMap(),
    replies: ReplyHandlers = ReplyHandlers(),
) {
    BoxWithConstraints(modifier.fillMaxSize().background(NomiTheme.extendedColors.ground).testTag(CHAT_SCREEN_TAG)) {
        val phone = maxWidth < PhoneBreakpoint
        Column(Modifier.fillMaxSize().safeDrawingPadding().imePadding()) {
            if (showAppBar) AppBar(title.ifBlank { UNTITLED }, chat?.messages?.size ?: NO_ITEMS, phone, onBack)
            Box(Modifier.weight(FILL).fillMaxWidth()) {
                when {
                    chat == null -> NomiLoadingIndicator(Modifier.align(Alignment.Center), contentDescription = "Loading messages")
                    chat.messages.isEmpty() && chat.draft == null && chat.activeAgent == null -> EmptyThread(onSuggestion, Modifier.align(Alignment.Center))
                    else -> Thread(chat, timeZone, approvals, onResolveApproval, onSendAnswer, onRetrySend, onDiscardSend, feedback, replies)
                }
            }
            Column(
                Modifier.align(Alignment.CenterHorizontally).widthIn(max = ColumnMaxWidth).fillMaxWidth().padding(horizontal = ThreadPaddingH).padding(bottom = DockPaddingBottom),
                verticalArrangement = Arrangement.spacedBy(MessageGap),
            ) {
                chat?.lastError?.let { Notice("Something went wrong — try sending again.") }
                if (chat?.accessLost == true) Notice("This chat is no longer available.")
                composer.error?.let { Notice(it) }
                Composer(
                    value = composer.draft,
                    onValueChange = onDraftChange,
                    onSend = onSend,
                    placeholder = placeholder,
                    sendLabel = "Send",
                    sending = composer.sending,
                )
            }
        }
    }
}

private val AppBarPaddingH = 12.dp
private val AppBarPaddingTop = 12.dp
private val AppBarPaddingBottom = 10.dp
private val AppBarGap = 4.dp

@Composable
private fun AppBar(title: String, messageCount: Int, phone: Boolean, onBack: (() -> Unit)?) {
    Row(
        Modifier.fillMaxWidth().padding(start = AppBarPaddingH, end = AppBarPaddingH, top = AppBarPaddingTop, bottom = AppBarPaddingBottom),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(AppBarGap),
    ) {
        if (onBack != null) {
            IconButton(onClick = onBack) { NomiIcon(NomiIcons.ArrowBack, contentDescription = "Back") }
        }
        Column(Modifier.weight(FILL).padding(start = if (onBack == null) AppBarPaddingH else 0.dp)) {
            val size = if (phone) TITLE_PHONE_SP else TITLE_WIDE_SP
            Text(
                title,
                style = MaterialTheme.typography.displaySmall.copy(fontSize = size.sp, lineHeight = (size * TITLE_LINE_HEIGHT).sp, fontWeight = FontWeight(TITLE_WEIGHT), letterSpacing = TITLE_TRACKING_EM.em),
                color = MaterialTheme.colorScheme.onSurface,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
                modifier = Modifier.semantics { heading() },
            )
            if (messageCount > NO_ITEMS) {
                Text(
                    if (messageCount == 1) "1 MESSAGE" else "$messageCount MESSAGES",
                    style = NomiTheme.metaStyle,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    maxLines = 1,
                )
            }
        }
    }
}

/** One row of the thread: a message (with the thinking that led to it), or the reply streaming in. */
private sealed interface ThreadItem {
    val key: String
    data class Stored(val entry: ThreadEntry) : ThreadItem {
        val message get() = entry.message
        override val key get() = message.id
    }
    data class Streaming(val turnJobId: String, val text: String, val agentType: String?) : ThreadItem { override val key get() = turnJobId }
}

private fun ThreadItem.isUser() = this is ThreadItem.Stored && message.role == ROLE_USER
private fun ThreadItem.author(): String? = when (this) {
    is ThreadItem.Stored -> if (message.role == ROLE_USER) null else message.agentDisplayName ?: message.agentType ?: FALLBACK_AGENT
    is ThreadItem.Streaming -> agentType?.let(::agentName) ?: FALLBACK_AGENT
}

/** An agent type as a name while its reply streams (the stored message later carries the real display name). */
private fun agentName(agentType: String): String =
    if (agentType == ActivityRepository.UNKNOWN_AGENT_TYPE || agentType == "chitchat") FALLBACK_AGENT
    else agentType.replace('_', ' ').replaceFirstChar { it.uppercase() }

/** Blocks the crew posts on its own (next-step buttons, a failed run): no copy, share or rating for them. */
private val StatusOnlyKinds = setOf("suggestions", "run_failed")

/** What's being deleted, and how the dialog names it. */
private data class Deleting(val messageId: String, val name: String?)

@Composable
private fun Thread(
    chat: ChatState,
    timeZone: TimeZone,
    approvals: Map<String, ApprovalState>,
    onResolveApproval: (messageId: String, approve: Boolean, remember: Boolean) -> Unit,
    onSendAnswer: ((String) -> Unit)?,
    onRetrySend: (String) -> Unit,
    onDiscardSend: (String) -> Unit,
    feedback: Map<String, ReplyFeedbackState>,
    replies: ReplyHandlers,
) {
    val working = chat.activeAgent != null || chat.draft != null
    val entries = remember(chat.messages) { groupThinking(chat.messages) }
    // A message the person has replied after is settled: its questions, decisions and forms show as answered.
    val lastUserIndex = entries.indexOfLast { it.message.role == ROLE_USER }
    val items = entries.map { ThreadItem.Stored(it) } +
        listOfNotNull(chat.draft?.let { ThreadItem.Streaming(it.turnJobId, it.text, chat.activeAgent?.agentType) })
    val list = rememberLazyListState()
    val count = items.size + if (working) LAST_OFFSET else NO_ITEMS
    var deleting by remember { mutableStateOf<Deleting?>(null) }
    var regenerating by remember { mutableStateOf<String?>(null) }
    // Follow the conversation: new messages and streaming text keep the newest line in view.
    LaunchedEffect(count, chat.draft?.text?.length) {
        if (count > NO_ITEMS) list.scrollToItem(count - LAST_OFFSET)
    }
    LazyColumn(state = list, modifier = Modifier.fillMaxSize(), horizontalAlignment = Alignment.CenterHorizontally) {
        itemsIndexed(items, key = { _, item -> item.key }) { i, item ->
            val previous = items.getOrNull(i - 1)
            val next = items.getOrNull(i + 1)
            // Consecutive turns by the same author read as one: no repeated avatar or name.
            val chained = previous != null && previous.isUser() == item.isUser() && previous.author() == item.author()
            val lastInChain = next == null || next.isUser() != item.isUser() || next.author() != item.author()
            Box(Modifier.widthIn(max = ColumnMaxWidth).fillMaxWidth().padding(horizontal = ThreadPaddingH).padding(top = if (i == 0) TurnGap else if (chained) ChainGap else TurnGap)) {
                when {
                    item.isUser() -> {
                        val message = (item as ThreadItem.Stored).message
                        UserTurn(
                            message,
                            if (lastInChain) clock(message.createdAt, timeZone) else null,
                            onRetry = { onRetrySend(message.id) },
                            onDiscard = { onDiscardSend(message.id) },
                            onDelete = { deleting = Deleting(message.id, name = null) },
                        )
                    }
                    item is ThreadItem.Stored -> {
                        val entry = item.entry
                        val message = entry.message
                        val blocks = remember(message.contentJson) { parseContentBlocks(message.contentJson) }
                        val steps = remember(message.stepsJson) { parseReplySteps(message.stepsJson) }
                        val statusOnly = blocks.isNotEmpty() && blocks.all { it is ContentBlock.Unknown && it.kind in StatusOnlyKinds }
                        val latest = i > lastUserIndex && !working
                        AssistantTurn(item.author(), chained, agentType = message.agentType, streaming = entry.thinkingOnly && working) { identity ->
                            ActivityDisclosure(entry.thinking, steps, live = entry.thinkingOnly && working, Modifier.padding(bottom = ChainGap))
                            if (entry.thinkingOnly) return@AssistantTurn
                            if (blocks.isEmpty()) {
                                MessageText(message.content)
                            } else {
                                MessageBlocks(
                                    blocks, item.author(), identity, message.content, approvals[message.id],
                                    { approve, remember -> onResolveApproval(message.id, approve, remember) },
                                    answered = i < lastUserIndex,
                                    onSend = onSendAnswer,
                                    timeZone = timeZone,
                                )
                            }
                            if (statusOnly) return@AssistantTurn
                            val state = feedback[message.id] ?: ReplyFeedbackState()
                            ResponseActions(
                                text = message.content,
                                time = if (lastInChain) clock(message.createdAt, timeZone) else null,
                                feedback = message.myFeedback,
                                onRate = { replies.rate(message.id, it) },
                                onShare = { done -> replies.share(message.id, done) },
                                onRegenerate = if (latest) ({ regenerating = if (regenerating == message.id) null else message.id }) else null,
                                onDelete = { deleting = Deleting(message.id, item.author()) },
                                modifier = Modifier.padding(top = ChainGap),
                            )
                            if (regenerating == message.id) {
                                RegenerateChoices(
                                    onPick = { style ->
                                        regenerating = null
                                        replies.regenerate(message.id, style)
                                    },
                                    modifier = Modifier.padding(top = ChainGap),
                                )
                            }
                            when {
                                state.dislikeOpen -> DislikeForm(
                                    sending = state.sending,
                                    onSend = { replies.sendDislike(message.id, it) },
                                    onCancel = { replies.closeDislike(message.id) },
                                    modifier = Modifier.padding(top = ChainGap),
                                )
                                state.sentWithChat != null -> FeedbackSent(state.sentWithChat, Modifier.padding(top = ChainGap))
                            }
                        }
                    }
                    item is ThreadItem.Streaming -> AssistantTurn(item.author(), chained, agentType = item.agentType, streaming = true) {
                        MessageText(item.text, Modifier.semantics { liveRegion = LiveRegionMode.Polite })
                    }
                }
            }
        }
        if (working && chat.draft == null) {
            item(key = "working") {
                Box(Modifier.widthIn(max = ColumnMaxWidth).fillMaxWidth().padding(horizontal = ThreadPaddingH).padding(top = TurnGap)) {
                    WorkingLine(chat.activeAgent)
                }
            }
        }
        item(key = "end") { Box(Modifier.padding(bottom = TurnGap)) }
    }
    deleting?.let { target ->
        val dismiss = { deleting = null }
        if (target.name != null) {
            DeleteReplyDialog(target.name, onDelete = { replies.delete(target.messageId); dismiss() }, onDismiss = dismiss)
        } else {
            DeleteMessageDialog(onDelete = { replies.delete(target.messageId); dismiss() }, onDismiss = dismiss)
        }
    }
}

private fun clock(epochMs: Long, timeZone: TimeZone): String {
    val local = Instant.fromEpochMilliseconds(epochMs).toLocalDateTime(timeZone)
    return "${local.hour}:${local.minute.toString().padStart(MINUTES_PAD, '0')}"
}

/**
 * `.message-bubble--user`: a primary bubble with the tight corner at the bottom end, up to 78% wide, with copy and
 * delete under it. A message still in the outbox is dimmed and says it is sending; one that wasn't sent says so,
 * with [onRetry] and [onDiscard].
 */
@Composable
private fun UserTurn(message: Message, time: String?, onRetry: () -> Unit, onDiscard: () -> Unit, onDelete: () -> Unit) {
    val sending = message.status == MESSAGE_STATUS_SENDING
    val failed = message.status == MESSAGE_STATUS_FAILED
    val clipboard = LocalClipboardManager.current
    var copied by remember { mutableStateOf(false) }
    val split = remember(message.content) { splitAttachments(message.content) }
    LaunchedEffect(copied) {
        if (copied) {
            delay(COPIED_RESET_MS)
            copied = false
        }
    }
    BoxWithConstraints(Modifier.fillMaxWidth()) {
        val maxBubble: Dp = maxWidth * UserBubbleShare
        Column(Modifier.align(Alignment.CenterEnd).widthIn(max = maxBubble), horizontalAlignment = Alignment.End) {
            Column(
                Modifier.alpha(if (sending) SENDING_ALPHA else OPAQUE),
                horizontalAlignment = Alignment.End,
                verticalArrangement = Arrangement.spacedBy(ChainGap),
            ) {
                // Attached files show as previews and cards above what was typed, never as their tags.
                if (split.files.isNotEmpty()) MessageAttachments(split.files)
                if (split.text.isNotEmpty()) {
                    Text(
                        split.text,
                        style = MaterialTheme.typography.bodyLarge.copy(fontSize = USER_SP.sp, lineHeight = (USER_SP * USER_LINE_HEIGHT).sp),
                        color = MaterialTheme.colorScheme.onPrimary,
                        modifier = Modifier
                            .background(MaterialTheme.colorScheme.primary, NomiTheme.bubbleShapes.end)
                            .padding(horizontal = UserBubblePaddingH, vertical = UserBubblePaddingV),
                    )
                }
            }
            if (failed) {
                NotSent(onRetry, onDiscard)
                return@Column
            }
            Row(
                Modifier.padding(top = ChainGap),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(ActionGap),
            ) {
                if (sending) Meta("Sending…") else time?.let { Meta(it) }
                UserAction(if (copied) NomiIcons.Check else NomiIcons.Copy, if (copied) "Copied" else "Copy message", active = copied) {
                    clipboard.setText(AnnotatedString(split.text.ifEmpty { message.content }))
                    copied = true
                }
                if (!sending) UserAction(NomiIcons.Delete, "Delete message", onClick = onDelete)
            }
        }
    }
}

@Composable
private fun UserAction(glyph: ai.nomi.designsystem.icon.NomiGlyph, label: String, active: Boolean = false, onClick: () -> Unit) {
    IconButton(onClick = onClick, modifier = Modifier.size(ActionIconSize)) {
        NomiIcon(
            glyph = glyph,
            contentDescription = label,
            size = ActionGlyphSize,
            tint = if (active) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}

/** `MessageDeleteSheet.svelte`, short: the person's message goes with the crew's reply to it. */
@Composable
private fun DeleteMessageDialog(onDelete: () -> Unit, onDismiss: () -> Unit) {
    androidx.compose.material3.AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text("Delete this message?") },
        text = { Text("It's removed from the chat, and the crew's reply to it goes too. This can't be undone.") },
        confirmButton = {
            NomiButton(
                onClick = onDelete,
                colors = androidx.compose.material3.ButtonDefaults.buttonColors(
                    containerColor = MaterialTheme.colorScheme.error,
                    contentColor = MaterialTheme.colorScheme.onError,
                ),
            ) { Text("Delete") }
        },
        dismissButton = { NomiButton(onClick = onDismiss, variant = NomiButtonVariant.Text) { Text("Cancel") } },
    )
}

/** Under a message that wasn't sent: why it stands out, and what can be done about it. */
@Composable
private fun NotSent(onRetry: () -> Unit, onDiscard: () -> Unit) {
    Row(
        Modifier.padding(top = ChainGap),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(ActionGap),
    ) {
        Text(
            "Not sent",
            style = NomiTheme.metaStyle,
            color = MaterialTheme.colorScheme.error,
            modifier = Modifier.semantics { liveRegion = LiveRegionMode.Polite },
        )
        NomiButton(onClick = onRetry, variant = NomiButtonVariant.Text, size = NomiButtonSize.ExtraSmall) { Text("Retry") }
        NomiButton(onClick = onDiscard, variant = NomiButtonVariant.Text, size = NomiButtonSize.ExtraSmall) { Text("Discard") }
    }
}

/**
 * Assistant turns are editorial: the agent's shape as avatar, its name, then the [body] (how it got there, the
 * markdown or blocks, and its actions), no bubble.
 */
@Composable
private fun AssistantTurn(
    author: String?,
    chained: Boolean,
    agentType: String?,
    streaming: Boolean = false,
    body: @Composable (AgentIdentity) -> Unit,
) {
    val identity = agentIdentity(agentType ?: author ?: FALLBACK_AGENT)
    Row(horizontalArrangement = Arrangement.spacedBy(MessageGap)) {
        Box(Modifier.width(AvatarSize)) {
            if (!chained) AgentShape(identity, AvatarSize, working = streaming, face = identity == NomiIdentity)
        }
        Column(Modifier.weight(FILL)) {
            if (!chained) {
                Text(
                    author ?: FALLBACK_AGENT,
                    style = MaterialTheme.typography.labelLarge.copy(fontWeight = FontWeight(SENDER_WEIGHT), lineHeight = AvatarSize.value.sp),
                    color = MaterialTheme.colorScheme.onSurface,
                )
            }
            body(identity)
        }
    }
}

/** An assistant message's markdown at the thread's reading size. */
@Composable
private fun MessageText(text: String, modifier: Modifier = Modifier) {
    NomiMarkdown(
        text,
        style = MaterialTheme.typography.bodyLarge.copy(fontSize = BODY_SP.sp, lineHeight = (BODY_SP * BODY_LINE_HEIGHT).sp),
        color = MaterialTheme.colorScheme.onSurface,
        modifier = modifier,
    )
}

@Composable
private fun Meta(text: String, modifier: Modifier = Modifier) {
    Text(text, style = NomiTheme.metaStyle, color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = modifier)
}

private val LoaderSize = 28.dp
private val StatusPaddingStart = 4.dp

/** `.thread__status`: the morphing loader and what the crew is doing ("Nomi is checking your transactions…"). */
@Composable
private fun WorkingLine(agent: AgentActivity?) {
    val activity = agent?.let { phaseActivity(it.phase, it.detail) } ?: "working"
    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(MessageGap),
        modifier = Modifier.padding(start = StatusPaddingStart).semantics(mergeDescendants = true) { liveRegion = LiveRegionMode.Polite },
    ) {
        NomiLoadingIndicator(size = LoaderSize, contentDescription = "Nomi is working")
        Text("Nomi is $activity…", style = MaterialTheme.typography.bodyLarge.copy(fontSize = USER_SP.sp), color = MaterialTheme.colorScheme.onSurfaceVariant)
    }
}

private val EmptyAvatar = 72.dp
private val EmptyGap = 12.dp
private val EmptyLedeWidth = 360.dp
private const val EMPTY_TITLE_SP = 24f

/** `.thread__empty`: Nomi, an invitation, and a few things to ask. */
@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun EmptyThread(onSuggestion: (String) -> Unit, modifier: Modifier) {
    Column(modifier.padding(ThreadPaddingH), horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(EmptyGap)) {
        AgentShape(NomiIdentity, EmptyAvatar, face = true)
        Text("What's on your mind?", style = brandStyle(EMPTY_TITLE_SP.sp).copy(fontWeight = FontWeight(TITLE_WEIGHT)), color = MaterialTheme.colorScheme.onSurface)
        Text(
            "Ask about plans, money or reminders, or just think out loud.",
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            textAlign = TextAlign.Center,
            modifier = Modifier.widthIn(max = EmptyLedeWidth),
        )
        FlowRow(horizontalArrangement = Arrangement.spacedBy(ChainGap, Alignment.CenterHorizontally), verticalArrangement = Arrangement.spacedBy(ChainGap)) {
            Suggestions.forEach { NomiSuggestionChip(it, onClick = { onSuggestion(it) }) }
        }
    }
}

private val NoticePaddingH = 14.dp
private val NoticePaddingV = 12.dp

/** `.thread__notice`: an error in the error container, announced politely. */
@Composable
private fun Notice(text: String) {
    Text(
        text,
        style = MaterialTheme.typography.bodyMedium,
        color = MaterialTheme.colorScheme.onErrorContainer,
        modifier = Modifier
            .fillMaxWidth()
            .background(MaterialTheme.colorScheme.errorContainer, MaterialTheme.shapes.large)
            .padding(horizontal = NoticePaddingH, vertical = NoticePaddingV)
            .semantics { liveRegion = LiveRegionMode.Polite },
    )
}
