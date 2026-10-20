package ai.nomi.feature.chat

import ai.nomi.data.ActivityRepository
import ai.nomi.data.AgentActivity
import ai.nomi.data.ChatState
import ai.nomi.data.Message
import ai.nomi.designsystem.brand.brandStyle
import ai.nomi.designsystem.component.Composer
import ai.nomi.designsystem.component.NomiLoadingIndicator
import ai.nomi.designsystem.component.NomiSuggestionChip
import ai.nomi.designsystem.icon.NomiIcon
import ai.nomi.designsystem.icon.NomiIcons
import ai.nomi.designsystem.shape.AgentIdentity
import ai.nomi.designsystem.shape.AgentShape
import ai.nomi.designsystem.shape.NomiIdentity
import ai.nomi.designsystem.shape.agentIdentity
import ai.nomi.designsystem.theme.NomiTheme
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
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
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

private val Suggestions = listOf("Plan my week", "What did I spend on food?", "Remind me to call mom tomorrow")

/** What the backend's agent phases read as on screen (`phaseText` on the web). */
private fun phaseLabel(phase: String): String = when (phase) {
    "thinking" -> "thinking"
    "calling_tool" -> "using a tool"
    "writing_reply", "writing" -> "writing"
    "running_command" -> "running a command"
    "retrying" -> "trying again"
    else -> phase.replace('_', ' ')
}

