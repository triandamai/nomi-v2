package ai.nomi.app.screens.reminders

import ai.nomi.designsystem.component.NomiButton
import ai.nomi.designsystem.component.NomiButtonSize
import ai.nomi.designsystem.component.NomiButtonVariant
import ai.nomi.designsystem.component.NomiCard
import ai.nomi.designsystem.component.NomiFilterChip
import ai.nomi.designsystem.component.NomiListPanel
import ai.nomi.designsystem.component.NomiLoadingIndicator
import ai.nomi.designsystem.component.NomiRow
import ai.nomi.designsystem.component.NomiSectionHeader
import ai.nomi.designsystem.component.NomiTextField
import ai.nomi.designsystem.icon.NomiIcon
import ai.nomi.designsystem.icon.NomiIcons
import ai.nomi.designsystem.layout.NomiPage
import ai.nomi.designsystem.shape.AgentShape
import ai.nomi.designsystem.shape.agentIdentity
import ai.nomi.network.ReminderDto
import ai.nomi.network.RemindersResponseDto
import ai.nomi.network.ScheduledTaskDto
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.BasicAlertDialog
import androidx.compose.material3.Checkbox
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.compose.collectAsStateWithLifecycle

private const val FONT_WEIGHT_BOLD = 700
private const val FONT_WEIGHT_SEMI = 600
private const val PAST_ALPHA = 0.6f
private const val DATE_PREFIX_LEN = 16

private val SectionSpacing = 20.dp
private val InnerCardPadding = 16.dp
private val SmallSpacing = 8.dp
private val MediumSpacing = 12.dp
private val AgentAvatarSize = 32.dp
private val IconBadgeSize = 36.dp
private val DialogPadding = 20.dp
private val DialogCorner = 20.dp
private val BottomListPadding = 32.dp

@Composable
fun RemindersScreen(
    viewModel: RemindersViewModel,
    modifier: Modifier = Modifier,
) {
    val state by viewModel.uiState.collectAsStateWithLifecycle()

    NomiPage(
        title = "Reminders",
        modifier = modifier,
        actions = {
            NomiButton(
                onClick = viewModel::openCreateSheet,
                variant = NomiButtonVariant.Filled,
                size = NomiButtonSize.Small,
            ) {
                NomiIcon(NomiIcons.Plus, contentDescription = null, size = 16.dp)
                Spacer(Modifier.width(SmallSpacing))
                Text("New")
            }
        },
    ) {
        if (state.loading && state.reminders == null) {
            Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                NomiLoadingIndicator(contentDescription = "Loading reminders")
            }
        } else {
            val remindersData = state.reminders
            if (remindersData != null) {
                RemindersContent(
                    data = remindersData,
                    onDone = viewModel::markDone,
                    onCancel = viewModel::cancelReminder,
                    onSnooze10m = viewModel::snooze10m,
                    onSnooze1h = viewModel::snooze1h,
                    onSnoozeTomorrow = viewModel::snoozeTomorrow,
                    onCancelTask = viewModel::cancelScheduledTask,
                )
            } else if (state.error != null) {
                Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                    Text(
                        state.error ?: "Failed to load",
                        color = MaterialTheme.colorScheme.error,
                        style = MaterialTheme.typography.bodyLarge,
                    )
                }
            }
        }

        if (state.createSheetOpen) {
            CreateReminderDialog(
                onDismiss = viewModel::closeCreateSheet,
                onConfirm = { title, dueAt, recurrence, notes ->
                    viewModel.createReminder(title, dueAt, recurrence, notes)
                },
                saving = state.saving,
            )
        }
    }
}

