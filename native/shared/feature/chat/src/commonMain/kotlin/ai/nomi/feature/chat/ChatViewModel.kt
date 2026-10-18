package ai.nomi.feature.chat

import ai.nomi.core.STATE_STOP_TIMEOUT_MS
import ai.nomi.data.ChatRepository
import ai.nomi.data.ChatState
import ai.nomi.data.ForegroundChat
import ai.nomi.data.SessionRepository
import ai.nomi.network.ApiException
import ai.nomi.realtime.ListenerHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import io.ktor.http.HttpStatusCode
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

private const val MESSAGE_SEND_FAILED = "Couldn't send. Check your connection and try again."
private const val APPROVAL_FAILED = "Failed to record your decision."

data class ChatComposerState(val draft: String = "", val sending: Boolean = false, val error: String? = null)

/**
 * Where the person's decision on one approval card stands. [sent] stays true once the server took it, until the
 * card itself flips (over realtime); [expired] means the turn behind the card moved on, so deciding can't help.
 */
data class ApprovalState(val sending: Boolean = false, val sent: Boolean = false, val error: String? = null, val expired: Boolean = false)

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
        chats.observe(sessionId).stateIn(viewModelScope, SharingStarted.WhileSubscribed(STATE_STOP_TIMEOUT_MS), null)

    /** The chat's name, empty until the server names it. */
    val title: StateFlow<String> = sessions.observeAll()
        .map { all -> all.firstOrNull { it.id == sessionId }?.title.orEmpty() }
        .stateIn(viewModelScope, SharingStarted.WhileSubscribed(STATE_STOP_TIMEOUT_MS), "")

    private val mutableComposer = MutableStateFlow(ChatComposerState())
    val composer: StateFlow<ChatComposerState> = mutableComposer.asStateFlow()

    /** The chat is on screen: its replies are not unread nor popped up. */
    fun onVisible() {
        foreground.enter(sessionId)
        viewModelScope.launch { attempt("mark read") { sessions.markRead(sessionId) } }
    }

    fun onHidden() = foreground.leave(sessionId)

    private val mutableApprovals = MutableStateFlow<Map<String, ApprovalState>>(emptyMap())

    /** Decisions in flight or refused, by the approval card's message id; a card with no entry is untouched. */
    val approvals: StateFlow<Map<String, ApprovalState>> = mutableApprovals.asStateFlow()

    fun resolveApproval(messageId: String, approve: Boolean, remember: Boolean) {
        val current = mutableApprovals.value[messageId]
        if (current?.sending == true || current?.sent == true || current?.expired == true) return
        setApproval(messageId, ApprovalState(sending = true))
        viewModelScope.launch {
            val outcome = try {
                chats.resolveApproval(sessionId, messageId, approve, remember)
                ApprovalState(sent = true)
            } catch (e: CancellationException) {
                throw e
            } catch (e: ApiException) {
                log("chat: approval failed: $e")
                if (e.status == HttpStatusCode.Conflict.value) ApprovalState(expired = true) else ApprovalState(error = APPROVAL_FAILED)
            } catch (e: Exception) {
                log("chat: approval failed: $e")
                ApprovalState(error = APPROVAL_FAILED)
            }
            setApproval(messageId, outcome)
        }
    }

    private fun setApproval(messageId: String, state: ApprovalState) {
        mutableApprovals.update { it + (messageId to state) }
    }

    fun onDraftChange(value: String) {
        mutableComposer.value = mutableComposer.value.copy(draft = value, error = null)
    }

    /** Puts a suggestion in the composer (the person still sends it). */
    fun useSuggestion(text: String) = onDraftChange(text)

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
