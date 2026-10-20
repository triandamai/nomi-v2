package ai.nomi.feature.home

import ai.nomi.designsystem.component.NomiButton
import ai.nomi.designsystem.component.NomiButtonSize
import ai.nomi.designsystem.component.NomiButtonVariant
import ai.nomi.designsystem.shape.AgentShape
import ai.nomi.designsystem.shape.agentIdentity
import ai.nomi.network.HomeSummaryDto
import ai.nomi.network.OutItemDto
import ai.nomi.network.PlanItemDto
import ai.nomi.network.TodayItemDto
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.role
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp

private val CardRadius = 24.dp
private val CardPadding = 20.dp
private val CardGap = 14.dp
private val ContentGap = 12.dp
private val ItemAvatar = 28.dp

@OptIn(ExperimentalLayoutApi::class)
@Composable
fun HomeStatusCards(
    summary: HomeSummaryDto,
    onOpenReminders: () -> Unit,
    onOpenPlans: () -> Unit,
    onOpenChat: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    BoxWithConstraints(modifier.fillMaxWidth()) {
        val minWidth = 280.dp
        val isWide = maxWidth >= 840.dp
        if (isWide) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(CardGap),
            ) {
                OutCard(
                    items = summary.whileYouWereOut,
                    total = summary.whileYouWereOutTotal,
                    onOpenChat = onOpenChat,
                    modifier = Modifier.weight(1f),
                )
                TodayCard(
                    items = summary.today,
                    total = summary.todayTotal,
                    onOpenReminders = onOpenReminders,
                    modifier = Modifier.weight(1f),
                )
                PlansCard(
                    items = summary.plans,
                    total = summary.plansTotal,
                    onOpenPlans = onOpenPlans,
                    onOpenChat = onOpenChat,
                    modifier = Modifier.weight(1f),
                )
            }
        } else {
            Column(
                modifier = Modifier.fillMaxWidth(),
                verticalArrangement = Arrangement.spacedBy(CardGap),
            ) {
                OutCard(
                    items = summary.whileYouWereOut,
                    total = summary.whileYouWereOutTotal,
                    onOpenChat = onOpenChat,
                    modifier = Modifier.fillMaxWidth(),
                )
                TodayCard(
                    items = summary.today,
                    total = summary.todayTotal,
                    onOpenReminders = onOpenReminders,
                    modifier = Modifier.fillMaxWidth(),
                )
                PlansCard(
                    items = summary.plans,
                    total = summary.plansTotal,
                    onOpenPlans = onOpenPlans,
                    onOpenChat = onOpenChat,
                    modifier = Modifier.fillMaxWidth(),
                )
            }
        }
    }
}