@Composable
private fun RemindersContent(
    data: RemindersResponseDto,
    onDone: (String) -> Unit,
    onCancel: (String) -> Unit,
    onSnooze10m: (String) -> Unit,
    onSnooze1h: (String) -> Unit,
    onSnoozeTomorrow: (String) -> Unit,
    onCancelTask: (String) -> Unit,
) {
    Column(
        Modifier
            .fillMaxWidth()
            .verticalScroll(rememberScrollState())
            .padding(bottom = BottomListPadding),
        verticalArrangement = Arrangement.spacedBy(SectionSpacing),
    ) {
        // Upcoming Reminders
        NomiSectionHeader("Upcoming (${data.upcoming.size})")
        if (data.upcoming.isEmpty()) {
            NomiListPanel {
                Box(Modifier.fillMaxWidth().padding(InnerCardPadding), contentAlignment = Alignment.Center) {
                    Text(
                        "No upcoming reminders. Relax!",
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }
        } else {
            NomiListPanel {
                data.upcoming.forEach { reminder ->
                    ReminderItemRow(
                        reminder = reminder,
                        onDone = { onDone(reminder.id) },
                        onCancel = { onCancel(reminder.id) },
                        onSnooze10m = { onSnooze10m(reminder.id) },
                        onSnooze1h = { onSnooze1h(reminder.id) },
                        onSnoozeTomorrow = { onSnoozeTomorrow(reminder.id) },
                    )
                }
            }
        }

        // Scheduled Agent Tasks
        if (data.scheduledTasks.isNotEmpty()) {
            NomiSectionHeader("Scheduled Agent Tasks (${data.scheduledTasks.size})")
            NomiListPanel {
                data.scheduledTasks.forEach { task ->
                    ScheduledTaskRow(
                        task = task,
                        onCancel = { onCancelTask(task.id) },
                    )
                }
            }
        }

        // Past Reminders
        if (data.past.isNotEmpty()) {
            NomiSectionHeader("Completed (${data.past.size})")
            NomiListPanel {
                data.past.forEach { reminder ->
                    PastReminderRow(reminder)
                }
            }
        }
    }
}

@Composable
private fun ReminderItemRow(
    reminder: ReminderDto,
    onDone: () -> Unit,
    onCancel: () -> Unit,
    onSnooze10m: () -> Unit,
    onSnooze1h: () -> Unit,
    onSnoozeTomorrow: () -> Unit,
) {
    var menuExpanded by remember { mutableStateOf(false) }

    val dueText = reminder.dueAt.take(DATE_PREFIX_LEN).replace('T', ' ')
    val subtitle = if (!reminder.notes.isNullOrBlank()) {
        "$dueText · ${reminder.notes}"
    } else {
        dueText
    }

    Row(
        Modifier
            .fillMaxWidth()
            .padding(horizontal = InnerCardPadding, vertical = SmallSpacing),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(SmallSpacing),
    ) {
        Checkbox(
            checked = false,
            onCheckedChange = { onDone() },
        )
        Column(Modifier.weight(1f)) {
            Text(
                reminder.title,
                style = MaterialTheme.typography.bodyLarge.copy(fontWeight = FontWeight(FONT_WEIGHT_SEMI)),
                color = MaterialTheme.colorScheme.onSurface,
            )
            Text(
                subtitle,
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
        Box {
            IconButton(onClick = { menuExpanded = true }) {
                NomiIcon(NomiIcons.Clock, contentDescription = "Snooze options")
            }
            DropdownMenu(
                expanded = menuExpanded,
                onDismissRequest = { menuExpanded = false },
            ) {
                DropdownMenuItem(
                    text = { Text("+10 minutes") },
                    onClick = {
                        menuExpanded = false
                        onSnooze10m()
                    },
                )
                DropdownMenuItem(
                    text = { Text("+1 hour") },
                    onClick = {
                        menuExpanded = false
                        onSnooze1h()
                    },
                )
                DropdownMenuItem(
                    text = { Text("Tomorrow") },
                    onClick = {
                        menuExpanded = false
                        onSnoozeTomorrow()
                    },
                )
                DropdownMenuItem(
                    text = { Text("Cancel reminder", color = MaterialTheme.colorScheme.error) },
                    onClick = {
                        menuExpanded = false
                        onCancel()
                    },
                )
            }
        }
    }
}

@Composable
private fun ScheduledTaskRow(
    task: ScheduledTaskDto,
    onCancel: () -> Unit,
) {
    val runText = task.runAt.take(DATE_PREFIX_LEN).replace('T', ' ')
    val subtitle = "Runs at $runText by ${task.agent}"

    Row(
        Modifier
            .fillMaxWidth()
            .padding(horizontal = InnerCardPadding, vertical = SmallSpacing),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(SmallSpacing),
    ) {
        AgentShape(
            identity = agentIdentity(task.agent),
            size = AgentAvatarSize,
        )
        Column(Modifier.weight(1f)) {
            Text(
                task.label,
                style = MaterialTheme.typography.bodyLarge.copy(fontWeight = FontWeight(FONT_WEIGHT_SEMI)),
                color = MaterialTheme.colorScheme.onSurface,
            )
            Text(
                subtitle,
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
        IconButton(onClick = onCancel) {
            NomiIcon(NomiIcons.Close, contentDescription = "Cancel task")
        }
    }
}

@Composable
private fun PastReminderRow(reminder: ReminderDto) {
    val completedDate = reminder.dueAt.take(DATE_PREFIX_LEN).replace('T', ' ')
    NomiRow(
        title = reminder.title,
        onClick = null,
        line = "Completed · $completedDate",
        leading = {
            Box(
                Modifier
                    .size(IconBadgeSize)
                    .clip(CircleShape)
                    .background(MaterialTheme.colorScheme.surfaceContainerHighest),
                contentAlignment = Alignment.Center,
            ) {
                NomiIcon(NomiIcons.Check, contentDescription = null, tint = MaterialTheme.colorScheme.primary)
            }
        },
    )
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun CreateReminderDialog(
    onDismiss: () -> Unit,
    onConfirm: (title: String, dueAt: String, recurrence: String?, notes: String?) -> Unit,
    saving: Boolean,
) {
    var title by remember { mutableStateOf("") }
    var dueAt by remember { mutableStateOf("2026-10-20T09:00:00") }
    var recurrence by remember { mutableStateOf<String?>("once") }
    var notes by remember { mutableStateOf("") }

    val recurrenceOptions = listOf("once" to "Once", "daily" to "Daily", "weekly" to "Weekly", "monthly" to "Monthly")

    BasicAlertDialog(onDismissRequest = onDismiss) {
        Surface(
            shape = RoundedCornerShape(DialogCorner),
            color = MaterialTheme.colorScheme.surfaceContainerHigh,
            modifier = Modifier.padding(DialogPadding),
        ) {
            Column(
                Modifier.padding(DialogPadding),
                verticalArrangement = Arrangement.spacedBy(MediumSpacing),
            ) {
                Text(
                    "New Reminder",
                    style = MaterialTheme.typography.titleLarge.copy(fontWeight = FontWeight(FONT_WEIGHT_BOLD)),
                    color = MaterialTheme.colorScheme.onSurface,
                )

                NomiTextField(
                    value = title,
                    onValueChange = { title = it },
                    label = "Reminder",
                    placeholder = "Remind me to...",
                    modifier = Modifier.fillMaxWidth(),
                )

                NomiTextField(
                    value = dueAt,
                    onValueChange = { dueAt = it },
                    label = "Due Date & Time",
                    placeholder = "YYYY-MM-DDTHH:MM:SS",
                    modifier = Modifier.fillMaxWidth(),
                )

                Row(horizontalArrangement = Arrangement.spacedBy(SmallSpacing)) {
                    recurrenceOptions.forEach { (key, label) ->
                        NomiFilterChip(
                            label = label,
                            selected = recurrence == key,
                            onClick = { recurrence = key },
                        )
                    }
                }

                NomiTextField(
                    value = notes,
                    onValueChange = { notes = it },
                    label = "Notes",
                    placeholder = "Notes (optional)",
                    modifier = Modifier.fillMaxWidth(),
                )

                Row(
                    Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.End,
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    NomiButton(onClick = onDismiss, variant = NomiButtonVariant.Text) {
                        Text("Cancel")
                    }
                    Spacer(Modifier.width(SmallSpacing))
                    NomiButton(
                        onClick = {
                            if (title.isNotBlank()) {
                                onConfirm(title, dueAt, recurrence, notes)
                            }
                        },
                        variant = NomiButtonVariant.Filled,
                        enabled = !saving && title.isNotBlank(),
                    ) {
                        Text(if (saving) "Saving..." else "Save")
                    }
                }
            }
        }
    }
}
