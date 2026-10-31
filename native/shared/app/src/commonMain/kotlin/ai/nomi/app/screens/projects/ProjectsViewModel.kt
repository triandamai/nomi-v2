package ai.nomi.app.screens.projects

import ai.nomi.data.ProjectsRepository
import ai.nomi.network.ProjectDto
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

sealed interface ProjectsUiState {
    data object Loading : ProjectsUiState
    data class Ready(
        val projects: List<ProjectDto>,
        val query: String = "",
    ) : ProjectsUiState {
        val filteredProjects: List<ProjectDto>
            get() = if (query.isBlank()) {
                projects
            } else {
                projects.filter {
                    it.name.contains(query, ignoreCase = true) ||
                        (it.description?.contains(query, ignoreCase = true) == true) ||
                        it.createdBy.contains(query, ignoreCase = true)
                }
            }
    }
    data class Error(val message: String) : ProjectsUiState
}

class ProjectsViewModel(
    private val repo: ProjectsRepository,
    /** Told the chats of the projects listed, so they open straight into the project workspace. */
    private val onProjectChats: (List<String>) -> Unit = {},
) : ViewModel() {
    private val mutableState = MutableStateFlow<ProjectsUiState>(ProjectsUiState.Loading)
    val uiState: StateFlow<ProjectsUiState> = mutableState.asStateFlow()

    init {
        refresh()
    }

    fun refresh() {
        viewModelScope.launch {
            mutableState.value = ProjectsUiState.Loading
            repo.list()
                .onSuccess { projects ->
                    onProjectChats(projects.map { it.sessionId }.filter { it.isNotBlank() })
                    mutableState.value = ProjectsUiState.Ready(projects = projects)
                }
                .onFailure { e ->
                    mutableState.value = ProjectsUiState.Error(e.message ?: "Failed to load projects")
                }
        }
    }

    fun setSearchQuery(query: String) {
        val current = mutableState.value
        if (current is ProjectsUiState.Ready) {
            mutableState.value = current.copy(query = query)
        }
    }
}
