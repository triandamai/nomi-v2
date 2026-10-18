package ai.nomi.feature.home

import ai.nomi.core.STATE_STOP_TIMEOUT_MS
import ai.nomi.data.ChatRepository
import ai.nomi.data.Session
import ai.nomi.data.SessionRepository
import ai.nomi.network.CrewMemberDto
import ai.nomi.network.HomeApi
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.launch
import kotlinx.datetime.TimeZone
import kotlinx.datetime.toLocalDateTime
import kotlin.time.Clock

/** Chats shown on Home (the stage layout's `recent` block); the full list is the Chats screen. */
const val HOME_RECENT_CHATS = 5

/** Crew members in Home's strip (the stage layout's `crew` block). */
const val HOME_CREW_LIMIT = 8

private const val MESSAGE_START_FAILED = "Could not start a new chat."

// `HomeGreeting.svelte`: the hour decides the greeting.
private const val STILL_UP_BEFORE = 5
private const val MORNING_BEFORE = 12
private const val AFTERNOON_BEFORE = 18
private const val ONE = 1

/** The person's local hour (0–23), injected so the greeting is testable. */
typealias HourOfDay = () -> Int

val SystemHourOfDay: HourOfDay = { Clock.System.now().toLocalDateTime(TimeZone.currentSystemDefault()).hour }

/** "Afternoon, Ana." — the web's greeting: time of day, then the first name when there is one. */
fun greeting(hour: Int, displayName: String?): String {
    val part = when {
        hour < STILL_UP_BEFORE -> "Still up"
        hour < MORNING_BEFORE -> "Morning"
        hour < AFTERNOON_BEFORE -> "Afternoon"
        else -> "Evening"
    }
    val first = displayName?.trim()?.split(' ')?.firstOrNull()?.takeIf { it.isNotEmpty() }
    return if (first != null) "$part, $first." else "$part."
}

/** The line under the greeting. */
fun subtitle(workingChats: Int): String = when {
    workingChats <= 0 -> "What should we get done?"
    workingChats == ONE -> "Your crew is working in 1 chat."
    else -> "Your crew is working in $workingChats chats."
}

private val StateOrder = mapOf("working" to 0, "waiting" to 1, "done" to 2, "idle" to 3)

/** `crewByActivity`: working, then waiting on you, then done, then ready; ties keep the roster's order (Nomi first). */
fun crewByActivity(crew: List<CrewMemberDto>): List<CrewMemberDto> =
    crew.withIndex().sortedWith(compareBy({ StateOrder[it.value.state] ?: StateOrder.size }, { it.index })).map { it.value }

data class HomeUiState(
    val greeting: String = "",
    val subtitle: String = subtitle(0),
    val crew: List<CrewMemberDto> = emptyList(),
    val recent: List<Session> = emptyList(),
    val draft: String = "",
    val sending: Boolean = false,
    val error: String? = null,
) {
    val workingCount: Int get() = crew.count { it.state == "working" }
}

private data class HomeExtras(val displayName: String? = null, val crew: List<CrewMemberDto> = emptyList())

class HomeViewModel(
    private val sessions: SessionRepository,
    private val chats: ChatRepository,
    private val home: HomeApi,
    private val hourOfDay: HourOfDay = SystemHourOfDay,
    private val log: (String) -> Unit = {},
) : ViewModel() {
    private val form = MutableStateFlow(HomeUiState())
    private val extras = MutableStateFlow(HomeExtras())

    val state: StateFlow<HomeUiState> =
        combine(form, extras, sessions.observeAll().map { it.take(HOME_RECENT_CHATS) }) { f, x, recent ->
            val crew = crewByActivity(x.crew).take(HOME_CREW_LIMIT)
            f.copy(
                greeting = greeting(hourOfDay(), x.displayName),
                subtitle = subtitle(crew.count { it.state == "working" }),
                crew = crew,
                recent = recent,
            )
        }.stateIn(viewModelScope, SharingStarted.WhileSubscribed(STATE_STOP_TIMEOUT_MS), HomeUiState(greeting = greeting(hourOfDay(), null)))

    init {
        viewModelScope.launch { attempt("refresh chats") { sessions.refresh() } }
        viewModelScope.launch { attempt("load profile") { extras.value = extras.value.copy(displayName = home.profile().displayName) } }
        viewModelScope.launch { attempt("load crew") { extras.value = extras.value.copy(crew = home.crew()) } }
    }

    fun onDraftChange(value: String) {
        form.value = form.value.copy(draft = value, error = null)
    }

    /** Puts a suggestion in the composer (the person still sends it), like the web's suggestion chips. */
    fun useSuggestion(text: String) = onDraftChange(text)

    /** Creates a chat, sends the draft as its first message, then hands the new chat's id to [onStarted]. */
    fun startChat(onStarted: (sessionId: String) -> Unit) {
        val current = form.value
        val text = current.draft.trim()
        if (text.isEmpty() || current.sending) return
        form.value = current.copy(sending = true, error = null)
        viewModelScope.launch {
            try {
                val id = sessions.create()
                chats.send(id, text)
                form.value = form.value.copy(draft = "", sending = false)
                onStarted(id)
                attempt("refresh chats") { sessions.refresh() }
            } catch (e: CancellationException) {
                throw e
            } catch (e: Exception) {
                log("home: starting a chat failed: $e")
                form.value = form.value.copy(sending = false, error = MESSAGE_START_FAILED)
            }
        }
    }

    private suspend fun attempt(what: String, block: suspend () -> Unit) {
        try {
            block()
        } catch (e: CancellationException) {
            throw e
        } catch (e: Exception) {
            log("home: $what failed: $e")
        }
    }
}
