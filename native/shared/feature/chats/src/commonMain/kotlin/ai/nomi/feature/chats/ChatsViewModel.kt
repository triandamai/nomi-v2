package ai.nomi.feature.chats

import ai.nomi.core.STATE_STOP_TIMEOUT_MS
import ai.nomi.data.Session
import ai.nomi.data.SessionRepository
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch


/** The chat list, from Room; [refresh] pulls the server's list into it. Null until Room's first read. */
class ChatsViewModel(
    private val sessions: SessionRepository,
    private val log: (String) -> Unit = {},
) : ViewModel() {
    val chats: StateFlow<List<Session>?> =
        sessions.observeAll().stateIn(viewModelScope, SharingStarted.WhileSubscribed(STATE_STOP_TIMEOUT_MS), null)

    private val mutableRefreshing = MutableStateFlow(false)
    val refreshing: StateFlow<Boolean> = mutableRefreshing.asStateFlow()

    init {
        refresh()
    }

    fun refresh() {
        if (mutableRefreshing.value) return
        mutableRefreshing.value = true
        viewModelScope.launch {
            try {
                sessions.refresh()
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                log("chats: refresh failed: $e")
            } finally {
                mutableRefreshing.value = false
            }
        }
    }
}
