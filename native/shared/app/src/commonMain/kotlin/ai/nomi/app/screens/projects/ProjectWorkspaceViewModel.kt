package ai.nomi.app.screens.projects

import ai.nomi.data.ProjectDeployInfo
import ai.nomi.data.ProjectInfo
import ai.nomi.data.ProjectIssueList
import ai.nomi.data.ProjectTeam
import ai.nomi.data.ProjectWorkspaceRepository
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch

/** How often a publish on its way is checked on. */
private const val PUBLISH_POLL_MS = 3_000L
private const val LOAD_FAILED = "Couldn't load this. Check your connection and try again."
private const val PUBLISH_FAILED = "Couldn't start publishing. Try again."
private const val ISSUE_FAILED = "Couldn't save the issue. Try again."

/** Whether a chat is a project's: unknown until asked, then a project or not. */
sealed interface ProjectLookup {
    data object Unknown : ProjectLookup
    data object NotAProject : ProjectLookup
    data class Project(val project: ProjectInfo) : ProjectLookup
}

/** Something loaded online: on its way, there, or failed with a reason to show. */
sealed interface Loadable<out T> {
    data object Loading : Loadable<Nothing>
    data class Ready<T>(val value: T) : Loadable<T>
    data class Failed(val message: String) : Loadable<Nothing>
}

/** The file open in the Code tab. */
data class OpenFile(val path: String, val text: Loadable<String>)

/**
 * A project's workspace around its chat ([sessionId]): the project itself, where it's published, its issues and
 * people, and the file open in the Code tab. Everything is online; the chat itself stays the chat's own.
 */
