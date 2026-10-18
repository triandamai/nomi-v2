package ai.nomi.feature.notifications

import ai.nomi.core.STATE_STOP_TIMEOUT_MS
import ai.nomi.data.NotificationItem
import ai.nomi.data.NotificationRepository
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch


/** The inbox from Room. Null until Room's first read. */
class NotificationsViewModel(
    private val notifications: NotificationRepository,
    private val log: (String) -> Unit = {},
) : ViewModel() {
    val items: StateFlow<List<NotificationItem>?> =
        notifications.observeAll().stateIn(viewModelScope, SharingStarted.WhileSubscribed(STATE_STOP_TIMEOUT_MS), null)

    init {
        launchAttempt("refresh") { notifications.refresh() }
    }

    fun markRead(id: String) = launchAttempt("mark read") { notifications.markRead(id) }

    fun markAllRead() = launchAttempt("mark all read") { notifications.markAllRead() }

    private fun launchAttempt(what: String, block: suspend () -> Unit) {
        viewModelScope.launch {
            try {
                block()
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                log("notifications: $what failed: $e")
            }
        }
    }
}
