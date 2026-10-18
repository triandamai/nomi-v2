package ai.nomi.feature.chats

import ai.nomi.core.UNTITLED_CHAT
import ai.nomi.core.timeAgo
import ai.nomi.data.Session
import ai.nomi.designsystem.component.NomiListPanel
import ai.nomi.designsystem.component.NomiLoadingIndicator
import ai.nomi.designsystem.component.NomiRow
import ai.nomi.designsystem.icon.NomiIcon
import ai.nomi.designsystem.icon.NomiIcons
import ai.nomi.designsystem.layout.NomiPage
import ai.nomi.designsystem.shape.AgentShape
import ai.nomi.designsystem.shape.NomiIdentity
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import kotlin.time.Clock

const val CHATS_SCREEN_TAG = "chats-screen"

private val RefreshIndicatorSize = 24.dp
private val RowAvatar = 32.dp
private val ListBottomPadding = 20.dp
private const val NO_MESSAGES = "No messages yet"

/** [selectedId] highlights the chat open next to the list on wide windows. */
@Composable
fun ChatsScreen(
    viewModel: ChatsViewModel,
    onOpenChat: (sessionId: String) -> Unit,
    modifier: Modifier = Modifier,
    selectedId: String? = null,
) {
    val chats by viewModel.chats.collectAsStateWithLifecycle()
    val refreshing by viewModel.refreshing.collectAsStateWithLifecycle()
    ChatsContent(chats, refreshing, viewModel::refresh, onOpenChat, modifier, selectedId)
}

/** `chats/+page.svelte`: the display title, then the chats as rows in one panel. */
@Composable
fun ChatsContent(
    chats: List<Session>?,
    refreshing: Boolean,
    onRefresh: () -> Unit,
    onOpenChat: (String) -> Unit,
    modifier: Modifier = Modifier,
    selectedId: String? = null,
    nowMs: Long = Clock.System.now().toEpochMilliseconds(),
) {
    NomiPage(
        title = "Chats",
        modifier = modifier.testTag(CHATS_SCREEN_TAG),
        actions = {
            if (refreshing) {
                NomiLoadingIndicator(size = RefreshIndicatorSize, contentDescription = "Refreshing")
            } else {
                IconButton(onClick = onRefresh) { NomiIcon(NomiIcons.Refresh, contentDescription = "Refresh") }
            }
        },
    ) {
        when {
            chats == null -> NomiLoadingIndicator(Modifier.align(Alignment.CenterHorizontally), contentDescription = "Loading chats")
            chats.isEmpty() -> Text("No chats yet.", style = MaterialTheme.typography.bodyLarge, color = MaterialTheme.colorScheme.onSurfaceVariant)
            else -> NomiListPanel(Modifier.padding(bottom = ListBottomPadding)) {
                LazyColumn(Modifier.fillMaxWidth().clip(MaterialTheme.shapes.large)) {
                    items(chats, key = { it.id }) { session ->
                        NomiRow(
                            title = session.title.ifEmpty { UNTITLED_CHAT },
                            line = session.lastMessagePreview.ifBlank { NO_MESSAGES },
                            meta = timeAgo(session.updatedAt, nowMs),
                            emphasized = session.unread,
                            selected = session.id == selectedId,
                            onClick = { onOpenChat(session.id) },
                            leading = { AgentShape(NomiIdentity, RowAvatar) },
                        )
                    }
                }
            }
        }
    }
}
