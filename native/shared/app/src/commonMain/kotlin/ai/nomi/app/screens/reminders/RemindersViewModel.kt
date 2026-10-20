package ai.nomi.app.screens.reminders

import ai.nomi.data.RemindersRepository
import ai.nomi.network.CreateReminderRequestDto
import ai.nomi.network.ReminderDto
import ai.nomi.network.RemindersResponseDto
import ai.nomi.network.ScheduledTaskDto
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

private const val ACTION_DONE = "done"
private const val ACTION_CANCEL = "cancel"
private const val ACTION_SNOOZE = "snooze"
private const val SNOOZE_10_MIN = 10L
private const val SNOOZE_60_MIN = 60L
private const val SNOOZE_DAY_MIN = 1440L

data class RemindersUiState(
    val loading: Boolean = true,
    val reminders: RemindersResponseDto? = null,
    val error: String? = null,
    val createSheetOpen: Boolean = false,
    val saving: Boolean = false,
)

class RemindersViewModel(
    private val repository: RemindersRepository,
) : ViewModel() {

    private val _uiState = MutableStateFlow(RemindersUiState())
    val uiState: StateFlow<RemindersUiState> = _uiState.asStateFlow()

    init {
        loadReminders()
    }

    fun loadReminders() {
        viewModelScope.launch {
            _uiState.value = _uiState.value.copy(loading = true, error = null)
            val result = repository.getReminders()
            result.onSuccess { response ->
                _uiState.value = _uiState.value.copy(
                    loading = false,
                    reminders = response,
                    error = null,
                )
            }.onFailure { error ->
                _uiState.value = _uiState.value.copy(
                    loading = false,
                    error = error.message ?: "Failed to load reminders",
                )
            }
        }
    }

    fun openCreateSheet() {
        _uiState.value = _uiState.value.copy(createSheetOpen = true)
    }

    fun closeCreateSheet() {
        _uiState.value = _uiState.value.copy(createSheetOpen = false)
    }

    fun createReminder(
        title: String,
        dueAt: String,
        recurrence: String?,
        notes: String?,
    ) {
        viewModelScope.launch {
            _uiState.value = _uiState.value.copy(saving = true)
            val req = CreateReminderRequestDto(
                title = title.trim(),
                dueAt = dueAt,
                recurrence = recurrence?.takeIf { it.isNotBlank() },
                notes = notes?.takeIf { it.isNotBlank() },
            )
            repository.createReminder(req).onSuccess {
                _uiState.value = _uiState.value.copy(saving = false, createSheetOpen = false)
                loadReminders()
            }.onFailure { error ->
                _uiState.value = _uiState.value.copy(
                    saving = false,
                    error = error.message ?: "Failed to create reminder",
                )
            }
        }
    }

    fun markDone(id: String) {
        viewModelScope.launch {
            repository.action(id, ACTION_DONE).onSuccess {
                loadReminders()
            }
        }
    }

    fun cancelReminder(id: String) {
        viewModelScope.launch {
            repository.action(id, ACTION_CANCEL).onSuccess {
                loadReminders()
            }
        }
    }

    fun snooze(id: String, minutes: Long) {
        viewModelScope.launch {
            repository.action(id, ACTION_SNOOZE, minutes).onSuccess {
                loadReminders()
            }
        }
    }

    fun snooze10m(id: String) = snooze(id, SNOOZE_10_MIN)
    fun snooze1h(id: String) = snooze(id, SNOOZE_60_MIN)
    fun snoozeTomorrow(id: String) = snooze(id, SNOOZE_DAY_MIN)

    fun cancelScheduledTask(id: String) {
        viewModelScope.launch {
            repository.cancelScheduledTask(id).onSuccess {
                loadReminders()
            }
        }
    }
}
