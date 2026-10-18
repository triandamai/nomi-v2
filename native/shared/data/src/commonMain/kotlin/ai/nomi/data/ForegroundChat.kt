package ai.nomi.data

import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow

/** The chat currently on screen, set by the chat screen; replies to it are not marked unread nor popped up. */
class ForegroundChat {
    private val current = MutableStateFlow<String?>(null)

    val sessionId: StateFlow<String?> = current.asStateFlow()

    fun enter(sessionId: String) {
        current.value = sessionId
    }

    /** Only clears when [sessionId] is still the foreground one, so leaving A after entering B keeps B. */
    fun leave(sessionId: String) {
        current.compareAndSet(sessionId, null)
    }
}
