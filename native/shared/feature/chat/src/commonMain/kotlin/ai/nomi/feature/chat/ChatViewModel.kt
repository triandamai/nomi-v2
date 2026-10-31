package ai.nomi.feature.chat

import ai.nomi.core.STATE_STOP_TIMEOUT_MS
import ai.nomi.data.ChatRepository
import ai.nomi.data.ChatState
import ai.nomi.data.ForegroundChat
import ai.nomi.data.ListenerHandle
import ai.nomi.data.SessionRepository
import ai.nomi.network.ApiException
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
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

private const val MESSAGE_RETRY_FAILED = "Couldn't retry that message. Try again."
private const val MESSAGE_DISCARD_FAILED = "Couldn't remove that message. Try again."
private const val FEEDBACK_FAILED = "Couldn't save your feedback. Try again."
private const val DELETE_FAILED = "Couldn't delete it. Try again."
private const val REGENERATE_FAILED = "Couldn't ask again. Try again in a moment."
private const val SHARE_FAILED = "Couldn't make the link. Try again."

/** The backend's answer to a decision on a card that is no longer pending. */
private const val HTTP_CONFLICT = 409

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

    /** Null until the store's first read. */
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
                if (e.status == HTTP_CONFLICT) ApprovalState(expired = true) else ApprovalState(error = APPROVAL_FAILED)
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

    private val mutableFeedback = MutableStateFlow<Map<String, ReplyFeedbackState>>(emptyMap())

    /** Each reply's feedback beyond its stored rating (the "what went wrong" form, the thanks), by message id. */
    val feedback: StateFlow<Map<String, ReplyFeedbackState>> = mutableFeedback.asStateFlow()

    private fun setFeedback(messageId: String, change: (ReplyFeedbackState) -> ReplyFeedbackState) {
        mutableFeedback.update { it + (messageId to change(it[messageId] ?: ReplyFeedbackState())) }
    }

    /** Rates a reply, or takes the rating back with null; a thumbs-down then asks what went wrong. */
    fun rate(messageId: String, rating: String?) {
        setFeedback(messageId) { ReplyFeedbackState(dislikeOpen = rating == RATING_DOWN) }
        replyAction(FEEDBACK_FAILED) { chats.rate(sessionId, messageId, rating) }
    }

    fun closeDislike(messageId: String) = setFeedback(messageId) { it.copy(dislikeOpen = false) }

    fun sendDislike(messageId: String, dislike: Dislike) {
        setFeedback(messageId) { it.copy(sending = true) }
        viewModelScope.launch {
            val sent = attemptOr(FEEDBACK_FAILED) {
                chats.rate(sessionId, messageId, RATING_DOWN, dislike.reasons.map { it.wire }, dislike.note.ifEmpty { null }, dislike.includeChat)
            }
            setFeedback(messageId) { if (sent) ReplyFeedbackState(sentWithChat = dislike.includeChat) else it.copy(sending = false) }
        }
    }

    fun deleteReply(messageId: String) = replyAction(DELETE_FAILED) { chats.delete(sessionId, messageId) }

    fun regenerate(messageId: String, style: RegenerateStyle) = replyAction(REGENERATE_FAILED) { chats.regenerate(sessionId, messageId, style.wire) }

    /** Makes a link to the reply and hands it to [deliver] (the share sheet, or the clipboard). */
    fun shareReply(messageId: String, deliver: (String?) -> Unit) {
        viewModelScope.launch {
            var link: String? = null
            attemptOr(SHARE_FAILED) { link = chats.shareLink(sessionId, messageId) }
            deliver(link)
        }
    }

    private fun replyAction(failure: String, action: suspend () -> Unit) {
        viewModelScope.launch { attemptOr(failure, action) }
    }

    /** Runs [action]; a failure shows [failure] over the composer. Returns whether it worked. */
    private suspend fun attemptOr(failure: String, action: suspend () -> Unit): Boolean {
        mutableComposer.value = mutableComposer.value.copy(error = null)
        return try {
            action()
            true
        } catch (e: CancellationException) {
            throw e
        } catch (e: Exception) {
            log("chat: reply action failed: $e")
            mutableComposer.value = mutableComposer.value.copy(error = failure)
            false
        }
    }

    /** Sends a message that wasn't sent again. */
    fun retrySend(messageId: String) = outboxAction(messageId, MESSAGE_RETRY_FAILED) { chats.retrySend(it) }

    /** Drops a message that wasn't sent. */
    fun discardSend(messageId: String) = outboxAction(messageId, MESSAGE_DISCARD_FAILED) { chats.discardSend(it) }

    private fun outboxAction(messageId: String, failure: String, action: suspend (String) -> Boolean) {
        mutableComposer.value = mutableComposer.value.copy(error = null)
        viewModelScope.launch {
            try {
                // False: it was delivered or removed meanwhile, and the thread already shows that.
                action(messageId)
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                log("chat: outbox action failed: $e")
                mutableComposer.value = mutableComposer.value.copy(error = failure)
            }
        }
    }

    fun onDraftChange(value: String) {
        mutableComposer.value = mutableComposer.value.copy(draft = value, error = null)
    }

    /** Puts a suggestion in the composer (the person still sends it). */
    fun useSuggestion(text: String) = onDraftChange(text)

    /**
     * Sends an answer a view gave for the person (a picked suggestion, decision, form or questions answers) as their
     * message. The composer's draft is left alone.
     */
    fun sendAnswer(text: String) {
        val answer = text.trim()
        if (answer.isEmpty()) return
        mutableComposer.value = mutableComposer.value.copy(sending = true, error = null)
        viewModelScope.launch {
            mutableComposer.value = try {
                chats.send(sessionId, answer)
                mutableComposer.value.copy(sending = false)
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                log("chat: sending an answer failed: $e")
                mutableComposer.value.copy(sending = false, error = MESSAGE_SEND_FAILED)
            }
        }
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
