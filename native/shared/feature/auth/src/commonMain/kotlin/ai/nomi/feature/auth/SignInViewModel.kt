package ai.nomi.feature.auth

import ai.nomi.data.SessionGate
import ai.nomi.network.ApiException
import ai.nomi.network.EmailCodeError
import ai.nomi.network.EmailCodeException
import ai.nomi.network.SignInResult
import ai.nomi.network.VerificationInfo
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

/**
 * The password step, then — when the server asks for one — the emailed-code step ([challenge] set).
 * [mustStartOver]: the sign-in expired or locked, so only "Start over" is left. [notice] is a non-error note
 * ("New code sent").
 */
data class SignInUiState(
    val email: String = "",
    val password: String = "",
    val busy: Boolean = false,
    val error: String? = null,
    val challenge: VerificationInfo? = null,
    val code: String = "",
    val notice: String? = null,
    val mustStartOver: Boolean = false,
)

const val CODE_LENGTH = 6
private const val SECONDS_PER_MINUTE = 60
private const val HTTP_UNAUTHORIZED = 401
private const val MESSAGE_MISSING_FIELDS = "Enter your email and password."
private const val MESSAGE_BAD_CREDENTIALS = "That email and password don't match."
private const val MESSAGE_FAILED = "Couldn't sign in. Check your connection and try again."

// The code step's copy matches the web's verify page (frontend/messages/en.json, verify_*).
private const val MESSAGE_SIX_DIGITS = "Enter the 6-digit code from the email."
private const val MESSAGE_LOCKED = "Too many wrong codes. Sign in again to get a new one."
private const val MESSAGE_EXPIRED = "This code has expired. Sign in again to get a new one."
private const val MESSAGE_CODE_FAILED = "Couldn't check the code. Try again."
private const val MESSAGE_RESENT = "New code sent. The old one no longer works."
private const val MESSAGE_NO_MORE_CODES = "No more codes can be sent for this sign-in. Start over to try again."
private const val MESSAGE_RESEND_FAILED = "Couldn't send the code email. Try again in a moment."

private fun wrongCodeMessage(left: Int) = "That code isn't right. $left tries left."
private fun waitMessage(seconds: Int) = "Wait $seconds seconds before asking for another code."

/** "We sent a 6-digit code to …" and how long it lasts, for the code step's header. */
fun VerificationInfo.instructions(): String =
    "We sent a $CODE_LENGTH-digit code to $email. Enter it to finish signing in. " +
        "It works for ${expiresIn / SECONDS_PER_MINUTE} minutes; check your spam folder if it isn't there."

/** Signing in routes the whole app: [SessionGate.signedIn] flips and the shell swaps the back stack. */
class SignInViewModel(private val gate: SessionGate) : ViewModel() {
    private val mutableState = MutableStateFlow(SignInUiState())
    val state: StateFlow<SignInUiState> = mutableState.asStateFlow()

    fun onEmailChange(value: String) = mutableState.update { it.copy(email = value, error = null) }

    fun onPasswordChange(value: String) = mutableState.update { it.copy(password = value, error = null) }

    /** Keeps digits only (pasted codes often carry spaces or dashes), at most [CODE_LENGTH]. */
    fun onCodeChange(value: String) =
        mutableState.update { it.copy(code = value.filter(Char::isDigit).take(CODE_LENGTH), error = null, notice = null) }

    fun submit() {
        val current = mutableState.value
        if (current.busy) return
        val email = current.email.trim()
        if (email.isEmpty() || current.password.isEmpty()) {
            mutableState.update { it.copy(error = MESSAGE_MISSING_FIELDS) }
            return
        }
        mutableState.update { it.copy(busy = true, error = null) }
        viewModelScope.launch {
            val next = try {
                when (val result = gate.signIn(email, current.password)) {
                    is SignInResult.SignedIn -> mutableState.value.copy(busy = false, password = "")
                    is SignInResult.EmailCode -> mutableState.value.copy(busy = false, challenge = result.verification, code = "")
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

    fun submitCode() {
        val current = mutableState.value
        val challenge = current.challenge ?: return
        if (current.busy || current.mustStartOver) return
        if (current.code.length != CODE_LENGTH) {
            mutableState.update { it.copy(error = MESSAGE_SIX_DIGITS) }
            return
        }
        mutableState.update { it.copy(busy = true, error = null, notice = null) }
        viewModelScope.launch {
            val next = try {
                gate.verifyCode(challenge.challengeId, current.code)
                mutableState.value.copy(busy = false, password = "", code = "")
            } catch (e: CancellationException) {
                throw e
            } catch (e: EmailCodeException) {
                when (val error = e.error) {
                    is EmailCodeError.WrongCode -> mutableState.value.copy(busy = false, code = "", error = wrongCodeMessage(error.attemptsLeft))
                    EmailCodeError.TooManyAttempts -> mutableState.value.copy(busy = false, error = MESSAGE_LOCKED, mustStartOver = true)
                    EmailCodeError.Expired -> mutableState.value.copy(busy = false, error = MESSAGE_EXPIRED, mustStartOver = true)
                    else -> mutableState.value.copy(busy = false, error = MESSAGE_CODE_FAILED)
                }
            } catch (_: Exception) {
                mutableState.value.copy(busy = false, error = MESSAGE_CODE_FAILED)
            }
            mutableState.value = next
        }
    }

    fun resendCode() {
        val current = mutableState.value
        val challenge = current.challenge ?: return
        if (current.busy || current.mustStartOver) return
        mutableState.update { it.copy(busy = true, error = null, notice = null) }
        viewModelScope.launch {
            val next = try {
                val fresh = gate.resendCode(challenge.challengeId)
                mutableState.value.copy(busy = false, challenge = fresh, code = "", notice = MESSAGE_RESENT)
            } catch (e: CancellationException) {
                throw e
            } catch (e: EmailCodeException) {
                when (val error = e.error) {
                    is EmailCodeError.Wait -> mutableState.value.copy(busy = false, error = waitMessage(error.seconds))
                    EmailCodeError.TooManyCodes -> mutableState.value.copy(busy = false, error = MESSAGE_NO_MORE_CODES, mustStartOver = true)
                    EmailCodeError.Expired, EmailCodeError.TooManyAttempts ->
                        mutableState.value.copy(busy = false, error = MESSAGE_EXPIRED, mustStartOver = true)
                    else -> mutableState.value.copy(busy = false, error = MESSAGE_RESEND_FAILED)
                }
            } catch (_: Exception) {
                mutableState.value.copy(busy = false, error = MESSAGE_RESEND_FAILED)
            }
            mutableState.value = next
        }
    }

    /** Back to the password step, keeping the email. */
    fun startOver() = mutableState.update { SignInUiState(email = it.email) }
}
