package ai.nomi.companion

import ai.nomi.data.CompanionSignal
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.advanceTimeBy
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs

@OptIn(ExperimentalCoroutinesApi::class)
class CompanionStateMachineTest {
    @Test
    fun workingEndsWithReplyThenPeeksAndSettlesToBadge() = runTest {
        val stateFlow = MutableStateFlow<CompanionSignal>(CompanionSignal.Idle)
        val signalsFlow = MutableSharedFlow<CompanionSignal>()
        val machine = CompanionStateMachine(backgroundScope, stateFlow, signalsFlow)
        runCurrent()
        assertEquals(CompanionUiState.Idle(unreadCount = 0), machine.state.value)

        // Agent starts working
        stateFlow.value = CompanionSignal.Working("coding", "thinking")
        runCurrent()
        assertEquals(CompanionUiState.Working("coding", "thinking", unreadCount = 0), machine.state.value)

        // Reply arrives and work ends
        signalsFlow.emit(CompanionSignal.NewReply("s1", "Here is the fix", "coding"))
        stateFlow.value = CompanionSignal.Idle
        runCurrent()

        val peeking = assertIs<CompanionUiState.Peeking>(machine.state.value)
        assertEquals("Here is the fix", (peeking.card as CompanionCard.Reply).preview)
        assertEquals(1, peeking.unreadCount)

        // After PEEK_MS, collapses to Idle with unread badge 1
        advanceTimeBy(PEEK_MS + 1)
        runCurrent()
        assertEquals(CompanionUiState.Idle(unreadCount = 1), machine.state.value)
    }

    @Test
    fun replyArrivingWhileHoveredStaysExpandedAndSwapsContent() = runTest {
        val stateFlow = MutableStateFlow<CompanionSignal>(CompanionSignal.Idle)
        val signalsFlow = MutableSharedFlow<CompanionSignal>()
        val machine = CompanionStateMachine(backgroundScope, stateFlow, signalsFlow)
        runCurrent()

        machine.setHovered(true)
        runCurrent()
        assertIs<CompanionUiState.Expanded>(machine.state.value)

        signalsFlow.emit(CompanionSignal.NewReply("s1", "First reply", null))
        runCurrent()
        val expanded1 = assertIs<CompanionUiState.Expanded>(machine.state.value)
        assertEquals("First reply", (expanded1.card as CompanionCard.Reply).preview)

        // Even after PEEK_MS passes, it remains Expanded because the cursor is still hovering
        advanceTimeBy(PEEK_MS * 2)
        runCurrent()
        assertIs<CompanionUiState.Expanded>(machine.state.value)

        // A second reply swaps the card in place
        signalsFlow.emit(CompanionSignal.NewReply("s2", "Second reply", "money"))
        runCurrent()
        val expanded2 = assertIs<CompanionUiState.Expanded>(machine.state.value)
        assertEquals("Second reply", (expanded2.card as CompanionCard.Reply).preview)

        // Leaving hover collapses after COLLAPSE_DELAY_MS
        machine.setHovered(false)
        advanceTimeBy(COLLAPSE_DELAY_MS - 50)
        runCurrent()
        assertIs<CompanionUiState.Expanded>(machine.state.value)

        advanceTimeBy(100)
        runCurrent()
        assertEquals(CompanionUiState.Idle(unreadCount = 0), machine.state.value)
    }

    @Test
    fun snoozedIgnoresSignalsUntilUnsnoozed() = runTest {
        val stateFlow = MutableStateFlow<CompanionSignal>(CompanionSignal.Idle)
        val signalsFlow = MutableSharedFlow<CompanionSignal>()
        val machine = CompanionStateMachine(backgroundScope, stateFlow, signalsFlow)
        runCurrent()

        machine.setSnoozed(true)
        assertEquals(CompanionUiState.Snoozed, machine.state.value)

        signalsFlow.emit(CompanionSignal.Notification("Quota", "80% used", "/billing"))
        runCurrent()
        assertEquals(CompanionUiState.Snoozed, machine.state.value)

        machine.setSnoozed(false)
        runCurrent()
        assertEquals(CompanionUiState.Idle(unreadCount = 0), machine.state.value)
    }
}
