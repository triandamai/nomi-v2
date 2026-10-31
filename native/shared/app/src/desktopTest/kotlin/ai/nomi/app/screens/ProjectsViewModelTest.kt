package ai.nomi.app.screens

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
}