/** [onBack] adds a back action for shells without a system back gesture (desktop); null hides it. */
@Composable
fun ChatScreen(viewModel: ChatViewModel, modifier: Modifier = Modifier, onBack: (() -> Unit)? = null) {
    val chat by viewModel.chat.collectAsStateWithLifecycle()
    val composer by viewModel.composer.collectAsStateWithLifecycle()
    val title by viewModel.title.collectAsStateWithLifecycle()
    val approvals by viewModel.approvals.collectAsStateWithLifecycle()
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
) {
    BoxWithConstraints(modifier.fillMaxSize().background(NomiTheme.extendedColors.ground).testTag(CHAT_SCREEN_TAG)) {
        val phone = maxWidth < PhoneBreakpoint
        Column(Modifier.fillMaxSize().safeDrawingPadding().imePadding()) {
            AppBar(title.ifBlank { UNTITLED }, chat?.messages?.size ?: NO_ITEMS, phone, onBack)
            Box(Modifier.weight(FILL).fillMaxWidth()) {
                when {
                    chat == null -> NomiLoadingIndicator(Modifier.align(Alignment.Center), contentDescription = "Loading messages")
                    chat.messages.isEmpty() && chat.draft == null && chat.activeAgent == null -> EmptyThread(onSuggestion, Modifier.align(Alignment.Center))
                    else -> Thread(chat, timeZone, approvals, onResolveApproval)
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
                    placeholder = PLACEHOLDER,
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

/** One row of the thread: a message, or the reply streaming in. */
private sealed interface ThreadItem {
    val key: String
    data class Stored(val message: Message) : ThreadItem { override val key get() = message.id }
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

@Composable
private fun Thread(
    chat: ChatState,
    timeZone: TimeZone,
    approvals: Map<String, ApprovalState>,
    onResolveApproval: (messageId: String, approve: Boolean, remember: Boolean) -> Unit,
) {
    val items = chat.messages.map { ThreadItem.Stored(it) } +
        listOfNotNull(chat.draft?.let { ThreadItem.Streaming(it.turnJobId, it.text, chat.activeAgent?.agentType) })
    val list = rememberLazyListState()
    val working = chat.activeAgent != null || chat.draft != null
    val count = items.size + if (working) LAST_OFFSET else NO_ITEMS
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
                    item.isUser() -> UserTurn((item as ThreadItem.Stored).message, if (lastInChain) clock(item.message.createdAt, timeZone) else null)
                    item is ThreadItem.Stored -> {
                        val message = item.message
                        AssistantTurn(item.author(), chained, if (lastInChain) clock(message.createdAt, timeZone) else null, message.agentType, messageContent = message.content) { identity ->
                            val blocks = remember(message.contentJson) { parseContentBlocks(message.contentJson) }
                            if (blocks.isEmpty()) {
                                MessageText(message.content)
                            } else {
                                MessageBlocks(blocks, item.author(), identity, message.content, approvals[message.id], { approve, remember ->
                                    onResolveApproval(message.id, approve, remember)
                                })
                            }
                        }
                    }
                    item is ThreadItem.Streaming -> AssistantTurn(item.author(), chained, time = null, agentType = item.agentType, streaming = true) {
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
}

private fun clock(epochMs: Long, timeZone: TimeZone): String {
    val local = Instant.fromEpochMilliseconds(epochMs).toLocalDateTime(timeZone)
    return "${local.hour}:${local.minute.toString().padStart(MINUTES_PAD, '0')}"
}

/** `.message-bubble--user`: a primary bubble with the tight corner at the bottom end, up to 78% wide. */
@Composable
private fun UserTurn(message: Message, time: String?) {
    val clipboard = LocalClipboardManager.current
    var copied by remember { mutableStateOf(false) }
    LaunchedEffect(copied) {
        if (copied) {
            delay(COPIED_RESET_MS)
            copied = false
        }
    }
    BoxWithConstraints(Modifier.fillMaxWidth()) {
        val maxBubble: Dp = maxWidth * UserBubbleShare
        Column(Modifier.align(Alignment.CenterEnd).widthIn(max = maxBubble), horizontalAlignment = Alignment.End) {
            Text(
                message.content,
                style = MaterialTheme.typography.bodyLarge.copy(fontSize = USER_SP.sp, lineHeight = (USER_SP * USER_LINE_HEIGHT).sp),
                color = MaterialTheme.colorScheme.onPrimary,
                modifier = Modifier
                    .background(MaterialTheme.colorScheme.primary, NomiTheme.bubbleShapes.end)
                    .padding(horizontal = UserBubblePaddingH, vertical = UserBubblePaddingV),
            )
            Row(
                Modifier.padding(top = ChainGap),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(ActionGap),
            ) {
                time?.let { Meta(it) }
                IconButton(
                    onClick = {
                        clipboard.setText(AnnotatedString(message.content))
                        copied = true
                    },
                    modifier = Modifier.size(ActionIconSize),
                ) {
                    NomiIcon(
                        glyph = if (copied) NomiIcons.Check else NomiIcons.Copy,
                        contentDescription = if (copied) "Copied" else "Copy message",
                        size = ActionGlyphSize,
                        tint = if (copied) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }
        }
    }
}

/** Assistant turns are editorial: the agent's shape as avatar, its name, then the [body] (markdown or blocks), no bubble. */
@Composable
private fun AssistantTurn(
    author: String?,
    chained: Boolean,
    time: String?,
    agentType: String?,
    messageContent: String? = null,
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
            Row(
                Modifier.padding(top = ChainGap),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(ActionGap),
            ) {
                time?.let { Meta(it) }
                if (messageContent != null && !streaming) {
                    val clipboard = LocalClipboardManager.current
                    var copied by remember { mutableStateOf(false) }
                    LaunchedEffect(copied) {
                        if (copied) {
                            delay(COPIED_RESET_MS)
                            copied = false
                        }
                    }
                    IconButton(
                        onClick = {
                            clipboard.setText(AnnotatedString(messageContent))
                            copied = true
                        },
                        modifier = Modifier.size(ActionIconSize),
                    ) {
                        NomiIcon(
                            glyph = if (copied) NomiIcons.Check else NomiIcons.Copy,
                            contentDescription = if (copied) "Copied" else "Copy message",
                            size = ActionGlyphSize,
                            tint = if (copied) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                }
            }
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

/** `.thread__status`: the morphing loader and what the crew is doing. */
@Composable
private fun WorkingLine(agent: AgentActivity?) {
    val name = agent?.agentType?.let(::agentName) ?: FALLBACK_AGENT
    val activity = agent?.let { phaseLabel(it.phase) } ?: "working"
    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(MessageGap),
        modifier = Modifier.semantics(mergeDescendants = true) { liveRegion = LiveRegionMode.Polite },
    ) {
        NomiLoadingIndicator(size = LoaderSize, contentDescription = null)
        Text("$name is $activity…", style = MaterialTheme.typography.bodyLarge.copy(fontSize = USER_SP.sp), color = MaterialTheme.colorScheme.onSurfaceVariant)
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
