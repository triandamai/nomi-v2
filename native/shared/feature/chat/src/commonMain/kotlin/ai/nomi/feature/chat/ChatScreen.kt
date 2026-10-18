package ai.nomi.feature.chat

import ai.nomi.data.ActivityRepository
import ai.nomi.data.AgentActivity
import ai.nomi.data.ChatState
import ai.nomi.data.Message
import ai.nomi.designsystem.component.Composer
import ai.nomi.designsystem.component.NomiButton
import ai.nomi.designsystem.component.NomiButtonVariant
import ai.nomi.designsystem.component.NomiLoadingIndicator
import ai.nomi.designsystem.layout.NomiPage
import ai.nomi.designsystem.layout.NomiSpacing
import ai.nomi.designsystem.shape.AgentShape
import ai.nomi.designsystem.shape.agentIdentity
import ai.nomi.designsystem.theme.NomiTheme
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.semantics.LiveRegionMode
import androidx.compose.ui.semantics.liveRegion
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.LifecycleResumeEffect
import androidx.lifecycle.compose.collectAsStateWithLifecycle

const val CHAT_SCREEN_TAG = "chat-screen"

private const val ROLE_USER = "user"
private const val FILL = 1f
private const val NO_ITEMS = 0
private const val LAST_OFFSET = 1
private const val FALLBACK_AGENT = "Nomi"
private val BubbleMaxWidth = 560.dp
private val AgentShapeSize = 24.dp

/** What the backend's agent phases read as on screen. */
private fun phaseLabel(phase: String): String = when (phase) {
    "thinking" -> "thinking"
    "calling_tool" -> "using a tool"
    "writing" -> "writing"
    else -> phase.replace('_', ' ')
}

/** [onBack] adds a back action for shells without a system back gesture (desktop); null hides it. */
@Composable
fun ChatScreen(viewModel: ChatViewModel, modifier: Modifier = Modifier, onBack: (() -> Unit)? = null) {
    val chat by viewModel.chat.collectAsStateWithLifecycle()
    val composer by viewModel.composer.collectAsStateWithLifecycle()
    LifecycleResumeEffect(viewModel) {
        viewModel.onVisible()
        onPauseOrDispose { viewModel.onHidden() }
    }
    NomiPage(
        title = "Chat",
        modifier = modifier.testTag(CHAT_SCREEN_TAG),
        actions = {
            if (onBack != null) NomiButton(onClick = onBack, variant = NomiButtonVariant.Text) { Text("Back") }
        },
    ) {
        val state = chat
        Box(Modifier.weight(FILL).fillMaxWidth()) {
            if (state == null) {
                NomiLoadingIndicator(Modifier.align(Alignment.Center), contentDescription = "Loading messages")
            } else {
                Transcript(state)
            }
        }
        state?.lastError?.let { Text("The reply failed: $it", color = MaterialTheme.colorScheme.error) }
        if (state?.accessLost == true) {
            Text("This chat is no longer available.", color = MaterialTheme.colorScheme.error)
        }
        composer.error?.let { Text(it, color = MaterialTheme.colorScheme.error) }
        state?.activeAgent?.let { AgentLine(it) }
        Composer(
            value = composer.draft,
            onValueChange = viewModel::onDraftChange,
            onSend = viewModel::send,
            placeholder = "Message Nomi",
            sendLabel = "Send",
            sending = composer.sending,
            modifier = Modifier.padding(bottom = NomiSpacing.Large),
        )
    }
}

@Composable
private fun Transcript(state: ChatState) {
    val list = rememberLazyListState()
    val count = state.messages.size + if (state.draft != null) LAST_OFFSET else NO_ITEMS
    // Follow the conversation: new messages and streaming text keep the newest line in view.
    LaunchedEffect(count, state.draft?.text?.length) {
        if (count > NO_ITEMS) list.scrollToItem(count - LAST_OFFSET)
    }
    LazyColumn(state = list, verticalArrangement = Arrangement.spacedBy(NomiSpacing.Small)) {
        items(state.messages, key = { it.id }) { Bubble(it.content, fromUser = it.role == ROLE_USER, author = it.author()) }
        state.draft?.let { draft ->
            item(key = draft.turnJobId) { Bubble(draft.text, fromUser = false, author = null, streaming = true) }
        }
    }
}

private fun Message.author(): String? = if (role == ROLE_USER) null else agentDisplayName

@Composable
private fun Bubble(text: String, fromUser: Boolean, author: String?, streaming: Boolean = false) {
    val colors = MaterialTheme.colorScheme
    val shapes = NomiTheme.bubbleShapes
    Row(Modifier.fillMaxWidth(), horizontalArrangement = if (fromUser) Arrangement.End else Arrangement.Start) {
        Surface(
            shape = if (fromUser) shapes.end else shapes.start,
            color = if (fromUser) colors.primaryContainer else colors.surfaceContainerLowest,
            contentColor = if (fromUser) colors.onPrimaryContainer else colors.onSurface,
            modifier = Modifier
                .widthIn(max = BubbleMaxWidth)
                .then(if (streaming) Modifier.semantics { liveRegion = LiveRegionMode.Polite } else Modifier),
        ) {
            Column(
                Modifier.padding(horizontal = NomiSpacing.Medium, vertical = NomiSpacing.Small),
                verticalArrangement = Arrangement.spacedBy(NomiSpacing.ExtraSmall),
            ) {
                author?.let { Text(it, style = NomiTheme.metaStyle, color = colors.onSurfaceVariant) }
                Text(text, style = MaterialTheme.typography.bodyLarge)
            }
        }
    }
}

@Composable
private fun AgentLine(agent: AgentActivity) {
    val name = agent.agentType.takeUnless { it == ActivityRepository.UNKNOWN_AGENT_TYPE }?.replaceFirstChar { it.uppercase() } ?: FALLBACK_AGENT
    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(NomiSpacing.Small),
        modifier = Modifier.semantics { liveRegion = LiveRegionMode.Polite },
    ) {
        AgentShape(agentIdentity(agent.agentType), size = AgentShapeSize, working = true)
        Text(
            "$name is ${phaseLabel(agent.phase)}",
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}
