package ai.nomi.app.screens.projects

import ai.nomi.designsystem.ds.Crew
import ai.nomi.designsystem.ds.DsMotion
import ai.nomi.designsystem.ds.DsSearch
import ai.nomi.designsystem.ds.EmptyState
import ai.nomi.designsystem.ds.ErrorCallout
import ai.nomi.designsystem.ds.ErrorDetail
import ai.nomi.designsystem.ds.ProjectRow
import ai.nomi.designsystem.ds.ProjectStatus
import ai.nomi.designsystem.ds.SkeletonRow
import ai.nomi.designsystem.ds.StatusKind
import ai.nomi.designsystem.icon.Lucide
import ai.nomi.designsystem.component.NomiCard
import ai.nomi.designsystem.component.NomiLoadingIndicator
import ai.nomi.designsystem.component.NomiTextField
import ai.nomi.designsystem.layout.NomiPage
import ai.nomi.designsystem.theme.NomiTheme
import ai.nomi.network.ProjectDto
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle

const val PROJECTS_SCREEN_TAG = "projects_screen"

@Composable
fun ProjectsScreen(
    viewModel: ProjectsViewModel,
    onOpenChat: (String) -> Unit = {},
    modifier: Modifier = Modifier,
) {
    val state by viewModel.uiState.collectAsStateWithLifecycle()

    NomiPage(
        title = "Projects",
        modifier = modifier.testTag(PROJECTS_SCREEN_TAG),
    ) {
        when (val s = state) {
            is ProjectsUiState.Loading -> repeat(SKELETON_ROWS) { SkeletonRow(if (it == 0) "loading projects…" else "") }
            is ProjectsUiState.Error -> ErrorCallout("Couldn't load projects", details = listOf(ErrorDetail("reason", s.message, strong = true)))
            is ProjectsUiState.Ready -> {
                Column(Modifier.fillMaxSize(), verticalArrangement = Arrangement.spacedBy(16.dp)) {
                    DsSearch(s.query, viewModel::setSearchQuery, "Search projects")
                    if (s.filteredProjects.isEmpty()) {
                        if (s.query.isBlank()) {
                            EmptyState(Lucide.Folder, "No projects yet", "Describe an app in a chat and Koda builds it here.", count = 0)
                        } else {
                            EmptyState(Lucide.Search, "Nothing matches", "No project's name has \"${s.query}\".")
                        }
                    } else {
                        LazyColumn(Modifier.fillMaxSize(), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                            items(s.filteredProjects, key = { it.id }) { project ->
                                ProjectRow(
                                    name = project.name,
                                    tone = Crew.Koda,
                                    status = projectStatus(project.status),
                                    updated = project.updatedAt.take(DATE_CHARS),
                                    onClick = { if (project.sessionId.isNotBlank()) onOpenChat(project.sessionId) },
                                    modifier = Modifier.animateItem(fadeInSpec = DsMotion.effects(), placementSpec = DsMotion.spatial(), fadeOutSpec = DsMotion.fast()),
                                    description = listOfNotNull(
                                        project.description?.takeIf { it.isNotBlank() },
                                        if (project.openIssues > 0) "${project.openIssues} open issues" else null,
                                    ).joinToString(" · ").ifEmpty { "By ${project.createdBy}" },
                                )
                            }
                        }
                    }
                }
            }
        }
    }
}

private const val SKELETON_ROWS = 3
/** `2026-11-07T…` → `2026-11-07`. */
private const val DATE_CHARS = 10

/** A project's state as the design system's status: live, ready, building (Koda's shape moving), failed, idle. */
private fun projectStatus(status: String): ProjectStatus = when (status.lowercase()) {
    "live", "deployed", "published" -> ProjectStatus(StatusKind.Live, "Live")
    "ready" -> ProjectStatus(StatusKind.Ready, "Ready")
    "building", "running", "deploying" -> ProjectStatus(StatusKind.Working, "Building")
    "failed", "error" -> ProjectStatus(StatusKind.Failed, "Failed")
    else -> ProjectStatus(StatusKind.Idle, status.replaceFirstChar { it.uppercase() })
}
