package ai.nomi.companion

import ai.nomi.data.ActivityRepository
import ai.nomi.data.AgentActivity
import ai.nomi.data.ChatRepository
import ai.nomi.data.CompanionPreferences
import ai.nomi.data.ListenerHandle
import ai.nomi.data.RealtimeSync
import ai.nomi.data.Session
import ai.nomi.data.SessionRepository
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.channels.BufferOverflow
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch

private const val NAV_BUFFER = 8
private const val CHAT_LINK_PREFIX = "/chat/"
private const val HOME_LINK = "/"

/**
 * Coordinates the companion surfaces (desktop Dynamic Island, Android Messenger chat-head overlay, and in-app floating
 * bubble) with [RealtimeSync], quick replies via [ChatRepository], user toggles, and deep-link navigation back into
 * the main `NomiApp` shell.
 */
class CompanionController(
    private val scope: CoroutineScope,
    sync: RealtimeSync,
    private val chatRepository: ChatRepository,
    private val preferences: CompanionPreferences,
    private val sessionRepository: SessionRepository? = null,
    activityRepository: ActivityRepository? = null,
    /** The crew roster, agent type to name; loaded when the companion opens. */
    private val loadCrewNames: suspend () -> Map<String, String> = { emptyMap() },
    /** The person's projects, for shortcuts; loaded when the companion opens. */
    private val loadProjects: suspend () -> List<CompanionProject> = { emptyList() },
    private val log: (String) -> Unit = {},
) {
    val stateMachine = CompanionStateMachine(
        scope = scope,
        companionState = sync.companionState,
        companionSignals = sync.companionSignals,
    )

    val uiState: StateFlow<CompanionUiState> = stateMachine.state

    val islandEnabled: StateFlow<Boolean> = preferences.islandEnabled
    val bubbleEnabled: StateFlow<Boolean> = preferences.bubbleEnabled

    private val mutableSnoozed = MutableStateFlow(false)
    val snoozed: StateFlow<Boolean> = mutableSnoozed.asStateFlow()

    private val mutableNavRequests = MutableSharedFlow<String>(
        extraBufferCapacity = NAV_BUFFER,
        onBufferOverflow = BufferOverflow.DROP_OLDEST,
    )

    /** Deep-link paths (`/chat/{id}`, `/notifications`, `/`) emitted when the person clicks "Open" in a companion. */
    val navigationRequests: SharedFlow<String> = mutableNavRequests.asSharedFlow()

    private val crewNames = MutableStateFlow<Map<String, String>>(emptyMap())
    private val projects = MutableStateFlow<List<CompanionProject>>(emptyList())

    /** The chat the person is talking to from the companion, and the question sent before its reply shows up. */
    private val conversationId = MutableStateFlow<String?>(null)
    private var conversationBinding: ListenerHandle? = null

    private val sessions: Flow<List<Session>> = sessionRepository?.observeAll() ?: flowOf(emptyList())
    private val active: Flow<List<AgentActivity>> = activityRepository?.observeActive() ?: flowOf(emptyList())

    @OptIn(ExperimentalCoroutinesApi::class)
    private val conversation: Flow<CompanionConversation?> = conversationId.flatMapLatest { id ->
        if (id == null) flowOf(null) else chatRepository.observe(id).map { conversationOf(id, it) }
    }

    /** The crew at work, recent chats, project shortcuts and the companion's own conversation. */
    val overview: StateFlow<CompanionOverview> = combine(sessions, active, crewNames, projects, conversation) { all, working, roster, shortcuts, talk ->
        CompanionOverview(
            crew = crewAtWork(working, all, roster),
            chats = recentChats(all, working),
            projects = shortcuts.take(COMPANION_PROJECTS_SHOWN),
            conversation = talk,
        )
    }.stateIn(scope, SharingStarted.Eagerly, CompanionOverview())

    /** Brings the roster and project shortcuts up to date; called as the companion opens. */
    fun refreshOverview() {
        scope.launch {
            runLogged("crew roster") { crewNames.value = loadCrewNames() }
            runLogged("projects") { projects.value = loadProjects() }
        }
    }

    /**
     * Asks Nomi [text] without leaving the companion: in the companion's current conversation, or [sessionId] when
     * given (replying to a chat the companion surfaced), or a new chat. The answer streams into [overview].
     */
    fun ask(text: String, sessionId: String? = null) {
        val question = text.trim()
        if (question.isEmpty()) return
        scope.launch {
            runLogged("ask") {
                val target = sessionId?.ifBlank { null } ?: conversationId.value ?: sessionRepository?.create().orEmpty()
                if (target.isBlank()) return@runLogged
                follow(target)
                chatRepository.send(target, question)
            }
        }
    }

    /** Ends the companion's conversation, so the next question starts a new chat. */
    fun newConversation() = follow(null)

    /** Where the companion's conversation lives in the app (the home screen when there is none). */
    fun conversationLink(): String = conversationId.value?.let { "$CHAT_LINK_PREFIX$it" } ?: HOME_LINK

    /** Opens the companion's conversation in the app. */
    fun openConversation() = openLink(conversationLink())

    private fun follow(sessionId: String?) {
        if (conversationId.value == sessionId) return
        conversationBinding?.remove()
        // Bound so the reply streams in over realtime even though no chat screen shows it.
        conversationBinding = sessionId?.let(chatRepository::bind)
        conversationId.value = sessionId
    }

    private suspend fun runLogged(what: String, block: suspend () -> Unit) {
        try {
            block()
        } catch (e: CancellationException) {
            throw e
        } catch (e: Exception) {
            log("companion: $what failed: $e")
        }
    }

    fun setIslandEnabled(enabled: Boolean) {
        preferences.setIslandEnabled(enabled)
    }

    fun setBubbleEnabled(enabled: Boolean) {
        preferences.setBubbleEnabled(enabled)
    }

    fun setSnoozed(snoozed: Boolean) {
        mutableSnoozed.value = snoozed
        stateMachine.setSnoozed(snoozed)
    }

    fun toggleSnoozed() {
        setSnoozed(!mutableSnoozed.value)
    }

    fun openLink(link: String) {
        stateMachine.collapse()
        mutableNavRequests.tryEmit(link)
    }

    fun openApp() {
        stateMachine.collapse()
        mutableNavRequests.tryEmit(HOME_LINK)
    }

    /** A reply typed into a companion card: asked in the card's chat (or a new one), answered in place. */
    fun sendQuickReply(sessionId: String, text: String) = ask(text, sessionId)
}
