package ai.nomi.app.screens.projects

import ai.nomi.data.ProjectsRepository
import ai.nomi.network.ProjectDto
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

/** The web's filter tabs (`projects/+page.svelte`): all, live, building, needs attention. */
enum class ProjectFilter { All, Live, Building, Attention }

/** The sort button's two orders. */
enum class ProjectSort { Updated, Name }

/** How a project's card reads it (`projectStatusKind`): open issues need attention first. */
enum class ProjectState { Live, Working, Failed, Idle }

fun ProjectDto.state(): ProjectState = when {
    openIssues > 0 -> ProjectState.Failed
    status == "ready" -> ProjectState.Live
    status == "building" -> ProjectState.Working
    else -> ProjectState.Idle
}

/** The published address without its scheme, or null before the first publish. */
fun ProjectDto.host(): String? = deployUrl?.removePrefix("https://")?.removePrefix("http://")?.trimEnd('/')

sealed interface ProjectsUiState {
    data object Loading : ProjectsUiState
    data class Ready(
        val projects: List<ProjectDto>,
        val query: String = "",
        val filter: ProjectFilter = ProjectFilter.All,
        val sort: ProjectSort = ProjectSort.Updated,
    ) : ProjectsUiState {
        fun count(filter: ProjectFilter): Int = projects.count { it.matches(filter) }

        val filteredProjects: List<ProjectDto>
            get() {
                val shown = projects.filter { it.matches(filter) && it.matches(query) }
                return when (sort) {
                    ProjectSort.Updated -> shown.sortedByDescending { it.updatedAt }
                    ProjectSort.Name -> shown.sortedBy { it.name.lowercase() }
                }
            }
    }
    data class Error(val message: String) : ProjectsUiState
}

private fun ProjectDto.matches(filter: ProjectFilter): Boolean = when (filter) {
    ProjectFilter.All -> true
    ProjectFilter.Live -> state() == ProjectState.Live
    ProjectFilter.Building -> state() == ProjectState.Working
    ProjectFilter.Attention -> state() == ProjectState.Failed
}

private fun ProjectDto.matches(query: String): Boolean =
    query.isBlank() ||
        name.contains(query, ignoreCase = true) ||
        description?.contains(query, ignoreCase = true) == true ||
        createdBy.contains(query, ignoreCase = true)

/** The "describe an app" card: what's typed, and whether the project is being started. */
data class NewProjectState(val prompt: String = "", val starting: Boolean = false, val error: String? = null)

private const val MESSAGE_START_FAILED = "Couldn't start a new project. Try again."

class ProjectsViewModel(
    private val repo: ProjectsRepository,
    /** Told the chats of the projects listed, so they open straight into the project workspace. */
    private val onProjectChats: (List<String>) -> Unit = {},
    /** Sends a new project's first message (what was typed) so Koda starts on it before the workspace opens. */
    private val sendFirstMessage: suspend (sessionId: String, text: String) -> Unit = { _, _ -> },
) : ViewModel() {
    private val mutableState = MutableStateFlow<ProjectsUiState>(ProjectsUiState.Loading)
    val uiState: StateFlow<ProjectsUiState> = mutableState.asStateFlow()

    private val mutableNewProject = MutableStateFlow(NewProjectState())
    val newProject: StateFlow<NewProjectState> = mutableNewProject.asStateFlow()

    init {
        refresh()
    }

    fun refresh() {
        viewModelScope.launch {
            val previous = mutableState.value as? ProjectsUiState.Ready
            if (previous == null) mutableState.value = ProjectsUiState.Loading
            repo.list()
                .onSuccess { projects ->
                    onProjectChats(projects.map { it.sessionId }.filter { it.isNotBlank() })
                    mutableState.value = previous?.copy(projects = projects) ?: ProjectsUiState.Ready(projects = projects)
                }
                .onFailure { e ->
                    if (previous == null) mutableState.value = ProjectsUiState.Error(e.message ?: "Failed to load projects")
                }
        }
    }

    private fun updateReady(change: (ProjectsUiState.Ready) -> ProjectsUiState.Ready) =
        mutableState.update { if (it is ProjectsUiState.Ready) change(it) else it }

    fun setSearchQuery(query: String) = updateReady { it.copy(query = query) }

    fun setFilter(filter: ProjectFilter) = updateReady { it.copy(filter = filter) }

    fun toggleSort() = updateReady { it.copy(sort = if (it.sort == ProjectSort.Updated) ProjectSort.Name else ProjectSort.Updated) }

    fun setPrompt(prompt: String) = mutableNewProject.update { it.copy(prompt = prompt, error = null) }

    /**
     * "Start building" (the web's `createProject`): a new project, its first message sent from what was typed, then
     * [onStarted] with its chat. The engine's outbox keeps a first message it couldn't send, to retry in the chat.
     */
    fun startBuilding(onStarted: (sessionId: String) -> Unit) {
        val current = mutableNewProject.value
        if (current.starting) return
        mutableNewProject.update { it.copy(starting = true, error = null) }
        viewModelScope.launch {
            val created = repo.create().getOrElse {
                mutableNewProject.update { s -> s.copy(starting = false, error = MESSAGE_START_FAILED) }
                return@launch
            }
            onProjectChats(listOf(created.sessionId))
            val prompt = current.prompt.trim()
            if (prompt.isNotEmpty()) {
                try {
                    sendFirstMessage(created.sessionId, prompt)
                } catch (e: CancellationException) {
                    throw e
                } catch (_: Exception) {
                    // The project exists either way; its chat shows the message as not sent, with Retry.
                }
            }
            mutableNewProject.value = NewProjectState()
            onStarted(created.sessionId)
            refresh()
        }
    }
}