class ProjectWorkspaceViewModel(
    val sessionId: String,
    private val repo: ProjectWorkspaceRepository,
    private val log: (String) -> Unit = {},
) : ViewModel() {
    private val mutableLookup = MutableStateFlow(repo.known(sessionId)?.let(ProjectLookup::Project) ?: ProjectLookup.Unknown)
    val lookup: StateFlow<ProjectLookup> = mutableLookup.asStateFlow()

    private val mutableDeploy = MutableStateFlow<Loadable<ProjectDeployInfo>>(Loadable.Loading)
    val deploy: StateFlow<Loadable<ProjectDeployInfo>> = mutableDeploy.asStateFlow()

    private val mutableIssues = MutableStateFlow<Loadable<ProjectIssueList>>(Loadable.Loading)
    val issues: StateFlow<Loadable<ProjectIssueList>> = mutableIssues.asStateFlow()

    private val mutableShowClosed = MutableStateFlow(false)
    val showClosed: StateFlow<Boolean> = mutableShowClosed.asStateFlow()

    private val mutableTeam = MutableStateFlow<Loadable<ProjectTeam>>(Loadable.Loading)
    val team: StateFlow<Loadable<ProjectTeam>> = mutableTeam.asStateFlow()

    private val mutableFile = MutableStateFlow<OpenFile?>(null)
    val file: StateFlow<OpenFile?> = mutableFile.asStateFlow()

    private val mutableNotice = MutableStateFlow<String?>(null)

    /** A failed action, until it's shown. */
    val notice: StateFlow<String?> = mutableNotice.asStateFlow()

    private var polling: Job? = null

    init {
        viewModelScope.launch {
            val found = attempt("project") { repo.projectForChat(sessionId) }
            mutableLookup.value = when {
                found != null -> ProjectLookup.Project(found)
                mutableLookup.value is ProjectLookup.Project -> mutableLookup.value
                // Offline or refused: an ordinary chat is the safe reading, and the chat itself still works.
                else -> ProjectLookup.NotAProject
            }
            if (found != null) refreshDeploy()
        }
    }

    private val projectId: String? get() = (mutableLookup.value as? ProjectLookup.Project)?.project?.id

    fun refreshDeploy() {
        val id = projectId ?: return
        viewModelScope.launch {
            val overview = attempt("deploy") { repo.deploy(id) }
            mutableDeploy.value = overview?.let { Loadable.Ready(it) } ?: Loadable.Failed(LOAD_FAILED)
            if (overview?.publishing == true) followPublish(id)
        }
    }

    fun publish() {
        val id = projectId ?: return
        viewModelScope.launch {
            val started = attempt("publish") { repo.publish(id) } != null
            if (!started) {
                mutableNotice.value = PUBLISH_FAILED
                return@launch
            }
            mutableDeploy.value = repo.runCatchingDeploy(id)
            followPublish(id)
        }
    }

    /** Checks on a publish until it's live or failed. */
    private fun followPublish(id: String) {
        if (polling?.isActive == true) return
        polling = viewModelScope.launch {
            while (true) {
                delay(PUBLISH_POLL_MS)
                val next = repo.runCatchingDeploy(id)
                mutableDeploy.value = next
                if ((next as? Loadable.Ready)?.value?.publishing != true) break
            }
        }
    }

    fun refreshIssues() {
        val id = projectId ?: return
        viewModelScope.launch {
            mutableIssues.value = Loadable.Loading
            val list = attempt("issues") { repo.issues(id, closed = mutableShowClosed.value) }
            mutableIssues.value = list?.let { Loadable.Ready(it) } ?: Loadable.Failed(LOAD_FAILED)
        }
    }

    fun showClosedIssues(closed: Boolean) {
        if (mutableShowClosed.value == closed) return
        mutableShowClosed.value = closed
        refreshIssues()
    }

    fun fileIssue(title: String, body: String, kind: String, severity: String) {
        val id = projectId ?: return
        viewModelScope.launch {
            val filed = attempt("file issue") { repo.fileIssue(id, title.trim(), body.trim(), kind, severity) }
            if (filed == null) mutableNotice.value = ISSUE_FAILED else refreshIssuesAndCount()
        }
    }

    fun setIssueOpen(number: Int, open: Boolean) {
        val id = projectId ?: return
        viewModelScope.launch {
            val changed = attempt("issue status") { repo.setIssueOpen(id, number, open) }
            if (changed == null) mutableNotice.value = ISSUE_FAILED else refreshIssuesAndCount()
        }
    }

    private suspend fun refreshIssuesAndCount() {
        refreshIssues()
        val found = attempt("project") { repo.projectForChat(sessionId) } ?: return
        mutableLookup.value = ProjectLookup.Project(found)
    }

    fun refreshTeam() {
        val id = projectId ?: return
        viewModelScope.launch {
            mutableTeam.value = Loadable.Loading
            val team = attempt("team") { repo.team(id) }
            mutableTeam.value = team?.let { Loadable.Ready(it) } ?: Loadable.Failed(LOAD_FAILED)
        }
    }

    fun openFile(path: String) {
        val id = projectId ?: return
        mutableFile.value = OpenFile(path, Loadable.Loading)
        viewModelScope.launch {
            val text = attempt("file") { repo.fileText(id, path) }
            mutableFile.update { open ->
                if (open?.path != path) open else open.copy(text = text?.let { Loadable.Ready(it) } ?: Loadable.Failed(LOAD_FAILED))
            }
        }
    }

    fun closeFile() {
        mutableFile.value = null
    }

    fun noticeShown() {
        mutableNotice.value = null
    }

    private suspend fun <T> attempt(what: String, block: suspend () -> T): T? = try {
        block()
    } catch (e: CancellationException) {
        throw e
    } catch (e: Exception) {
        log("project workspace: $what failed: $e")
        null
    }

    private suspend fun ProjectWorkspaceRepository.runCatchingDeploy(id: String): Loadable<ProjectDeployInfo> =
        attempt("deploy") { deploy(id) }?.let { Loadable.Ready(it) } ?: Loadable.Failed(LOAD_FAILED)
}
