package ai.nomi.feature.notifications

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
import androidx.lifecycle.compose.collectAsStateWithLifecycle

const val NOTIFICATIONS_SCREEN_TAG = "notifications-screen"

private const val UNREAD_META = "New"

/** Tapping a notification marks it read and hands its link (if any) to [onOpenLink]. */
@Composable
fun NotificationsScreen(
    viewModel: NotificationsViewModel,
    onOpenLink: (link: String) -> Unit,
    modifier: Modifier = Modifier,
) {
    val items by viewModel.items.collectAsStateWithLifecycle()
    NomiPage(
        title = "Notifications",
        modifier = modifier.testTag(NOTIFICATIONS_SCREEN_TAG),
        actions = {
            if (items.orEmpty().any { it.readAt == null }) {
                NomiButton(onClick = viewModel::markAllRead, variant = NomiButtonVariant.Text) { Text("Mark all read") }
            }
        },
    ) {
        val list = items
        when {
            list == null -> NomiLoadingIndicator(Modifier.align(Alignment.CenterHorizontally), contentDescription = "Loading notifications")
            list.isEmpty() -> Text(
                "You're all caught up.",
                style = MaterialTheme.typography.bodyLarge,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            else -> LazyColumn(Modifier.fillMaxWidth(), verticalArrangement = Arrangement.spacedBy(NomiSpacing.ExtraSmall)) {
                items(list, key = { it.id }) { item ->
                    val unread = item.readAt == null
                    NomiListRow(
                        title = item.title,
                        supporting = item.body,
                        meta = if (unread) UNREAD_META else null,
                        emphasized = unread,
                        onClick = {
                            if (unread) viewModel.markRead(item.id)
                            item.link?.let(onOpenLink)
                        },
                    )
                }
            }
        }
    }
}
