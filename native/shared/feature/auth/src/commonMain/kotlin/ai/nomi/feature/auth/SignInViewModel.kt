package ai.nomi.feature.auth

import ai.nomi.data.SessionGate
import ai.nomi.network.ApiException
import ai.nomi.network.SignInResult
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

/** [codeSentTo] is set when the server answered with an email-code challenge instead of a session. */
data class SignInUiState(
    val email: String = "",
    val password: String = "",
    val busy: Boolean = false,
    val error: String? = null,
    val codeSentTo: String? = null,
)

private const val HTTP_UNAUTHORIZED = 401
private const val MESSAGE_MISSING_FIELDS = "Enter your email and password."
private const val MESSAGE_BAD_CREDENTIALS = "That email and password don't match."
private const val MESSAGE_FAILED = "Couldn't sign in. Check your connection and try again."

/** Signing in routes the whole app: [SessionGate.signedIn] flips and the shell swaps the back stack. */
class SignInViewModel(private val gate: SessionGate) : ViewModel() {
    private val mutableState = MutableStateFlow(SignInUiState())
    val state: StateFlow<SignInUiState> = mutableState.asStateFlow()

    fun onEmailChange(value: String) = mutableState.update { it.copy(email = value, error = null) }

    fun onPasswordChange(value: String) = mutableState.update { it.copy(password = value, error = null) }

    fun submit() {
        val current = mutableState.value
        if (current.busy) return
        val email = current.email.trim()
        if (email.isEmpty() || current.password.isEmpty()) {
            mutableState.update { it.copy(error = MESSAGE_MISSING_FIELDS) }
            return
        }
        mutableState.update { it.copy(busy = true, error = null, codeSentTo = null) }
        viewModelScope.launch {
            val next = try {
                when (val result = gate.signIn(email, current.password)) {
                    is SignInResult.SignedIn -> mutableState.value.copy(busy = false, password = "")
                    is SignInResult.EmailCode -> mutableState.value.copy(busy = false, codeSentTo = result.verification.email)
                }
            } catch (e: CancellationException) {
                throw e
            } catch (e: ApiException) {
                val message = if (e.status == HTTP_UNAUTHORIZED) MESSAGE_BAD_CREDENTIALS else MESSAGE_FAILED
                mutableState.value.copy(busy = false, error = message)
            } catch (_: Exception) {
                mutableState.value.copy(busy = false, error = MESSAGE_FAILED)
            }
            mutableState.value = next
        }
    }
}
