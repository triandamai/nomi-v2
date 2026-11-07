package ai.nomi.app

import ai.nomi.data.GoogleSignInReturns
import ai.nomi.data.SessionGate
import ai.nomi.feature.auth.SignInStep
import ai.nomi.feature.auth.SignInViewModel
import ai.nomi.network.EmailCodeError
import ai.nomi.network.EmailCodeException
import ai.nomi.network.SignInResult
import ai.nomi.network.VerificationInfo
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.setMain
import org.koin.core.context.startKoin
import org.koin.core.context.stopKoin
import org.koin.core.Koin
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import kotlin.test.assertTrue

@OptIn(ExperimentalCoroutinesApi::class)
class SignInViewModelTest {
    private val graph = TestGraph()
    private lateinit var koin: Koin

    @BeforeTest
    fun start() {
        Dispatchers.setMain(UnconfinedTestDispatcher())
        koin = startKoin { modules(graph.modules) }.koin
    }

    @AfterTest
    fun stop() {
        stopKoin()
        Dispatchers.resetMain()
    }

    @Test
    fun successful_sign_in_signs_the_gate_in_and_begins_the_engine_session() {
        val vm = koin.get<SignInViewModel>()
        vm.onEmailChange("a@b.c")
        vm.onPasswordChange("pw")

        vm.submit()

        assertTrue(koin.get<SessionGate>().signedIn.value)
        assertEquals(1, graph.engine.begins)
        assertEquals(listOf("a@b.c" to "pw"), graph.auth.signIns)
        assertFalse(vm.state.value.busy)
    }

    @Test
    fun email_code_challenge_is_shown_and_nothing_starts() {
        graph.auth.nextSignIn = SignInResult.EmailCode(VerificationInfo("c1", "login", "a***@b.c", 600, 60, 3))
        val vm = koin.get<SignInViewModel>()
        vm.onEmailChange("a@b.c")
        vm.onPasswordChange("pw")

        vm.submit()

        assertEquals("a***@b.c", vm.state.value.challenge?.email)
        assertFalse(koin.get<SessionGate>().signedIn.value)
        assertEquals(0, graph.engine.begins)
    }

    @Test
    fun blank_fields_do_not_call_the_server() {
        val vm = koin.get<SignInViewModel>()

        vm.submit()

        assertNotNull(vm.state.value.error)
        assertTrue(graph.auth.signIns.isEmpty())
        assertNull(vm.state.value.challenge)
    }

    @Test
    fun forgot_password_sends_a_reset_link_and_can_go_again() {
        val vm = koin.get<SignInViewModel>()
        vm.onEmailChange(" a@b.c ")
        vm.openForgot()
        assertEquals(SignInStep.Forgot, vm.state.value.step)

        vm.sendResetLink()

        assertEquals(listOf("a@b.c"), graph.auth.forgotten)
        assertEquals(SignInStep.ResetSent, vm.state.value.step)
        assertEquals("a@b.c", vm.state.value.resetSentTo)
        assertEquals("Reset link sent.", vm.state.value.toast)
        vm.forgotAgain()
        assertEquals(SignInStep.Forgot, vm.state.value.step)
        vm.closeForgot()
        assertEquals(SignInStep.Password, vm.state.value.step)
    }

    @Test
    fun forgot_password_needs_an_email_and_says_when_it_fails() {
        val vm = koin.get<SignInViewModel>()
        vm.openForgot()
        vm.sendResetLink()
        assertEquals("Enter your email.", vm.state.value.error)
        assertTrue(graph.auth.forgotten.isEmpty())

        graph.auth.nextForgot = Result.failure(IllegalStateException("offline"))
        vm.onEmailChange("a@b.c")
        vm.sendResetLink()
        assertEquals(SignInStep.Forgot, vm.state.value.step)
        assertEquals("Couldn't send the link. Try again in a moment.", vm.state.value.error)
    }

