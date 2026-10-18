package ai.nomi.feature.home

import ai.nomi.designsystem.component.Composer
import ai.nomi.designsystem.component.NomiListRow
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
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.semantics
import androidx.lifecycle.compose.collectAsStateWithLifecycle

const val HOME_SCREEN_TAG = "home-screen"

private const val UNTITLED_CHAT = "New chat"
private const val UNREAD_META = "New"

@Composable
fun HomeScreen(
    viewModel: HomeViewModel,
    onOpenChat: (sessionId: String) -> Unit,
    modifier: Modifier = Modifier,
) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    NomiPage(title = "What should we work on?", modifier = modifier.testTag(HOME_SCREEN_TAG)) {
        Composer(
            value = state.draft,
            onValueChange = viewModel::onDraftChange,
            onSend = { viewModel.startChat(onOpenChat) },
            placeholder = "Ask Nomi anything",
            sendLabel = "Start chat",
            sending = state.sending,
        )
        state.error?.let { Text(it, color = MaterialTheme.colorScheme.error, style = MaterialTheme.typography.bodyMedium) }
        if (state.recent.isNotEmpty()) {
            Text(
                "Recent chats",
                style = MaterialTheme.typography.titleSmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                modifier = Modifier.semantics { heading() },
            )
        }
        LazyColumn(Modifier.fillMaxWidth(), verticalArrangement = Arrangement.spacedBy(NomiSpacing.ExtraSmall)) {
            items(state.recent, key = { it.id }) { session ->
                NomiListRow(
                    title = session.title.ifEmpty { UNTITLED_CHAT },
                    supporting = session.lastMessagePreview,
                    meta = if (session.unread) UNREAD_META else null,
                    emphasized = session.unread,
                    onClick = { onOpenChat(session.id) },
                )
            }
        }
    }
}
