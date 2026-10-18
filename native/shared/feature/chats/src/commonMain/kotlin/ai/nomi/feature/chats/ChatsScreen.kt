package ai.nomi.feature.chats

import ai.nomi.designsystem.component.NomiButton
import ai.nomi.designsystem.component.NomiButtonVariant
import ai.nomi.designsystem.component.NomiListRow
import ai.nomi.designsystem.component.NomiLoadingIndicator
import ai.nomi.designsystem.layout.NomiPage
import ai.nomi.designsystem.layout.NomiSpacing
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle

const val CHATS_SCREEN_TAG = "chats-screen"

private const val UNTITLED_CHAT = "New chat"
private const val UNREAD_META = "New"
private val RefreshIndicatorSize = 24.dp

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
    NomiPage(
        title = "Chats",
        modifier = modifier.testTag(CHATS_SCREEN_TAG),
        actions = {
            if (refreshing) {
                NomiLoadingIndicator(size = RefreshIndicatorSize, contentDescription = "Refreshing")
            } else {
                NomiButton(onClick = viewModel::refresh, variant = NomiButtonVariant.Text) { Text("Refresh") }
            }
        },
    ) {
        val list = chats
        when {
            list == null -> NomiLoadingIndicator(Modifier.align(Alignment.CenterHorizontally), contentDescription = "Loading chats")
            list.isEmpty() -> Text(
                "No chats yet. Start one from Home.",
                style = MaterialTheme.typography.bodyLarge,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            else -> LazyColumn(Modifier.fillMaxWidth(), verticalArrangement = Arrangement.spacedBy(NomiSpacing.ExtraSmall)) {
                items(list, key = { it.id }) { session ->
                    NomiListRow(
                        title = session.title.ifEmpty { UNTITLED_CHAT },
                        supporting = session.lastMessagePreview,
                        meta = if (session.unread) UNREAD_META else null,
                        emphasized = session.unread,
                        selected = session.id == selectedId,
                        onClick = { onOpenChat(session.id) },
                    )
                }
            }
        }
    }
}
