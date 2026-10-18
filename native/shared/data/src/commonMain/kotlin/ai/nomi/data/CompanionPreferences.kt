package ai.nomi.data

import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow

/**
 * User toggles for the companion surfaces: the desktop Dynamic Island and the mobile/in-app Messenger chat-head bubble.
 */
class CompanionPreferences {
    private val mutableIslandEnabled = MutableStateFlow(true)
    val islandEnabled: StateFlow<Boolean> = mutableIslandEnabled.asStateFlow()

    private val mutableBubbleEnabled = MutableStateFlow(true)
    val bubbleEnabled: StateFlow<Boolean> = mutableBubbleEnabled.asStateFlow()

    fun setIslandEnabled(enabled: Boolean) {
        mutableIslandEnabled.value = enabled
    }

    fun setBubbleEnabled(enabled: Boolean) {
        mutableBubbleEnabled.value = enabled
    }
}