@Composable
private fun OutCard(
    items: List<OutItemDto>,
    total: Int,
    onOpenChat: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    StatusCardContainer(modifier) {
        Text(
            text = "While you were out",
            style = MaterialTheme.typography.titleMedium.copy(fontWeight = FontWeight.Bold),
            color = MaterialTheme.colorScheme.onSurface,
        )
        if (items.isEmpty()) {
            Text(
                text = "Nothing moved while you were away.",
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        } else {
            Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                items.forEach { item ->
                    val sid = item.sessionId
                    val clickModifier = if (sid != null) {
                        Modifier
                            .clickable { onOpenChat(sid) }
                            .semantics { this.role = Role.Button }
                    } else Modifier
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .clip(RoundedCornerShape(12.dp))
                            .then(clickModifier)
                            .padding(vertical = 4.dp),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(10.dp),
                    ) {
                        AgentShape(agentIdentity(item.agent), ItemAvatar)
                        Column(modifier = Modifier.weight(1f)) {
                            Text(
                                text = item.title,
                                style = MaterialTheme.typography.bodyMedium.copy(fontWeight = FontWeight.SemiBold),
                                color = MaterialTheme.colorScheme.onSurface,
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis,
                            )
                            if (item.detail.isNotBlank()) {
                                Text(
                                    text = item.detail,
                                    style = MaterialTheme.typography.bodySmall,
                                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                                    maxLines = 1,
                                    overflow = TextOverflow.Ellipsis,
                                )
                            }
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun TodayCard(
    items: List<TodayItemDto>,
    total: Int,
    onOpenReminders: () -> Unit,
    modifier: Modifier = Modifier,
) {
    StatusCardContainer(modifier) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text(
                text = "Today",
                style = MaterialTheme.typography.titleMedium.copy(fontWeight = FontWeight.Bold),
                color = MaterialTheme.colorScheme.onSurface,
            )
            NomiButton(
                onClick = onOpenReminders,
                variant = NomiButtonVariant.Text,
                size = NomiButtonSize.Small,
            ) {
                Text("Reminders")
            }
        }
        if (items.isEmpty()) {
            Text(
                text = "No reminders or routines due today.",
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        } else {
            Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                items.forEach { item ->
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .clip(RoundedCornerShape(12.dp))
                            .clickable(onClick = onOpenReminders)
                            .padding(vertical = 4.dp),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(10.dp),
                    ) {
                        AgentShape(agentIdentity(item.agent), ItemAvatar)
                        Column(modifier = Modifier.weight(1f)) {
                            Text(
                                text = item.label,
                                style = MaterialTheme.typography.bodyMedium.copy(fontWeight = FontWeight.SemiBold),
                                color = MaterialTheme.colorScheme.onSurface,
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis,
                            )
                            val meta = if (item.recurrence != null) "Due today · ${item.recurrence}" else "Due today"
                            Text(
                                text = meta,
                                style = MaterialTheme.typography.bodySmall,
                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                            )
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun PlansCard(
    items: List<PlanItemDto>,
    total: Int,
    onOpenPlans: () -> Unit,
    onOpenChat: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    StatusCardContainer(modifier) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text(
                text = "Plans in progress",
                style = MaterialTheme.typography.titleMedium.copy(fontWeight = FontWeight.Bold),
                color = MaterialTheme.colorScheme.onSurface,
            )
            NomiButton(
                onClick = onOpenPlans,
                variant = NomiButtonVariant.Text,
                size = NomiButtonSize.Small,
            ) {
                Text("All plans")
            }
        }
        if (items.isEmpty()) {
            Text(
                text = "No active plans or to-do lists right now.",
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        } else {
            Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                items.forEach { item ->
                    Column(
                        modifier = Modifier
                            .fillMaxWidth()
                            .clip(RoundedCornerShape(12.dp))
                            .clickable { onOpenChat(item.sessionId) }
                            .padding(vertical = 4.dp),
                        verticalArrangement = Arrangement.spacedBy(4.dp),
                    ) {
                        Row(
                            modifier = Modifier.fillMaxWidth(),
                            horizontalArrangement = Arrangement.SpaceBetween,
                            verticalAlignment = Alignment.CenterVertically,
                        ) {
                            Text(
                                text = item.title,
                                style = MaterialTheme.typography.bodyMedium.copy(fontWeight = FontWeight.SemiBold),
                                color = MaterialTheme.colorScheme.onSurface,
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis,
                                modifier = Modifier.weight(1f),
                            )
                            if (item.total > 0) {
                                Text(
                                    text = "${item.done}/${item.total}",
                                    style = MaterialTheme.typography.labelSmall,
                                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                                )
                            }
                        }
                        if (item.total > 0) {
                            val progress = (item.done.toFloat() / item.total.toFloat()).coerceIn(0f, 1f)
                            LinearProgressIndicator(
                                progress = { progress },
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .clip(RoundedCornerShape(4.dp)),
                            )
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun StatusCardContainer(
    modifier: Modifier = Modifier,
    content: @Composable () -> Unit,
) {
    Column(
        modifier = modifier
            .clip(RoundedCornerShape(CardRadius))
            .background(MaterialTheme.colorScheme.surfaceContainerLowest)
            .padding(CardPadding),
        verticalArrangement = Arrangement.spacedBy(ContentGap),
    ) {
        content()
    }
}
