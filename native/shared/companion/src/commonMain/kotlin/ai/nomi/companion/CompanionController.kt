package ai.nomi.companion

import ai.nomi.data.ChatRepository
import ai.nomi.data.CompanionPreferences
import ai.nomi.data.RealtimeSync
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.channels.BufferOverflow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

private const val NAV_BUFFER = 8

/**
 * Coordinates the companion surfaces (desktop Dynamic Island, Android Messenger chat-head overlay, and in-app floating
 * bubble) with [RealtimeSync], quick replies via [ChatRepository], user toggles, and deep-link navigation back into
 * the main `NomiApp` shell.
 */
class CompanionController(
    private val scope: CoroutineScope,
    sync: RealtimeSync,
    private val chatRepository: ChatRepository,
    private val preferences: CompanionPreferences,
    private val log: (String) -> Unit = {},
) {
    val stateMachine = CompanionStateMachine(
        scope = scope,
        companionState = sync.companionState,
        companionSignals = sync.companionSignals,
    )

    val uiState: StateFlow<CompanionUiState> = stateMachine.state

    val islandEnabled: StateFlow<Boolean> = preferences.islandEnabled
    val bubbleEnabled: StateFlow<Boolean> = preferences.bubbleEnabled

    private val mutableSnoozed = MutableStateFlow(false)
    val snoozed: StateFlow<Boolean> = mutableSnoozed.asStateFlow()

    private val mutableNavRequests = MutableSharedFlow<String>(
        extraBufferCapacity = NAV_BUFFER,
        onBufferOverflow = BufferOverflow.DROP_OLDEST,
    )

    /** Deep-link paths (`/chat/{id}`, `/notifications`, `/`) emitted when the person clicks "Open" in a companion. */
    val navigationRequests: SharedFlow<String> = mutableNavRequests.asSharedFlow()

    fun setIslandEnabled(enabled: Boolean) {
        preferences.setIslandEnabled(enabled)
    }

    fun setBubbleEnabled(enabled: Boolean) {
        preferences.setBubbleEnabled(enabled)
    }

    fun setSnoozed(snoozed: Boolean) {
        mutableSnoozed.value = snoozed
        stateMachine.setSnoozed(snoozed)
    }

    fun toggleSnoozed() {
        setSnoozed(!mutableSnoozed.value)
    }

    fun openLink(link: String) {
        stateMachine.collapse()
        mutableNavRequests.tryEmit(link)
    }

    fun openApp() {
        stateMachine.collapse()
        mutableNavRequests.tryEmit("/")
    }

    fun sendQuickReply(sessionId: String, text: String) {
        stateMachine.collapse()
        scope.launch {
            try {
                chatRepository.send(sessionId, text)
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                log("companion: quick reply failed: $e")
            }
        }
    }
}
