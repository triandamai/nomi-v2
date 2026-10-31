package ai.nomi.data

import ai.nomi.network.ApiException

/** The backend's answer for a chat that isn't a project's. */
private const val HTTP_NOT_FOUND = 404
private const val ISSUES_OPEN = "open"
private const val ISSUES_CLOSED = "closed"

/** A file in a project. */
data class ProjectFileEntry(val path: String, val contentType: String, val sizeBytes: Long)

/** A project as its workspace shows it. [sessionId] is its chat. */
data class ProjectInfo(
    val id: String,
    val name: String,
    val description: String?,
    val status: String,
    val stack: String,
    val files: List<ProjectFileEntry>,
    val sessionId: String,
    val orgName: String,
    val openIssues: Long,
)

/** One publish. [status] is `queued`, `building`, `deploying`, `live`, `failed` or `superseded`. */
data class ProjectPublishInfo(
    val id: String,
    val status: String,
    val url: String?,
    val error: String?,
    val createdBy: String?,
    val createdAt: String,
)

/** Where a project is published; [canPublish] when this Nomi publishes and the person is the project's admin. */
data class ProjectDeployInfo(
    val canPublish: Boolean,
    val url: String?,
    val publishes: List<ProjectPublishInfo>,
    val backendSetupStep: String?,
    val backendSetupError: String?,
) {
    /** A publish still on its way. */
    val publishing: Boolean get() = publishes.firstOrNull()?.status in IN_FLIGHT

    private companion object {
        val IN_FLIGHT = setOf("queued", "building", "deploying")
    }
}

data class ProjectIssueInfo(
    val number: Int,
    val title: String,
    val body: String,
    val kind: String,
    val severity: String,
    val open: Boolean,
    val createdBy: String?,
    val createdAt: String,
)

data class ProjectIssueList(val issues: List<ProjectIssueInfo>, val open: Long, val closed: Long)

data class ProjectMemberInfo(
    val name: String,
    val email: String?,
    val avatarUrl: String?,
    val role: String,
    val isOwner: Boolean,
)

/** Who works on a project: everyone in the organization ([orgWide]) or only the people added. */
data class ProjectTeam(val orgWide: Boolean, val people: List<ProjectMemberInfo>)

/** One project's workspace, online: the project behind a chat, its files, publishes, issues and people. */
class ProjectWorkspaceRepository(private val engine: NomiEngine) {
    // Projects seen while the app runs, so reopening one shows its workspace at once.
    private val seen = mutableMapOf<String, ProjectInfo>()
    private val projectChats = mutableSetOf<String>()

    /** The project last seen for chat [sessionId], if any. Main thread only. */
    fun known(sessionId: String): ProjectInfo? = seen[sessionId]

    /** Whether chat [sessionId] is known to be a project's (from the projects list), before it's fetched. */
    fun isProjectChat(sessionId: String): Boolean = sessionId in projectChats || sessionId in seen

    /** Notes the chats of projects listed elsewhere (the projects page). Main thread only. */
    fun rememberProjectChats(sessionIds: Collection<String>) {
        projectChats += sessionIds
    }

    /** The project whose chat is [sessionId], or null for an ordinary chat. */
    suspend fun projectForChat(sessionId: String): ProjectInfo? = try {
        translated { engine.get().projectForChat(sessionId).toInfo() }.also { seen[sessionId] = it }
    } catch (notFound: ApiException) {
        if (notFound.status == HTTP_NOT_FOUND) null else throw notFound
    }

    /** A file's text (as UTF-8). */
    suspend fun fileText(projectId: String, path: String): String =
        translated { engine.get().projectFile(projectId, path).decodeToString() }

    suspend fun deploy(projectId: String): ProjectDeployInfo = translated {
        val overview = engine.get().projectDeploy(projectId)
        ProjectDeployInfo(
            canPublish = overview.available && overview.role == ROLE_ADMIN,
            url = overview.url,
            publishes = overview.publishes.map { ProjectPublishInfo(it.id, it.status, it.url, it.error, it.createdBy, it.createdAt) },
            backendSetupStep = overview.backendSetup?.step ?: overview.backendSetup?.state,
            backendSetupError = overview.backendSetup?.error,
        )
    }

    suspend fun publish(projectId: String) = translated { engine.get().publishProject(projectId) }

    /** Open issues, or closed ones when [closed]. */
    suspend fun issues(projectId: String, closed: Boolean = false): ProjectIssueList = translated {
        val list = engine.get().projectIssues(projectId, if (closed) ISSUES_CLOSED else ISSUES_OPEN)
        ProjectIssueList(list.issues.map { it.toInfo() }, list.counts.open, list.counts.closed)
    }

    suspend fun fileIssue(projectId: String, title: String, body: String, kind: String, severity: String): ProjectIssueInfo =
        translated { engine.get().fileProjectIssue(projectId, ai.nomi.sdk.NewProjectIssue(title, body, kind, severity)).toInfo() }

    suspend fun setIssueOpen(projectId: String, number: Int, open: Boolean): ProjectIssueInfo = translated {
        engine.get().setProjectIssueStatus(projectId, number, if (open) ISSUES_OPEN else ISSUES_CLOSED).toInfo()
    }

    suspend fun team(projectId: String): ProjectTeam = translated {
        val members = engine.get().projectMembers(projectId)
        val onProject = members.people.filter { members.access == ACCESS_ORG || it.projectRole != null || it.isOwner }
        ProjectTeam(
            orgWide = members.access == ACCESS_ORG,
            people = onProject.map {
                ProjectMemberInfo(it.name, it.email, it.avatarUrl, it.projectRole ?: it.orgRole, it.isOwner)
            },
        )
    }

    private companion object {
        const val ROLE_ADMIN = "admin"
        const val ACCESS_ORG = "org"
    }
}

private fun ai.nomi.sdk.ProjectDetail.toInfo() = ProjectInfo(
    id = id,
    name = name,
    description = description,
    status = status,
    stack = stack,
    files = files.map { ProjectFileEntry(it.path, it.contentType, it.sizeBytes) },
    sessionId = sessionId,
    orgName = orgName,
    openIssues = openIssues,
)

private fun ai.nomi.sdk.ProjectIssue.toInfo() =
    ProjectIssueInfo(number, title, body, kind, severity, status == ISSUES_OPEN, createdBy, createdAt)
