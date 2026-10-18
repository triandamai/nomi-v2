package ai.nomi.app

import ai.nomi.data.SessionGate
import ai.nomi.feature.auth.SignInViewModel
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
    fun successful_sign_in_signs_the_gate_in_and_starts_the_hub() {
        val vm = koin.get<SignInViewModel>()
        vm.onEmailChange("a@b.c")
        vm.onPasswordChange("pw")

        vm.submit()

        assertTrue(koin.get<SessionGate>().signedIn.value)
        assertEquals(1, graph.hub.starts)
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

        assertEquals("a***@b.c", vm.state.value.codeSentTo)
        assertFalse(koin.get<SessionGate>().signedIn.value)
        assertEquals(0, graph.hub.starts)
    }

    @Test
    fun blank_fields_do_not_call_the_server() {
        val vm = koin.get<SignInViewModel>()

        vm.submit()

        assertNotNull(vm.state.value.error)
        assertTrue(graph.auth.signIns.isEmpty())
        assertNull(vm.state.value.codeSentTo)
    }
}
