package ai.nomi.feature.notifications

import ai.nomi.core.timeAgo
import ai.nomi.data.NotificationItem
import ai.nomi.designsystem.component.NomiButton
import ai.nomi.designsystem.component.NomiButtonSize
import ai.nomi.designsystem.component.NomiButtonVariant
import ai.nomi.designsystem.component.NomiListPanel
import ai.nomi.designsystem.component.NomiLoadingIndicator
import ai.nomi.designsystem.component.NomiRow
import ai.nomi.designsystem.icon.NomiIcon
import ai.nomi.designsystem.icon.NomiIcons
import ai.nomi.designsystem.layout.NomiPage
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
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

const val NOTIFICATIONS_SCREEN_TAG = "notifications-screen"

private val IconBadgeSize = 36.dp
private val ListBottomPadding = 20.dp

/** Tapping a notification marks it read and hands its link (if any) to [onOpenLink]. */
@Composable
fun NotificationsScreen(
    viewModel: NotificationsViewModel,
    onOpenLink: (link: String) -> Unit,
    modifier: Modifier = Modifier,
) {
    val items by viewModel.items.collectAsStateWithLifecycle()
    NotificationsContent(
        items,
        onOpen = { item ->
            if (item.readAt == null) viewModel.markRead(item.id)
            item.link?.let(onOpenLink)
        },
        onMarkAllRead = viewModel::markAllRead,
        modifier = modifier,
    )
}

/** The inbox: the display title, then notices as rows in one panel; unread ones read bolder. */
@Composable
fun NotificationsContent(
    items: List<NotificationItem>?,
    onOpen: (NotificationItem) -> Unit,
    onMarkAllRead: () -> Unit,
    modifier: Modifier = Modifier,
    nowMs: Long = Clock.System.now().toEpochMilliseconds(),
) {
    NomiPage(
        title = "Inbox",
        modifier = modifier.testTag(NOTIFICATIONS_SCREEN_TAG),
        actions = {
            if (items.orEmpty().any { it.readAt == null }) {
                NomiButton(onClick = onMarkAllRead, variant = NomiButtonVariant.Tonal, size = NomiButtonSize.Small) { Text("Mark all read") }
            }
        },
    ) {
        when {
            items == null -> NomiLoadingIndicator(Modifier.align(Alignment.CenterHorizontally), contentDescription = "Loading notifications")
            items.isEmpty() -> Text("You're all caught up.", style = MaterialTheme.typography.bodyLarge, color = MaterialTheme.colorScheme.onSurfaceVariant)
            else -> NomiListPanel(Modifier.padding(bottom = ListBottomPadding)) {
                LazyColumn(Modifier.fillMaxWidth().clip(MaterialTheme.shapes.large)) {
                    items(items, key = { it.id }) { item ->
                        val unread = item.readAt == null
                        NomiRow(
                            title = item.title,
                            line = item.body,
                            meta = timeAgo(item.createdAt, nowMs),
                            emphasized = unread,
                            onClick = { onOpen(item) },
                            leading = { KindIcon(unread) },
                        )
                    }
                }
            }
        }
    }
}

/** The bell in a tonal circle: secondary container while unread, quieter once read. */
@Composable
private fun KindIcon(unread: Boolean) {
    val colors = MaterialTheme.colorScheme
    Box(
        Modifier.size(IconBadgeSize).clip(CircleShape).background(if (unread) colors.secondaryContainer else colors.surfaceContainerHigh),
        contentAlignment = Alignment.Center,
    ) {
        NomiIcon(NomiIcons.Bell, contentDescription = null, tint = if (unread) colors.onSecondaryContainer else colors.onSurfaceVariant)
    }
}
