package ai.nomi.feature.chats

import ai.nomi.designsystem.ds.Crew
import ai.nomi.designsystem.ds.DsMotion
import ai.nomi.designsystem.ds.DsSearch
import ai.nomi.designsystem.ds.EmptyState
import ai.nomi.designsystem.ds.SkeletonRow
import ai.nomi.designsystem.ds.mobile.ChatRow
import ai.nomi.designsystem.icon.Lucide

import ai.nomi.core.UNTITLED_CHAT
import ai.nomi.core.timeAgo
import ai.nomi.data.Session
import ai.nomi.designsystem.component.NomiListPanel
import ai.nomi.designsystem.component.NomiLoadingIndicator
import ai.nomi.designsystem.component.NomiRow
import ai.nomi.designsystem.component.NomiTextField
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
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import kotlin.time.Clock

const val CHATS_SCREEN_TAG = "chats-screen"

private const val SKELETON_ROWS = 3
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

/** `chats/+page.svelte`: the display title, search filter, then the chats as rows in one panel. */
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
    var query by rememberSaveable { mutableStateOf("") }
    val filtered = remember(chats, query) {
        val needle = query.trim().lowercase()
        if (chats == null || needle.isEmpty()) chats
        else chats.filter {
            it.title.lowercase().contains(needle) || it.lastMessagePreview.lowercase().contains(needle)
        }
    }

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
            chats == null -> repeat(SKELETON_ROWS) { SkeletonRow(if (it == 0) "loading chats…" else "") }
            chats.isEmpty() -> EmptyState(Lucide.Message, "No chats yet", "Start one with New chat. Every conversation with the crew lands here.")
            else -> {
                DsSearch(query, { query = it }, "Search chats…")
                val shown = filtered.orEmpty()
                if (shown.isEmpty()) {
                    EmptyState(Lucide.Search, "Nothing matches", "No chat's title or last message has \"$query\".")
                } else {
                    LazyColumn(Modifier.fillMaxWidth().padding(bottom = ListBottomPadding)) {
                        items(shown, key = { it.id }) { session ->
                            ChatRow(
                                agent = Crew.Nomi,
                                title = session.title.ifEmpty { UNTITLED_CHAT },
                                onClick = { onOpenChat(session.id) },
                                modifier = Modifier.animateItem(fadeInSpec = DsMotion.effects(), placementSpec = DsMotion.spatial(), fadeOutSpec = DsMotion.fast()),
                                time = timeAgo(session.updatedAt, nowMs),
                                preview = session.lastMessagePreview.ifBlank { NO_MESSAGES },
                                unread = session.unread,
                                selected = session.id == selectedId,
                            )
                        }
                    }
                }
            }
        }
    }
}
