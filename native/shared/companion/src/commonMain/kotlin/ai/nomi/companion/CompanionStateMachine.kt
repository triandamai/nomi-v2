package ai.nomi.companion

import ai.nomi.data.CompanionSignal
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

const val PEEK_MS = 4_000L
const val COLLAPSE_DELAY_MS = 400L
private const val INITIAL_UNREAD = 0
private const val UNREAD_INCREMENT = 1

/** What an expanded or peeking companion card shows and where tapping "Open" navigates. */
sealed interface CompanionCard {
    val title: String
    val body: String
    val link: String
    val sessionId: String?
    val agentType: String?

    data class Reply(
        override val sessionId: String,
        val preview: String,
        override val agentType: String? = null,
    ) : CompanionCard {
        override val title: String = "New reply"
        override val body: String = preview.ifBlank { "Tap to read the reply" }
        override val link: String = "/chat/$sessionId"
    }

    data class Notice(
        override val title: String,
        override val body: String,
        val rawLink: String?,
    ) : CompanionCard {
        override val link: String = rawLink?.takeIf { it.isNotBlank() } ?: "/notifications"
        override val sessionId: String? = null
        override val agentType: String? = null
    }

    data class Activity(
        override val sessionId: String,
        override val title: String,
        override val body: String,
        val isError: Boolean = false,
    ) : CompanionCard {
        override val link: String = "/chat/$sessionId"
        override val agentType: String? = null
    }
}

/** Visual state shared by the desktop Dynamic Island and the Android/in-app Messenger chat-head bubble. */
sealed interface CompanionUiState {
    data object Snoozed : CompanionUiState
    data class Idle(val unreadCount: Int = INITIAL_UNREAD) : CompanionUiState
    data class Working(val agentType: String, val phase: String, val unreadCount: Int = INITIAL_UNREAD) : CompanionUiState
    data class Peeking(val card: CompanionCard, val unreadCount: Int = INITIAL_UNREAD) : CompanionUiState
    data class Expanded(
        val card: CompanionCard?,
        val working: CompanionSignal.Working? = null,
        val unreadCount: Int = INITIAL_UNREAD,
    ) : CompanionUiState
}

/** Human-readable label for a backend agent phase slug. */
fun formatAgentPhase(phase: String): String = when (phase) {
    "thinking" -> "Thinking…"
    "calling_tool", "using_tool" -> "Using a tool…"
    "writing", "writing_reply" -> "Writing a reply…"
    else -> phase.replace('_', ' ').replaceFirstChar { it.uppercase() }
}

/**
 * Pure state machine driving both the desktop Dynamic Island and the Android Messenger-style chat head.
 *
 * - Collects [companionState] (`Working` / `Idle`) and [companionSignals] (`NewReply`, `Notification`, `NeedsYou`,
 *   `ChatDone`, `ChatFailed`).
 * - Peeks for [peekDurationMs] when a signal arrives while collapsed, then settles into [CompanionUiState.Idle] (or
 *   [CompanionUiState.Working]) with an incremented unread badge.
 * - Hovering (desktop) or tapping open (mobile) expands immediately, cancels any pending peek/collapse timer, and
 *   keeps the surface expanded even if a new reply arrives mid-hover (the card swaps in place).
 */
