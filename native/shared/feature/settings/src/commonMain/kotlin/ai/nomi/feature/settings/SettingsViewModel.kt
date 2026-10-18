package ai.nomi.feature.settings

import ai.nomi.data.SessionGate
import androidx.lifecycle.ViewModel

class SettingsViewModel(private val gate: SessionGate) : ViewModel() {
    /** The shell returns to sign-in once [SessionGate.signedIn] flips. */
    fun signOut() = gate.signOut()
}
