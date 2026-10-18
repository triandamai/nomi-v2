package ai.nomi.feature.notifications

import ai.nomi.core.timeAgo
import ai.nomi.data.NotificationItem
import ai.nomi.designsystem.component.NomiButton
import ai.nomi.designsystem.component.NomiButtonSize
import ai.nomi.designsystem.component.NomiButtonVariant
import ai.nomi.designsystem.component.NomiFilterChip
import ai.nomi.designsystem.component.NomiListPanel
import ai.nomi.designsystem.component.NomiLoadingIndicator
import ai.nomi.designsystem.component.NomiRow
import ai.nomi.designsystem.layout.NomiPage
import ai.nomi.designsystem.shape.AgentIdentity
import ai.nomi.designsystem.shape.AgentShape
import ai.nomi.designsystem.shape.NomiIdentity
import ai.nomi.designsystem.shape.agentIdentity
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
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

const val NOTIFICATIONS_SCREEN_TAG = "notifications-screen"

private val RowAvatarSize = 36.dp
private val EmptyAvatarSize = 64.dp
private val EmptyVerticalPadding = 36.dp
private val EmptyHorizontalPadding = 14.dp
private val EmptyGap = 12.dp
private val FilterGap = 8.dp
private val ListBottomPadding = 20.dp

enum class NotificationFilter(val label: String, val emptyMessage: String) {
    All("All", "You're all caught up."),
    Unread("Unread", "Nothing unread right now."),
    Read("Read", "No read notifications yet."),
    Promo("Promo", "No promotions yet."),
}

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

/** The inbox: the display title, filter chips, then notices as rows with crew shapes; unread ones read bolder. */
@OptIn(ExperimentalLayoutApi::class)
@Composable
fun NotificationsContent(
    items: List<NotificationItem>?,
    onOpen: (NotificationItem) -> Unit,
    onMarkAllRead: () -> Unit,
    modifier: Modifier = Modifier,
    nowMs: Long = Clock.System.now().toEpochMilliseconds(),
) {
    var filter by rememberSaveable { mutableStateOf(NotificationFilter.All) }
    val filtered = remember(items, filter) {
        items?.let { all ->
            when (filter) {
                NotificationFilter.All -> all
                NotificationFilter.Unread -> all.filter { it.readAt == null }
                NotificationFilter.Read -> all.filter { it.readAt != null }
                NotificationFilter.Promo -> all.filter { it.kind == "promo" }
            }
        }
    }

    NomiPage(
        title = "Inbox",
        modifier = modifier.testTag(NOTIFICATIONS_SCREEN_TAG),
        actions = {
            if (items.orEmpty().any { it.readAt == null }) {
                NomiButton(onClick = onMarkAllRead, variant = NomiButtonVariant.Tonal, size = NomiButtonSize.Small) { Text("Mark all read") }
            }
        },
    ) {
        if (items != null) {
            FlowRow(
                horizontalArrangement = Arrangement.spacedBy(FilterGap),
                verticalArrangement = Arrangement.spacedBy(FilterGap),
            ) {
                NotificationFilter.entries.forEach { entry ->
                    NomiFilterChip(
                        label = entry.label,
                        selected = filter == entry,
                        onClick = { filter = entry },
                    )
                }
            }
        }

        when {
            filtered == null -> NomiLoadingIndicator(Modifier.align(Alignment.CenterHorizontally), contentDescription = "Loading notifications")
            filtered.isEmpty() -> EmptyInboxCard(filter.emptyMessage)
            else -> NomiListPanel(Modifier.padding(bottom = ListBottomPadding)) {
                LazyColumn(Modifier.fillMaxWidth().clip(MaterialTheme.shapes.large)) {
                    items(filtered, key = { it.id }) { item ->
                        val unread = item.readAt == null
                        NomiRow(
                            title = item.title,
                            line = item.body,
                            meta = timeAgo(item.createdAt, nowMs),
                            emphasized = unread,
                            onClick = { onOpen(item) },
                            leading = { AgentShape(kindIdentity(item.kind), RowAvatarSize) },
                        )
                    }
                }
            }
        }
    }
}

@Composable
private fun EmptyInboxCard(message: String) {
    Column(
        Modifier
            .fillMaxWidth()
            .clip(MaterialTheme.shapes.extraLarge)
            .background(MaterialTheme.colorScheme.surfaceContainerLow)
            .padding(vertical = EmptyVerticalPadding, horizontal = EmptyHorizontalPadding),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(EmptyGap),
    ) {
        AgentShape(NomiIdentity, EmptyAvatarSize, face = true)
        Text(message, style = MaterialTheme.typography.bodyLarge, color = MaterialTheme.colorScheme.onSurfaceVariant)
    }
}

/** Mirrors `KIND` in `frontend/src/routes/(app)/notifications/+page.svelte`. */
private fun kindIdentity(kind: String): AgentIdentity = when (kind) {
    "subscription" -> agentIdentity("supervisor")
    "quota" -> agentIdentity("money")
    "promo" -> agentIdentity("reminders")
    "deploy", "project" -> agentIdentity("coding")
    else -> NomiIdentity
}
