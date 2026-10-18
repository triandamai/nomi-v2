package ai.nomi.app

import ai.nomi.data.ChatRepository
import ai.nomi.data.ForegroundChat
import ai.nomi.data.SessionRepository
import ai.nomi.data.SessionScope
import ai.nomi.feature.chat.ApprovalState
import ai.nomi.feature.chat.ChatViewModel
import ai.nomi.network.ApiException
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.withContext
import kotlinx.coroutines.withTimeout
import org.koin.core.context.startKoin
import org.koin.core.context.stopKoin
import org.koin.mp.KoinPlatform
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals

/** Deciding an approval card from the chat: what the view model sends and what the card shows afterwards. */
class ApprovalDecisionTest {
    private val graph = TestGraph()
    private lateinit var viewModel: ChatViewModel

    @BeforeTest
    fun start() {
        startKoin { modules(graph.modules) }
        val koin = KoinPlatform.getKoin()
        koin.get<SessionScope>().open()
        viewModel = runBlocking(Dispatchers.Main) {
            ChatViewModel(CHAT_ID, koin.get<ChatRepository>(), koin.get<SessionRepository>(), koin.get<ForegroundChat>())
        }
    }

    @AfterTest
    fun stop() = stopKoin()

    private fun decide(approve: Boolean, remember: Boolean): ApprovalState = runBlocking {
        withContext(Dispatchers.Main) { viewModel.resolveApproval(MESSAGE_ID, approve, remember) }
        withTimeout(TIMEOUT_MS) { viewModel.approvals.first { it[MESSAGE_ID]?.sending == false } }.getValue(MESSAGE_ID)
    }

    @Test
    fun a_decision_is_sent_once_and_the_card_waits_for_the_server() {
        assertEquals(ApprovalState(sent = true), decide(approve = true, remember = true))
        assertEquals(listOf(Triple(MESSAGE_ID, true, true)), graph.messagesApi.approvals)

        // A second tap while the card hasn't flipped yet sends nothing.
        runBlocking(Dispatchers.Main) { viewModel.resolveApproval(MESSAGE_ID, false, false) }
        assertEquals(1, graph.messagesApi.approvals.size)
    }

    @Test
    fun a_card_no_longer_pending_reads_as_expired() {
        graph.messagesApi.approvalFailure = ApiException(CONFLICT, "this action is no longer pending")
        assertEquals(ApprovalState(expired = true), decide(approve = false, remember = false))
    }

    @Test
    fun any_other_failure_can_be_retried() {
        graph.messagesApi.approvalFailure = ApiException(SERVER_ERROR, "failed to queue approval decision")
        assertEquals(ApprovalState(error = "Failed to record your decision."), decide(approve = true, remember = false))

        graph.messagesApi.approvalFailure = null
        assertEquals(ApprovalState(sent = true), decide(approve = true, remember = false))
    }

    private companion object {
        const val CHAT_ID = "s1"
        const val MESSAGE_ID = "m1"
        const val CONFLICT = 409
        const val SERVER_ERROR = 500
        const val TIMEOUT_MS = 5_000L
    }
}