class CompanionStateMachine(
    private val scope: CoroutineScope,
    companionState: Flow<CompanionSignal>,
    companionSignals: Flow<CompanionSignal>,
    private val peekDurationMs: Long = PEEK_MS,
    private val collapseDelayMs: Long = COLLAPSE_DELAY_MS,
) {
    private val mutableState = MutableStateFlow<CompanionUiState>(CompanionUiState.Idle())
    val state: StateFlow<CompanionUiState> = mutableState.asStateFlow()

    private var currentWorking: CompanionSignal.Working? = null
    private var lastCard: CompanionCard? = null
    private var unreadCount: Int = INITIAL_UNREAD
    private var hovered: Boolean = false
    private var pinnedOpen: Boolean = false
    private var snoozed: Boolean = false
    private var timerJob: Job? = null

    init {
        scope.launch {
            companionState.collect { signal ->
                currentWorking = signal as? CompanionSignal.Working
                if (snoozed) return@collect
                when (val cur = mutableState.value) {
                    is CompanionUiState.Expanded -> {
                        mutableState.value = cur.copy(working = currentWorking)
                    }
                    is CompanionUiState.Peeking -> Unit
                    else -> settleCollapsed()
                }
            }
        }
        scope.launch {
            companionSignals.collect { signal ->
                val card = signal.toCard() ?: return@collect
                lastCard = card
                if (snoozed) return@collect
                if (hovered || pinnedOpen || mutableState.value is CompanionUiState.Expanded) {
                    timerJob?.cancel()
                    unreadCount = INITIAL_UNREAD
                    mutableState.value = CompanionUiState.Expanded(
                        card = card,
                        working = currentWorking,
                        unreadCount = INITIAL_UNREAD,
                    )
                } else {
                    unreadCount += UNREAD_INCREMENT
                    mutableState.value = CompanionUiState.Peeking(card = card, unreadCount = unreadCount)
                    timerJob?.cancel()
                    timerJob = scope.launch {
                        delay(peekDurationMs)
                        if (!hovered && !pinnedOpen && !snoozed) {
                            settleCollapsed()
                        }
                    }
                }
            }
        }
    }

    /** Called by the desktop island when the cursor enters or leaves the pill's hit rect. */
    fun setHovered(isHovered: Boolean) {
        if (hovered == isHovered) return
        hovered = isHovered
        if (snoozed) return
        timerJob?.cancel()
        if (isHovered) {
            unreadCount = INITIAL_UNREAD
            mutableState.value = CompanionUiState.Expanded(
                card = lastCard,
                working = currentWorking,
                unreadCount = INITIAL_UNREAD,
            )
        } else if (!pinnedOpen) {
            timerJob = scope.launch {
                delay(collapseDelayMs)
                if (!hovered && !pinnedOpen && !snoozed) {
                    settleCollapsed()
                }
            }
        }
    }

    /** Called by the touch bubble (or island click) to toggle the expanded quick-chat card. */
    fun toggleExpanded() {
        if (snoozed) return
        timerJob?.cancel()
        if (mutableState.value is CompanionUiState.Expanded) {
            pinnedOpen = false
            hovered = false
            settleCollapsed()
        } else {
            pinnedOpen = true
            unreadCount = INITIAL_UNREAD
            mutableState.value = CompanionUiState.Expanded(
                card = lastCard,
                working = currentWorking,
                unreadCount = INITIAL_UNREAD,
            )
        }
    }

    /** Collapses an expanded or peeking companion immediately. */
    fun collapse() {
        timerJob?.cancel()
        pinnedOpen = false
        hovered = false
        if (!snoozed) settleCollapsed()
    }

    /** Pauses or resumes the companion surface ("Pause island" in tray / Settings toggle). */
    fun setSnoozed(isSnoozed: Boolean) {
        if (snoozed == isSnoozed) return
        snoozed = isSnoozed
        timerJob?.cancel()
        if (isSnoozed) {
            hovered = false
            pinnedOpen = false
            mutableState.value = CompanionUiState.Snoozed
        } else {
            settleCollapsed()
        }
    }

    fun clearUnread() {
        unreadCount = INITIAL_UNREAD
        when (val cur = mutableState.value) {
            is CompanionUiState.Idle -> mutableState.value = cur.copy(unreadCount = INITIAL_UNREAD)
            is CompanionUiState.Working -> mutableState.value = cur.copy(unreadCount = INITIAL_UNREAD)
            is CompanionUiState.Peeking -> mutableState.value = cur.copy(unreadCount = INITIAL_UNREAD)
            is CompanionUiState.Expanded -> mutableState.value = cur.copy(unreadCount = INITIAL_UNREAD)
            CompanionUiState.Snoozed -> Unit
        }
    }

    private fun settleCollapsed() {
        val working = currentWorking
        mutableState.value = if (working != null) {
            CompanionUiState.Working(working.agentType, working.phase, unreadCount = unreadCount)
        } else {
            CompanionUiState.Idle(unreadCount = unreadCount)
        }
    }
}

private fun CompanionSignal.toCard(): CompanionCard? = when (this) {
    is CompanionSignal.NewReply -> CompanionCard.Reply(sessionId = sessionId, preview = preview, agentType = agentType)
    is CompanionSignal.Notification -> CompanionCard.Notice(title = title, body = body, rawLink = link)
    is CompanionSignal.NeedsYou -> CompanionCard.Activity(
        sessionId = sessionId,
        title = title?.takeIf { it.isNotBlank() } ?: "Needs your input",
        body = when (waiting) {
            "approval" -> "Waiting for your approval"
            "connect" -> "Waiting for a connection"
            else -> "Waiting for your answer"
        },
    )
    is CompanionSignal.ChatDone -> CompanionCard.Activity(
        sessionId = sessionId,
        title = title?.takeIf { it.isNotBlank() } ?: "Chat finished",
        body = "Your crew wrapped up.",
    )
    is CompanionSignal.ChatFailed -> CompanionCard.Activity(
        sessionId = sessionId,
        title = title?.takeIf { it.isNotBlank() } ?: "Chat stopped",
        body = "Something went wrong in this chat.",
        isError = true,
    )
    is CompanionSignal.Working, CompanionSignal.Idle -> null
}
