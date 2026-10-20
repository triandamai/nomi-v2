package ai.nomi.data

import ai.nomi.network.ProjectDto
import ai.nomi.network.ProjectsApi
import ai.nomi.network.UsageApi
import ai.nomi.network.UsageBriefDto
import ai.nomi.network.UsagePlanDto
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class FakeUsageApi : UsageApi {
    var briefResult = UsageBriefDto(
        plan = UsagePlanDto(name = "Pro", monthlyTokens = 1_000_000),
        month = "2026-10",
        tokensUsed = 450_000,
        canUpgrade = false,
    )

    override suspend fun brief(): UsageBriefDto = briefResult
}

class FakeProjectsApi : ProjectsApi {
    var projectsList = listOf(
        ProjectDto(
            id = "proj-1",
            sessionId = "sess-1",
            name = "Project Phoenix",
            description = "Native rework",
            status = "active",
            createdAt = "2026-10-01T00:00:00Z",
            updatedAt = "2026-10-19T00:00:00Z",
            createdBy = "user-1",
            openIssues = 3,
        )
    )

    override suspend fun list(orgId: String?): List<ProjectDto> = projectsList
}

class ExtraRepositoriesTest {
    @Test
    fun usageRepositoryReturnsBriefSuccessfully() = runTest {
        val fakeApi = FakeUsageApi()
        val session = backgroundScope.openSession()
        val repo = UsageRepository(fakeApi, session)

        val result = repo.getBrief()
        assertTrue(result.isSuccess)
        assertEquals("Pro", result.getOrNull()?.plan?.name)
        assertEquals(450_000L, result.getOrNull()?.tokensUsed)
    }

    @Test
    fun projectsRepositoryReturnsListSuccessfully() = runTest {
        val fakeApi = FakeProjectsApi()
        val session = backgroundScope.openSession()
        val repo = ProjectsRepository(fakeApi, session)

        val result = repo.list()
        assertTrue(result.isSuccess)
        val list = result.getOrNull().orEmpty()
        assertEquals(1, list.size)
        assertEquals("Project Phoenix", list[0].name)
    }
}

