package ai.nomi.feature.home

import ai.nomi.data.ChatRepository
import ai.nomi.data.Session
import ai.nomi.data.SessionRepository
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch

/** Chats shown under the composer on Home; the full list is the Chats screen. */
const val HOME_RECENT_CHATS = 5

/** How long Room keeps feeding a screen that went to the background (configuration changes resubscribe within it). */
private const val STOP_TIMEOUT_MS = 5_000L
private const val MESSAGE_START_FAILED = "Couldn't start the chat. Check your connection and try again."

data class HomeUiState(
    val recent: List<Session> = emptyList(),
    val draft: String = "",
    val sending: Boolean = false,
    val error: String? = null,
)

class HomeViewModel(
    private val sessions: SessionRepository,
    private val chats: ChatRepository,
    private val log: (String) -> Unit = {},
) : ViewModel() {
    private val form = MutableStateFlow(HomeUiState())

    val state: StateFlow<HomeUiState> = combine(form, sessions.observeAll().map { it.take(HOME_RECENT_CHATS) }) { f, recent ->
        f.copy(recent = recent)
    }.stateIn(viewModelScope, SharingStarted.WhileSubscribed(STOP_TIMEOUT_MS), HomeUiState())

    init {
        viewModelScope.launch { attempt("refresh chats") { sessions.refresh() } }
    }

    fun onDraftChange(value: String) {
        form.value = form.value.copy(draft = value, error = null)
    }

    /** Creates a chat, sends the draft as its first message, then hands the new chat's id to [onStarted]. */
    fun startChat(onStarted: (sessionId: String) -> Unit) {
        val current = form.value
        val text = current.draft.trim()
        if (text.isEmpty() || current.sending) return
        form.value = current.copy(sending = true, error = null)
        viewModelScope.launch {
            try {
                val id = sessions.create()
                chats.send(id, text)
                form.value = form.value.copy(draft = "", sending = false)
                onStarted(id)
                attempt("refresh chats") { sessions.refresh() }
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                log("home: starting a chat failed: $e")
                form.value = form.value.copy(sending = false, error = MESSAGE_START_FAILED)
            }
        }
    }

    private suspend fun attempt(what: String, block: suspend () -> Unit) {
        try {
            block()
        } catch (e: CancellationException) {
            throw e
        } catch (e: Exception) {
            log("home: $what failed: $e")
        }
    }
}
