package ai.nomi.app.screens

import ai.nomi.feature.auth.SignInContent
import ai.nomi.feature.auth.SignInUiState
import ai.nomi.network.VerificationInfo
import androidx.compose.ui.test.junit4.createComposeRule
import org.junit.Rule
import kotlin.test.Test

class SignInScreenshotTest {
    @get:Rule val rule = createComposeRule()

    private val filled = SignInUiState(email = "ana@nomi.test", password = "secret")
    private val code = SignInUiState(
        email = "ana@nomi.test",
        challenge = VerificationInfo("c1", "login", "a***@nomi.test", 600, 60, 3),
        code = "1234",
    )

    private fun capture(name: String, device: Device, dark: Boolean, state: SignInUiState) = rule.captureScreen(name, device, dark) {
        SignInContent(state, {}, {}, {}, {}, {}, {}, {})
    }

    @Test fun passwordPhoneLight() = capture("sign_in", Device.Phone, false, filled)
    @Test fun passwordPhoneDark() = capture("sign_in", Device.Phone, true, filled)
    @Test fun passwordWideLight() = capture("sign_in", Device.Wide, false, filled)
    @Test fun passwordWideDark() = capture("sign_in", Device.Wide, true, filled)
    @Test fun codePhoneLight() = capture("sign_in_code", Device.Phone, false, code)
    @Test fun wrongPasswordPhoneLight() = capture("sign_in_error", Device.Phone, false, filled.copy(error = "That email and password don't match."))
}
