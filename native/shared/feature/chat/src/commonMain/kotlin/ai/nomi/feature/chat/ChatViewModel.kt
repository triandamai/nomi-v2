package ai.nomi.feature.chat

import ai.nomi.data.ChatRepository
import ai.nomi.data.ChatState
import ai.nomi.data.ForegroundChat
import ai.nomi.data.SessionRepository
import ai.nomi.realtime.ListenerHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch

private const val STOP_TIMEOUT_MS = 5_000L
private const val MESSAGE_SEND_FAILED = "Couldn't send. Check your connection and try again."

data class ChatComposerState(val draft: String = "", val sending: Boolean = false, val error: String? = null)

/**
 * One open chat. Created per navigation entry, so the chat's realtime binding lives exactly as long as the entry is
 * on the back stack: bound in init, released in [onCleared]. Call from the main thread (ViewModels are).
 */
class ChatViewModel(
    val sessionId: String,
    private val chats: ChatRepository,
    private val sessions: SessionRepository,
    private val foreground: ForegroundChat,
    private val log: (String) -> Unit = {},
) : ViewModel() {
    private val binding: ListenerHandle = chats.bind(sessionId)

    /** Null until Room's first read. */
    val chat: StateFlow<ChatState?> =
        chats.observe(sessionId).stateIn(viewModelScope, SharingStarted.WhileSubscribed(STOP_TIMEOUT_MS), null)

    private val mutableComposer = MutableStateFlow(ChatComposerState())
    val composer: StateFlow<ChatComposerState> = mutableComposer.asStateFlow()

    /** The chat is on screen: its replies are not unread nor popped up. */
    fun onVisible() {
        foreground.enter(sessionId)
        viewModelScope.launch { attempt("mark read") { sessions.markRead(sessionId) } }
    }

    fun onHidden() = foreground.leave(sessionId)

    fun onDraftChange(value: String) {
        mutableComposer.value = mutableComposer.value.copy(draft = value, error = null)
    }

    fun send() {
        val current = mutableComposer.value
        val text = current.draft.trim()
        if (text.isEmpty() || current.sending) return
        mutableComposer.value = current.copy(sending = true, error = null)
        viewModelScope.launch {
            mutableComposer.value = try {
                chats.send(sessionId, text)
                mutableComposer.value.copy(draft = "", sending = false)
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                log("chat: send failed: $e")
                mutableComposer.value.copy(sending = false, error = MESSAGE_SEND_FAILED)
            }
        }
    }

    override fun onCleared() {
        binding.remove()
        foreground.leave(sessionId)
    }

    private suspend fun attempt(what: String, block: suspend () -> Unit) {
        try {
            block()
        } catch (e: CancellationException) {
            throw e
        } catch (e: Exception) {
            log("chat: $what failed: $e")
        }
    }
}