    @Test
    fun google_opens_the_web_hand_off_and_its_answer_signs_in() {
        val vm = koin.get<SignInViewModel>()
        val opened = mutableListOf<String>()

        vm.continueWithGoogle { opened += it }
        assertTrue(vm.state.value.googleBusy)
        assertTrue(opened.single().endsWith("/auth/google/native"))

        GoogleSignInReturns.deliver("nomi://auth/google?code=c%2F0de&state=st")

        assertEquals(listOf("c/0de" to "st"), graph.auth.googleSignIns)
        assertTrue(koin.get<SessionGate>().signedIn.value)
        assertFalse(vm.state.value.googleBusy)
    }

    @Test
    fun a_cancelled_google_sign_in_says_so() {
        val vm = koin.get<SignInViewModel>()
        vm.continueWithGoogle { }

        GoogleSignInReturns.deliver("nomi://auth/google?error=access_denied")

        assertFalse(vm.state.value.googleBusy)
        assertEquals("Google sign-in was cancelled.", vm.state.value.error)
        assertFalse(koin.get<SessionGate>().signedIn.value)
    }

    @Test
    fun password_can_be_shown_and_hidden() {
        val vm = koin.get<SignInViewModel>()
        vm.togglePasswordVisible()
        assertTrue(vm.state.value.showPassword)
        vm.togglePasswordVisible()
        assertFalse(vm.state.value.showPassword)
    }

    private fun atCodeStep(): SignInViewModel {
        graph.auth.nextSignIn = SignInResult.EmailCode(VerificationInfo("c1", "login", "a***@b.c", 600, 60, 3))
        return koin.get<SignInViewModel>().apply {
            onEmailChange("a@b.c")
            onPasswordChange("pw")
            submit()
        }
    }

    @Test
    fun right_code_signs_in_and_begins_the_engine_session() {
        val vm = atCodeStep()
        vm.onCodeChange(" 12a3-456 ")

        vm.submitCode()

        assertEquals(listOf("c1" to "123456"), graph.auth.verified)
        assertTrue(koin.get<SessionGate>().signedIn.value)
        assertEquals(1, graph.engine.begins)
    }

    @Test
    fun code_must_be_six_digits_before_asking_the_server() {
        val vm = atCodeStep()
        vm.onCodeChange("123")

        vm.submitCode()

        assertTrue(graph.auth.verified.isEmpty())
        assertEquals(SIX_DIGITS, vm.state.value.error)
    }

    @Test
    fun wrong_code_says_how_many_tries_are_left_and_stays_on_the_code_step() {
        val vm = atCodeStep()
        graph.auth.nextCode = Result.failure(EmailCodeException(EmailCodeError.WrongCode(2)))
        vm.onCodeChange("000000")

        vm.submitCode()

        assertEquals("That code isn't right. 2 tries left.", vm.state.value.error)
        assertEquals("", vm.state.value.code)
        assertFalse(vm.state.value.mustStartOver)
        assertFalse(koin.get<SessionGate>().signedIn.value)
    }

    @Test
    fun expired_or_locked_sign_in_can_only_start_over() {
        val vm = atCodeStep()
        graph.auth.nextCode = Result.failure(EmailCodeException(EmailCodeError.TooManyAttempts))
        vm.onCodeChange("000000")

        vm.submitCode()

        assertTrue(vm.state.value.mustStartOver)
        vm.startOver()
        assertNull(vm.state.value.challenge)
        assertEquals("a@b.c", vm.state.value.email)
        assertEquals("", vm.state.value.password)
    }

    @Test
    fun resend_replaces_the_challenge_or_explains_the_wait() {
        val vm = atCodeStep()
        graph.auth.nextResend = Result.success(VerificationInfo("c2", "login", "a***@b.c", 600, 60, 1))

        vm.resendCode()

        assertEquals(listOf("c1"), graph.auth.resends)
        assertEquals("c2", vm.state.value.challenge?.challengeId)
        assertEquals("New code sent. The old one no longer works.", vm.state.value.notice)

        graph.auth.nextResend = Result.failure(EmailCodeException(EmailCodeError.Wait(42)))
        vm.resendCode()
        assertEquals("Wait 42 seconds before asking for another code.", vm.state.value.error)
    }

    private companion object {
        const val SIX_DIGITS = "Enter the 6-digit code from the email."
    }
}
