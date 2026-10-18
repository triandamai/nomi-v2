package ai.nomi.feature.settings

import ai.nomi.data.SessionGate
import ai.nomi.network.HomeApi
import ai.nomi.network.ProfileDto
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

class SettingsViewModel(
    private val gate: SessionGate,
    private val home: HomeApi,
    private val log: (String) -> Unit = {},
) : ViewModel() {
    private val mutableProfile = MutableStateFlow<ProfileDto?>(null)

    /** Who is signed in; null until loaded (or when it couldn't be). */
    val profile: StateFlow<ProfileDto?> = mutableProfile.asStateFlow()

    init {
        viewModelScope.launch {
            try {
                mutableProfile.value = home.profile()
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                log("settings: load profile failed: $e")
            }
        }
    }

    /** The shell returns to sign-in once [SessionGate.signedIn] flips. */
    fun signOut() = gate.signOut()
}
