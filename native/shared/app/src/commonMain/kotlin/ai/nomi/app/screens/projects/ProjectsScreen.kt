package ai.nomi.app.screens.projects

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
            is ProjectsUiState.Loading -> {
                Box(Modifier.fillMaxSize().padding(32.dp), contentAlignment = Alignment.Center) {
                    NomiLoadingIndicator(contentDescription = "Loading projects")
                }
            }
            is ProjectsUiState.Error -> {
                Box(Modifier.fillMaxSize().padding(24.dp), contentAlignment = Alignment.Center) {
                    Text(
                        text = s.message,
                        style = MaterialTheme.typography.bodyLarge,
                        color = MaterialTheme.colorScheme.error,
                    )
                }
            }
            is ProjectsUiState.Ready -> {
                Column(Modifier.fillMaxSize(), verticalArrangement = Arrangement.spacedBy(16.dp)) {
                    NomiTextField(
                        value = s.query,
                        onValueChange = viewModel::setSearchQuery,
                        label = "Search projects",
                        modifier = Modifier.fillMaxWidth(),
                    )

                    if (s.filteredProjects.isEmpty()) {
                        Box(Modifier.fillMaxWidth().padding(32.dp), contentAlignment = Alignment.Center) {
                            Text(
                                text = if (s.query.isBlank()) "No projects yet." else "No projects match '${s.query}'.",
                                style = MaterialTheme.typography.bodyMedium,
                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                            )
                        }
                    } else {
                        LazyColumn(
                            modifier = Modifier.fillMaxSize(),
                            verticalArrangement = Arrangement.spacedBy(12.dp),
                        ) {
                            items(s.filteredProjects, key = { it.id }) { project ->
                                ProjectCard(
                                    project = project,
                                    onClick = {
                                        if (project.sessionId.isNotBlank()) {
                                            onOpenChat(project.sessionId)
                                        }
                                    },
                                )
                            }
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun ProjectCard(
    project: ProjectDto,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    NomiCard(
        modifier = modifier
            .fillMaxWidth()
            .clickable(onClick = onClick),
    ) {
        Column(
            modifier = Modifier.padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(
                    text = project.name,
                    style = MaterialTheme.typography.titleMedium.copy(fontWeight = FontWeight.Bold),
                    color = MaterialTheme.colorScheme.onSurface,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier.weight(1f),
                )
                ai.nomi.designsystem.component.NomiFilterChip(
                    label = project.status.replaceFirstChar { it.uppercase() },
                    selected = project.status == "ready",
                    onClick = onClick,
                )
            }
            val desc = project.description
            if (!desc.isNullOrBlank()) {
                Text(
                    text = desc,
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis,
                )
            }

            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
            ) {
                Text(
                    text = "By ${project.createdBy}",
                    style = MaterialTheme.typography.labelSmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                if (project.openIssues > 0) {
                    Text(
                        text = "${project.openIssues} open issues",
                        style = MaterialTheme.typography.labelSmall.copy(fontWeight = FontWeight.SemiBold),
                        color = MaterialTheme.colorScheme.primary,
                    )
                }
            }
        }
    }
}
