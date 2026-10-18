package ai.nomi.feature.settings

import ai.nomi.data.AppearanceRepository
import ai.nomi.data.CompanionPreferences
import ai.nomi.data.DEFAULT_ACCENT_COLOR
import ai.nomi.data.SessionGate
import ai.nomi.data.THEME_SYSTEM
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
    private val appearance: AppearanceRepository? = null,
    private val companionPreferences: CompanionPreferences? = null,
    private val log: (String) -> Unit = {},
) : ViewModel() {
    private val mutableProfile = MutableStateFlow<ProfileDto?>(null)

    /** Who is signed in; null until loaded (or when it couldn't be). */
    val profile: StateFlow<ProfileDto?> = mutableProfile.asStateFlow()

    val theme: StateFlow<String> = appearance?.theme ?: MutableStateFlow(THEME_SYSTEM).asStateFlow()
    val accentColor: StateFlow<String> = appearance?.accentColor ?: MutableStateFlow(DEFAULT_ACCENT_COLOR).asStateFlow()
    val islandEnabled: StateFlow<Boolean> = companionPreferences?.islandEnabled ?: MutableStateFlow(true).asStateFlow()
    val bubbleEnabled: StateFlow<Boolean> = companionPreferences?.bubbleEnabled ?: MutableStateFlow(true).asStateFlow()

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
        viewModelScope.launch { appearance?.refresh() }
    }

    fun selectTheme(theme: String) {
        appearance?.selectTheme(theme)
    }

    fun selectAccentColor(accentColor: String) {
        appearance?.selectAccentColor(accentColor)
    }

    fun setIslandEnabled(enabled: Boolean) {
        companionPreferences?.setIslandEnabled(enabled)
    }

    fun setBubbleEnabled(enabled: Boolean) {
        companionPreferences?.setBubbleEnabled(enabled)
    }

    /** The shell returns to sign-in once [SessionGate.signedIn] flips. */
    fun signOut() = gate.signOut()
}
