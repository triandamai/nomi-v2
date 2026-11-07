package ai.nomi.app.screens

import ai.nomi.app.screens.projects.NewProjectState
import ai.nomi.app.screens.projects.ProjectsContent
import ai.nomi.app.screens.projects.ProjectsUiState
import ai.nomi.network.ProjectDto
import androidx.compose.ui.test.junit4.createComposeRule
import org.junit.Rule
import kotlin.test.Test

private const val MINUTE_MS = 60_000L
private const val HOUR = 60 * MINUTE_MS
private fun ago(ms: Long) = kotlin.time.Instant.fromEpochMilliseconds(SNAPSHOT_NOW - ms).toString()

private val Projects = listOf(
    ProjectDto(
        id = "p1", sessionId = "s1", name = "Bakery orders", description = "Pre-orders for the weekend bake",
        status = "ready", updatedAt = ago(HOUR * 3), createdBy = "Ana", stack = "sveltekit", backend = "drizzle",
        chatsCount = 3, deploysCount = 4, filesCount = 42, deployUrl = "https://bakery.nomi.app",
    ),
    ProjectDto(
        id = "p2", sessionId = "s2", name = "Barbershop booking", description = "Pick a slot, get a reminder",
        status = "building", updatedAt = ago(MINUTE_MS * 20), createdBy = "Ana", stack = "sveltekit", chatsCount = 1,
    ),
    ProjectDto(
        id = "p3", sessionId = "s3", name = "Quiz night", description = null, status = "ready", openIssues = 2,
        updatedAt = ago(HOUR * 30), createdBy = "Bob", deploysCount = 1,
    ),
)

class ProjectsScreenshotTest {
    @get:Rule val rule = createComposeRule()

    private fun capture(name: String, device: Device, dark: Boolean, state: ProjectsUiState, prompt: NewProjectState = NewProjectState()) =
        rule.captureScreen(name, device, dark) {
            ProjectsContent(state, prompt, {}, {}, {}, {}, {}, {}, nowMs = SNAPSHOT_NOW)
        }

    @Test fun listPhoneLight() = capture("projects", Device.Phone, false, ProjectsUiState.Ready(Projects))
    @Test fun listPhoneDark() = capture("projects", Device.Phone, true, ProjectsUiState.Ready(Projects))
    @Test fun listWideLight() = capture("projects", Device.Wide, false, ProjectsUiState.Ready(Projects))
    @Test fun startingPhoneLight() =
        capture("projects_starting", Device.Phone, false, ProjectsUiState.Ready(Projects), NewProjectState("A booking page for my barbershop", starting = true))
    @Test fun emptyPhoneLight() = capture("projects_empty", Device.Phone, false, ProjectsUiState.Ready(emptyList()))
    @Test fun loadingPhoneLight() = capture("projects_loading", Device.Phone, false, ProjectsUiState.Loading)
}
