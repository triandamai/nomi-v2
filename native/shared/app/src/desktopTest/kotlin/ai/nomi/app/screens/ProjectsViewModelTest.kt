package ai.nomi.app.screens

import ai.nomi.app.screens.projects.ProjectFilter
import ai.nomi.app.screens.projects.ProjectsUiState
import ai.nomi.app.screens.projects.ProjectsViewModel
import ai.nomi.data.ProjectsRepository
import ai.nomi.data.SessionScope
import ai.nomi.network.ProjectDto
import ai.nomi.network.ProjectsApi
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.filterIsInstance
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.UnconfinedTestDispatcher
import kotlinx.coroutines.test.resetMain
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class FakeProjectsApi : ProjectsApi {
    val created = mutableListOf<String?>()
    var nextCreate: Result<ai.nomi.network.CreatedProjectDto> = Result.success(ai.nomi.network.CreatedProjectDto("p-new", "s-new"))

    override suspend fun create(orgId: String?): ai.nomi.network.CreatedProjectDto {
        created += orgId
        return nextCreate.getOrThrow()
    }

    var projects = listOf(
        ProjectDto(
            id = "p1",
            sessionId = "s1",
            name = "Project Alpha",
            description = "Alpha desc",
            status = "ready",
            createdBy = "Ana",
            openIssues = 2,
        ),
        ProjectDto(
            id = "p2",
            sessionId = "s2",
            name = "Project Beta",
            description = "Beta desc",
            status = "building",
            createdBy = "Bob",
            openIssues = 0,
        )
    )

    override suspend fun list(orgId: String?): List<ProjectDto> = projects
}

@OptIn(ExperimentalCoroutinesApi::class)
class ProjectsViewModelTest {
    @BeforeTest
    fun setUp() {
        Dispatchers.setMain(UnconfinedTestDispatcher())
    }

    @AfterTest
    fun tearDown() {
        Dispatchers.resetMain()
    }

    @Test
    fun loadProjectsSetsReadyState() = runTest {
        val fakeApi = FakeProjectsApi()
        val session = SessionScope(backgroundScope).also { it.open() }
        val repo = ProjectsRepository(fakeApi, session)
        var projectChats = emptyList<String>()
        val vm = ProjectsViewModel(repo, onProjectChats = { projectChats = it })

        val state = vm.uiState.filterIsInstance<ProjectsUiState.Ready>().first()
        assertEquals(listOf("s1", "s2"), projectChats)
        assertEquals(2, state.projects.size)
        assertEquals("Project Alpha", state.projects[0].name)
        assertEquals("ready", state.projects[0].status)
    }

    @Test
    fun searchFiltersProjects() = runTest {
        val fakeApi = FakeProjectsApi()
        val session = SessionScope(backgroundScope).also { it.open() }
        val repo = ProjectsRepository(fakeApi, session)
        val vm = ProjectsViewModel(repo)

        vm.uiState.filterIsInstance<ProjectsUiState.Ready>().first()
        vm.setSearchQuery("Beta")
        val state = vm.uiState.value as ProjectsUiState.Ready
        assertEquals(1, state.filteredProjects.size)
        assertEquals("Project Beta", state.filteredProjects[0].name)
    }

    @Test
    fun filtersCountAndNarrowByState() = runTest {
        val session = SessionScope(backgroundScope).also { it.open() }
        val vm = ProjectsViewModel(ProjectsRepository(FakeProjectsApi(), session))
        val ready = vm.uiState.filterIsInstance<ProjectsUiState.Ready>().first()

        // Alpha is "ready" but has open issues: it needs attention, not live.
        assertEquals(2, ready.count(ProjectFilter.All))
        assertEquals(0, ready.count(ProjectFilter.Live))
        assertEquals(1, ready.count(ProjectFilter.Building))
        assertEquals(1, ready.count(ProjectFilter.Attention))
        vm.setFilter(ProjectFilter.Attention)
        assertEquals(listOf("Project Alpha"), (vm.uiState.value as ProjectsUiState.Ready).filteredProjects.map { it.name })
    }

    @Test
    fun sortTogglesBetweenUpdatedAndName() = runTest {
        val api = FakeProjectsApi().apply {
            projects = listOf(
                ProjectDto(id = "a", name = "Zebra", updatedAt = "2026-11-07T10:00:00Z"),
                ProjectDto(id = "b", name = "apple", updatedAt = "2026-11-01T10:00:00Z"),
            )
        }
        val session = SessionScope(backgroundScope).also { it.open() }
        val vm = ProjectsViewModel(ProjectsRepository(api, session))
        vm.uiState.filterIsInstance<ProjectsUiState.Ready>().first()

        assertEquals(listOf("Zebra", "apple"), (vm.uiState.value as ProjectsUiState.Ready).filteredProjects.map { it.name })
        vm.toggleSort()
        assertEquals(listOf("apple", "Zebra"), (vm.uiState.value as ProjectsUiState.Ready).filteredProjects.map { it.name })
    }

    @Test
    fun startBuildingCreatesAProjectSendsThePromptAndOpensIt() = runTest {
        val api = FakeProjectsApi()
        val session = SessionScope(backgroundScope).also { it.open() }
        val sent = mutableListOf<Pair<String, String>>()
        var opened: String? = null
        val vm = ProjectsViewModel(ProjectsRepository(api, session), sendFirstMessage = { id, text -> sent += id to text })
        vm.uiState.filterIsInstance<ProjectsUiState.Ready>().first()

        vm.setPrompt("  a booking page  ")
        vm.startBuilding { opened = it }
        vm.newProject.first { !it.starting }

        assertEquals(listOf<String?>(null), api.created)
        assertEquals(listOf("s-new" to "a booking page"), sent)
        assertEquals("s-new", opened)
        assertEquals("", vm.newProject.value.prompt)
    }

    @Test
    fun aProjectThatCantStartSaysSo() = runTest {
        val api = FakeProjectsApi().apply { nextCreate = Result.failure(IllegalStateException("409")) }
        val session = SessionScope(backgroundScope).also { it.open() }
        var opened: String? = null
        val vm = ProjectsViewModel(ProjectsRepository(api, session))
        vm.uiState.filterIsInstance<ProjectsUiState.Ready>().first()

        vm.setPrompt("x")
        vm.startBuilding { opened = it }
        vm.newProject.first { !it.starting }

        assertEquals(null, opened)
        assertEquals(false, vm.newProject.value.starting)
        assertEquals("Couldn't start a new project. Try again.", vm.newProject.value.error)
        assertEquals("x", vm.newProject.value.prompt)
    }
}
