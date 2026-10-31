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
    /** `drizzle` or `relay`: where the app keeps its data. */
    val backend: String = DEFAULT_BACKEND,
) {
    val relay: Boolean get() = backend == RELAY_BACKEND
}

private const val DEFAULT_BACKEND = "drizzle"

/** One publish. [status] is `running`, `succeeded` or `failed`. */
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
        val IN_FLIGHT = setOf("running")
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
    backend = backend,
)

private fun ai.nomi.sdk.ProjectIssue.toInfo() =
    ProjectIssueInfo(number, title, body, kind, severity, status == ISSUES_OPEN, createdBy, createdAt)

/** The project's production address; [hostname] is null until the first publish gives it one. */
data class ProjectAddress(val hostname: String?, val subdomain: String?, val url: String?, val canEdit: Boolean)

/** Whether a name is free; when not, why and free names ([suggestions] are subdomains) instead. */
data class AddressCheck(val available: Boolean, val hostname: String?, val message: String?, val suggestions: List<String>)

/** A rename's outcome: the new hostname, or why it was refused. */
sealed interface AddressChange {
    data class Changed(val hostname: String) : AddressChange
    data class Refused(val check: AddressCheck) : AddressChange
}

data class ContainerStat(val service: String, val cpuPercent: Double?, val memoryBytes: Long?, val memoryLimitBytes: Long?)

/** Traffic totals for a period; null where the backend can't tell. */
data class TrafficTotals(val visitors: Double?, val requests: Double?, val errors5xx: Double?, val errors4xx: Double?, val p50Ms: Double?, val p95Ms: Double?)

/** Production's traffic over [range], requests per bucket for a spark line, against the period before. */
data class ProjectTrafficInfo(val range: String, val requestsPerBucket: List<Double>, val totals: TrafficTotals, val previous: TrafficTotals)

/** How production is doing: its containers, traffic and latest log lines. */
data class ProjectHealth(val containers: List<ContainerStat>, val traffic: ProjectTrafficInfo?, val logLines: List<String>, val logNote: String?)

data class DataTable(val name: String, val rows: Long?, val detail: String?)

/** A project's data: its tables, or a Relay backend still being set up ([setupState], [setupError]). */
data class ProjectData(val relay: Boolean, val tables: List<DataTable>, val setupState: String?, val setupError: String?)

data class DataQueryResult(val columns: List<String>, val rows: List<List<String?>>, val truncated: Boolean, val rowsAffected: Long?, val tookMs: Double)

data class ShowcaseInfo(val showcase: Boolean, val blurb: String?, val canEdit: Boolean)

data class TransferTarget(val id: String, val name: String, val personal: Boolean, val ownerName: String?)

private const val RELAY_BACKEND = "relay"

/** A project's tools: its address, production's health, its data, and its settings. All online. */
class ProjectToolsRepository(private val engine: NomiEngine) {
    suspend fun address(projectId: String): ProjectAddress = translated {
        engine.get().projectDomain(projectId).let { ProjectAddress(it.hostname, it.subdomain, it.url, it.canEdit) }
    }

    suspend fun checkAddress(projectId: String, subdomain: String): AddressCheck =
        translated { engine.get().checkProjectDomain(projectId, subdomain).toCheck() }

    suspend fun changeAddress(projectId: String, subdomain: String): AddressChange = translated {
        when (val change = engine.get().changeProjectDomain(projectId, subdomain)) {
            is ai.nomi.sdk.DomainChange.Changed -> AddressChange.Changed(change.hostname)
            is ai.nomi.sdk.DomainChange.Refused -> AddressChange.Refused(change.check.toCheck())
        }
    }

    /** Containers, traffic over [range] and logs; parts the backend can't give (nothing published) are left empty. */
    suspend fun health(projectId: String, range: String): ProjectHealth = translated {
        val client = engine.get()
        val containers = runCatching { client.projectUsage(projectId) }.getOrDefault(emptyList())
        val traffic = runCatching { client.projectTraffic(projectId, range) }.getOrNull()
        val logs = runCatching { client.projectLogs(projectId) }.getOrNull()
        ProjectHealth(
            containers = containers.map { ContainerStat(it.service, it.cpuPercent, it.memoryUsageBytes, it.memoryLimitBytes) },
            traffic = traffic?.let { t ->
                ProjectTrafficInfo(t.range, t.buckets.map { it.requests ?: 0.0 }, t.totals.toTotals(), t.previousTotals.toTotals())
            },
            logLines = logs?.lines.orEmpty(),
            logNote = logs?.note,
        )
    }

    suspend fun takeOffline(projectId: String) = translated { engine.get().takeProjectOffline(projectId) }

    /** The project's tables in [production] or its development copy; a Relay backend when [relay]. */
    suspend fun data(projectId: String, relay: Boolean, production: Boolean): ProjectData = translated {
        val source = source(production)
        if (relay) {
            val overview = engine.get().projectRelay(projectId, source)
            ProjectData(
                relay = overview.backend == RELAY_BACKEND,
                tables = overview.tables.map { table ->
                    DataTable(table.name, rows = null, detail = "${table.columns.size} columns" + if (table.rlsEnabled) " · RLS on" else " · RLS off")
                },
                setupState = overview.setup?.state,
                setupError = overview.setup?.error,
            )
        } else {
            val tables = engine.get().projectTables(projectId, source)
            ProjectData(false, tables.map { DataTable(it.name, it.rowsEstimate, it.schema) }, null, null)
        }
    }

    suspend fun retryRelay(projectId: String, production: Boolean) =
        translated { engine.get().retryProjectRelay(projectId, source(production)) }

    suspend fun query(projectId: String, sql: String, relay: Boolean, production: Boolean): DataQueryResult = translated {
        val result = engine.get().queryProjectData(projectId, sql, source(production), relay)
        DataQueryResult(result.columns, result.rows, result.truncated, result.rowsAffected, result.tookMs)
    }

    suspend fun showcase(projectId: String): ShowcaseInfo =
        translated { engine.get().projectShowcase(projectId).let { ShowcaseInfo(it.showcase, it.blurb, it.canEdit) } }

    suspend fun setShowcase(projectId: String, showcase: Boolean, blurb: String?): ShowcaseInfo = translated {
        engine.get().setProjectShowcase(projectId, showcase, blurb).let { ShowcaseInfo(it.showcase, it.blurb, it.canEdit) }
    }

    suspend fun transferTargets(projectId: String, query: String): List<TransferTarget> = translated {
        engine.get().transferOrgs(projectId, query).map { TransferTarget(it.id, it.name, it.isPersonal, it.ownerName) }
    }

    suspend fun transfer(projectId: String, orgId: String, confirmName: String) =
        translated { engine.get().transferProject(projectId, orgId, confirmName) }

    suspend fun delete(projectId: String) = translated { engine.get().deleteProject(projectId) }

    private fun source(production: Boolean) =
        if (production) ai.nomi.sdk.DataSource.PRODUCTION else ai.nomi.sdk.DataSource.DEVELOPMENT
}

private fun ai.nomi.sdk.DomainCheck.toCheck() = AddressCheck(available, hostname, message, suggestions.map { it.subdomain })

private fun ai.nomi.sdk.TrafficCounters.toTotals() = TrafficTotals(visitors, requests, errors5xx, errors4xx, p50Ms, p95Ms)
