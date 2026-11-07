package ai.nomi.app.screens.projects

import ai.nomi.core.timeAgo
import ai.nomi.designsystem.brand.NomiPageTitle
import ai.nomi.designsystem.ds.ChipKind
import ai.nomi.designsystem.ds.Crew
import ai.nomi.designsystem.ds.Ds
import ai.nomi.designsystem.ds.DsButton
import ai.nomi.designsystem.ds.DsButtonVariant
import ai.nomi.designsystem.ds.DsChip
import ai.nomi.designsystem.ds.DsIcon
import ai.nomi.designsystem.ds.DsIconButton
import ai.nomi.designsystem.ds.DsIconButtonVariant
import ai.nomi.designsystem.ds.DsMenu
import ai.nomi.designsystem.ds.DsMotion
import ai.nomi.designsystem.ds.DsRule
import ai.nomi.designsystem.ds.DsSearch
import ai.nomi.designsystem.ds.DsSegmented
import ai.nomi.designsystem.ds.DsStatus
import ai.nomi.designsystem.ds.EmptyState
import ai.nomi.designsystem.ds.ErrorCallout
import ai.nomi.designsystem.ds.ErrorDetail
import ai.nomi.designsystem.ds.KVRow
import ai.nomi.designsystem.ds.Mascot
import ai.nomi.designsystem.ds.MenuEntry
import ai.nomi.designsystem.ds.SegmentOption
import ai.nomi.designsystem.ds.SkeletonRow
import ai.nomi.designsystem.ds.StatusKind
import ai.nomi.designsystem.icon.Lucide
import ai.nomi.designsystem.layout.LocalNomiPageTopPadding
import ai.nomi.designsystem.theme.NomiTheme
import ai.nomi.network.ProjectDto
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.expandVertically
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.shrinkVertically
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawingPadding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Text
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.compositeOver
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import kotlin.time.Clock
import kotlin.time.Instant

const val PROJECTS_SCREEN_TAG = "projects_screen"

// The web's Projects page (`(app)/projects/+page.svelte`): the title and lede, the "describe an app" card, filter
// tabs with counts, search and sort, the project cards, and a project's details (the web's side panel, a sheet here).

private val PageMaxWidth = 960.dp
private val TwoColumnWidth = 840.dp
private val PagePadding = 16.dp
private val SectionGap = 16.dp
private val CardGap = 14.dp
private val CardPadding = 16.dp
private val HeroPadding = 14.dp
private val HeroGap = 12.dp
private val HeroMascot = 32.dp
private val ChipGap = 6.dp
private val ToneSize = 40.dp
private val ToneSheetSize = 36.dp
private val CardRowGap = 12.dp
private val StatusTop = 14.dp
private val FooterTop = 16.dp
private val FooterPadTop = 12.dp
private val SheetPadding = 20.dp
private val SheetGap = 16.dp
private val PreviewPadding = 16.dp
private val BorderWidth = 1.dp
private const val SKELETON_ROWS = 3
private const val TONE_MIX = 0.18f
private const val HASH_SHIFT = 5

/** `toneFor`: the crew colours a project's letter can take, picked by a hash of its name. */
private val Tones = listOf(Crew.Nomi, Crew.Dana, Crew.Kala, Crew.Koda, Crew.Rena, Crew.Supervisor)

private fun toneFor(name: String): Crew {
    var hash = 0
    for (c in name) hash = (hash shl HASH_SHIFT) - hash + c.code
    return Tones[kotlin.math.abs(hash % Tones.size)]
}

private fun ProjectFilter.label() = when (this) {
    ProjectFilter.All -> "All"
    ProjectFilter.Live -> "Live"
    ProjectFilter.Building -> "Building"
    ProjectFilter.Attention -> "Needs attention"
}

/** "2 chats · deploy #3 · 4h ago" under a card. */
private fun ProjectDto.meta(nowMs: Long): String {
    val chats = chatsCount ?: 1
    val deploys = deploysCount?.takeIf { it > 0 }?.let { "deploy #$it" } ?: "draft"
    val updated = runCatching { timeAgo(Instant.parse(updatedAt).toEpochMilliseconds(), nowMs) }.getOrDefault(updatedAt)
    return "$chats ${if (chats == 1L) "chat" else "chats"} · $deploys · $updated"
}

@Composable
fun ProjectsScreen(
    viewModel: ProjectsViewModel,
    onOpenChat: (String) -> Unit = {},
    modifier: Modifier = Modifier,
) {
    val state by viewModel.uiState.collectAsStateWithLifecycle()
    val newProject by viewModel.newProject.collectAsStateWithLifecycle()
    ProjectsContent(
        state = state,
        newProject = newProject,
        onPrompt = viewModel::setPrompt,
        onStartBuilding = { viewModel.startBuilding(onOpenChat) },
        onQuery = viewModel::setSearchQuery,
        onFilter = viewModel::setFilter,
        onToggleSort = viewModel::toggleSort,
        onOpenChat = onOpenChat,
        modifier = modifier,
    )
}

@Composable
fun ProjectsContent(
    state: ProjectsUiState,
    newProject: NewProjectState,
    onPrompt: (String) -> Unit,
    onStartBuilding: () -> Unit,
    onQuery: (String) -> Unit,
    onFilter: (ProjectFilter) -> Unit,
    onToggleSort: () -> Unit,
    onOpenChat: (String) -> Unit,
    modifier: Modifier = Modifier,
    nowMs: Long = Clock.System.now().toEpochMilliseconds(),
) {
    var selectedId by rememberSaveable { mutableStateOf<String?>(null) }
    BoxWithConstraints(modifier.fillMaxSize().background(NomiTheme.extendedColors.ground).safeDrawingPadding().testTag(PROJECTS_SCREEN_TAG), contentAlignment = Alignment.TopCenter) {
        val columns = if (maxWidth >= TwoColumnWidth) 2 else 1
        LazyColumn(
            Modifier.widthIn(max = PageMaxWidth).fillMaxSize(),
            contentPadding = PaddingValues(start = PagePadding, end = PagePadding, top = LocalNomiPageTopPadding.current, bottom = PagePadding),
            verticalArrangement = Arrangement.spacedBy(SectionGap),
        ) {
            item(key = "title") { Heading() }
            item(key = "hero") { HeroPrompt(newProject, onPrompt, onStartBuilding) }
            when (state) {
                ProjectsUiState.Loading -> items(SKELETON_ROWS) { SkeletonRow(if (it == 0) "loading projects…" else "") }
                is ProjectsUiState.Error -> item(key = "error") {
                    ErrorCallout("Couldn't load projects", details = listOf(ErrorDetail("reason", state.message, strong = true)))
                }
                is ProjectsUiState.Ready -> {
                    item(key = "toolbar") { Toolbar(state, onQuery, onFilter, onToggleSort) }
                    val shown = state.filteredProjects
                    if (shown.isEmpty()) {
                        item(key = "empty") {
                            if (state.query.isBlank()) {
                                EmptyState(Lucide.Folder, "No projects here yet", "Start one and Rena will ask what you need.", count = 0)
                            } else {
                                EmptyState(Lucide.Search, "Nothing matches", "No project matches \"${state.query}\".")
                            }
                        }
                    } else {
                        // `lg:grid-cols-2`: two cards a row once there's room.
                        items(shown.chunked(columns), key = { row -> row.joinToString { it.id } }) { row ->
                            Row(
                                Modifier.height(IntrinsicSize.Max).animateItem(fadeInSpec = DsMotion.effects(), placementSpec = DsMotion.spatial(), fadeOutSpec = DsMotion.fast()),
                                horizontalArrangement = Arrangement.spacedBy(CardGap),
                            ) {
                                row.forEach { project ->
                                    ProjectCard(
                                        project,
                                        nowMs,
                                        selected = project.id == selectedId,
                                        onSelect = { selectedId = project.id },
                                        onOpen = { if (project.sessionId.isNotBlank()) onOpenChat(project.sessionId) },
                                        modifier = Modifier.weight(1f).fillMaxHeight(),
                                    )
                                }
                                repeat(columns - row.size) { Spacer(Modifier.weight(1f)) }
                            }
                        }
                    }
                }
            }
        }
    }
    val selected = (state as? ProjectsUiState.Ready)?.projects?.firstOrNull { it.id == selectedId }
    if (selected != null) {
        ProjectSheet(selected, nowMs, onDismiss = { selectedId = null }, onOpen = {
            selectedId = null
            if (selected.sessionId.isNotBlank()) onOpenChat(selected.sessionId)
        })
    }
}

@Composable
private fun Heading() {
    Column(verticalArrangement = Arrangement.spacedBy(ChipGap)) {
        NomiPageTitle("Projects")
        Text(
            "Apps Koda builds and ships for you. Each one has its own code, live URL and data, and you can export it any time. " +
                "For questions and planning, use Chats.",
            style = Ds.caption,
            color = Ds.text2,
        )
    }
}

/** `.hero-prompt`: Koda beside "Describe an app to build…", what Koda sets up, and Start building. */
@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun HeroPrompt(state: NewProjectState, onPrompt: (String) -> Unit, onStart: () -> Unit) {
    Column(
        Modifier.fillMaxWidth().clip(MaterialTheme.shapes.large).background(Ds.surface).border(BorderWidth, Ds.border, MaterialTheme.shapes.large).padding(HeroPadding),
        verticalArrangement = Arrangement.spacedBy(HeroGap),
    ) {
        Row(horizontalArrangement = Arrangement.spacedBy(HeroGap), verticalAlignment = Alignment.CenterVertically) {
            Mascot(Crew.Koda, HeroMascot)
            Box(Modifier.weight(1f)) {
                if (state.prompt.isEmpty()) {
                    Text("Describe an app to build, e.g. a booking page for my barbershop…", style = Ds.body, color = Ds.text3)
                }
                BasicTextField(
                    state.prompt,
                    onPrompt,
                    Modifier.fillMaxWidth(),
                    enabled = !state.starting,
                    textStyle = Ds.body.copy(color = Ds.text),
                    cursorBrush = SolidColor(Ds.accent),
                    keyboardOptions = KeyboardOptions(imeAction = ImeAction.Go),
                    keyboardActions = KeyboardActions(onGo = { onStart() }),
                )
            }
        }
        DsRule()
        FlowRow(horizontalArrangement = Arrangement.spacedBy(ChipGap), verticalArrangement = Arrangement.spacedBy(ChipGap), itemVerticalAlignment = Alignment.CenterVertically) {
            Text("Koda sets up", style = Ds.ui, color = Ds.text3)
            DsChip(ChipKind.Provision, "git repo", icon = Lucide.GitBranch)
            DsChip(ChipKind.Provision, "database · optional", icon = Lucide.Database)
        }
        AnimatedVisibility(state.error != null, enter = expandVertically() + fadeIn(), exit = shrinkVertically() + fadeOut()) {
            Text(state.error.orEmpty(), style = Ds.caption, color = Ds.danger)
        }
        DsButton("Start building", onStart, Modifier.fillMaxWidth(), iconEnd = Lucide.ArrowRight, loading = state.starting)
    }
}

/** The filter tabs (with counts, scrolling sideways on a phone), then search and the sort button. */
@Composable
private fun Toolbar(state: ProjectsUiState.Ready, onQuery: (String) -> Unit, onFilter: (ProjectFilter) -> Unit, onToggleSort: () -> Unit) {
    Column(verticalArrangement = Arrangement.spacedBy(CardRowGap)) {
        Box(Modifier.fillMaxWidth().horizontalScroll(rememberScrollState())) {
            DsSegmented(
                "Filter projects",
                ProjectFilter.entries.map { SegmentOption(it.name, it.label(), state.count(it).toString()) },
                state.filter.name,
                { onFilter(ProjectFilter.valueOf(it)) },
            )
        }
        Row(horizontalArrangement = Arrangement.spacedBy(CardRowGap), verticalAlignment = Alignment.CenterVertically) {
            DsSearch(state.query, onQuery, "Search projects", Modifier.weight(1f), plain = true)
            DsButton(
                if (state.sort == ProjectSort.Updated) "Updated" else "Name",
                onToggleSort,
                variant = DsButtonVariant.Secondary,
                iconEnd = Lucide.ChevronDown,
            )
        }
    }
}

/** The project's letter on its crew tone (`toneFor`). */
@Composable
private fun ToneMark(name: String, size: androidx.compose.ui.unit.Dp) {
    val tone = toneFor(name)
    Box(
        Modifier.size(size).clip(MaterialTheme.shapes.small)
            .background(tone.tone.copy(alpha = TONE_MIX).compositeOver(MaterialTheme.colorScheme.surfaceContainerHigh)),
        contentAlignment = Alignment.Center,
    ) {
        Text(name.take(1).uppercase(), style = MaterialTheme.typography.titleSmall, color = tone.tone)
    }
}

/** The status row: Koda building, live with its address, failed with its issues, or a draft. */
@Composable
private fun StatusLine(project: ProjectDto) {
    Row(horizontalArrangement = Arrangement.spacedBy(ChipGap), verticalAlignment = Alignment.CenterVertically) {
        val state = project.state()
        when (state) {
            ProjectState.Working -> DsStatus(StatusKind.Working, "Koda is building", agent = Crew.Koda, small = true)
            ProjectState.Live -> DsStatus(StatusKind.Live, "Live", small = true)
            ProjectState.Failed -> DsStatus(StatusKind.Failed, "Deploy failed", small = true)
            ProjectState.Idle -> DsStatus(StatusKind.Idle, "Draft", small = true)
        }
        val detail = when (state) {
            ProjectState.Working -> project.stack?.let { "$it app" } ?: "writing code"
            ProjectState.Live -> project.host() ?: "not published"
            ProjectState.Failed -> "${project.openIssues} issue(s)"
            ProjectState.Idle -> "not published"
        }
        Text(detail, style = Ds.mono, color = Ds.text3, maxLines = 1, overflow = TextOverflow.Ellipsis)
    }
}

@Composable
private fun ProjectCard(project: ProjectDto, nowMs: Long, selected: Boolean, onSelect: () -> Unit, onOpen: () -> Unit, modifier: Modifier = Modifier) {
    val uri = LocalUriHandler.current
    val edge by animateColorAsState(if (selected) Ds.text else Ds.border, DsMotion.fast(), label = "card-edge")
    Column(
        modifier.fillMaxWidth().clip(MaterialTheme.shapes.large).background(Ds.surface).border(BorderWidth, edge, MaterialTheme.shapes.large)
            .clickable(role = Role.Button, onClickLabel = "Show details", onClick = onSelect).padding(CardPadding),
    ) {
        Row(horizontalArrangement = Arrangement.spacedBy(CardRowGap), verticalAlignment = Alignment.Top) {
            ToneMark(project.name, ToneSize)
            Column(Modifier.weight(1f)) {
                Text(project.name, style = Ds.body, color = Ds.text, maxLines = 1, overflow = TextOverflow.Ellipsis)
                Text(project.description?.ifBlank { null } ?: "No description yet.", style = Ds.caption, color = Ds.text2, maxLines = 1, overflow = TextOverflow.Ellipsis)
            }
            DsMenu(
                listOfNotNull(
                    MenuEntry.Item("Open", Lucide.ArrowRight, onSelect = onOpen),
                    project.deployUrl?.let { url -> MenuEntry.Item("Visit", Lucide.ExternalLink, onSelect = { runCatching { uri.openUri(url) } }) },
                ),
            ) { open, _ -> DsIconButton(Lucide.Ellipsis, "Project options", open, variant = DsIconButtonVariant.Ghost) }
        }
        Box(Modifier.padding(top = StatusTop)) { StatusLine(project) }
        Column(Modifier.padding(top = FooterTop)) {
            DsRule()
            Row(Modifier.padding(top = FooterPadTop), horizontalArrangement = Arrangement.spacedBy(CardRowGap), verticalAlignment = Alignment.CenterVertically) {
                Text(project.meta(nowMs), style = Ds.mono, color = Ds.text3, maxLines = 1, overflow = TextOverflow.Ellipsis, modifier = Modifier.weight(1f))
                if (project.state() == ProjectState.Failed) {
                    DsButton("Ask Koda to fix", onOpen, variant = DsButtonVariant.Secondary, iconEnd = Lucide.Sparkles)
                } else {
                    DsButton("Open", onOpen, variant = DsButtonVariant.Secondary, iconEnd = Lucide.ArrowRight)
                }
            }
        }
    }
}

/** A project's details (the web's side panel): its preview card, what's in it, and the way into its workspace. */
@OptIn(ExperimentalMaterial3Api::class, ExperimentalLayoutApi::class)
@Composable
private fun ProjectSheet(project: ProjectDto, nowMs: Long, onDismiss: () -> Unit, onOpen: () -> Unit) {
    val uri = LocalUriHandler.current
    val live = project.state() == ProjectState.Live
    ModalBottomSheet(onDismissRequest = onDismiss, sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true), containerColor = Ds.surface) {
        Column(Modifier.fillMaxWidth().padding(horizontal = SheetPadding).padding(bottom = SheetPadding).navigationBarsPadding(), verticalArrangement = Arrangement.spacedBy(SheetGap)) {
            Row(horizontalArrangement = Arrangement.spacedBy(CardRowGap), verticalAlignment = Alignment.CenterVertically) {
                ToneMark(project.name, ToneSheetSize)
                Column(Modifier.weight(1f)) {
                    Text(project.name, style = Ds.body, color = Ds.text, maxLines = 1, overflow = TextOverflow.Ellipsis)
                    Text(
                        "${project.state().name.lowercase()} · ${project.meta(nowMs).substringAfter(" · ")}",
                        style = Ds.caption,
                        color = Ds.text3,
                    )
                }
                DsIconButton(Lucide.X, "Close", onDismiss, variant = DsIconButtonVariant.Ghost)
            }
            Column(
                Modifier.fillMaxWidth().clip(MaterialTheme.shapes.large).background(Ds.surface2).border(BorderWidth, Ds.border, MaterialTheme.shapes.large).padding(PreviewPadding),
                verticalArrangement = Arrangement.spacedBy(ChipGap),
            ) {
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Text(project.stack?.let { "${it.uppercase()} APP" } ?: "APPLICATION PREVIEW", style = Ds.labelCaps, color = Ds.text3, modifier = Modifier.weight(1f))
                    if (project.deployUrl != null) Text("LIVE", style = Ds.labelCaps, color = Ds.accent)
                }
                DsRule()
                Text(project.name, style = MaterialTheme.typography.titleMedium, color = Ds.text)
                Text(project.description?.ifBlank { null } ?: "No description yet.", style = Ds.caption, color = Ds.text2, maxLines = 2, overflow = TextOverflow.Ellipsis)
                FlowRow(horizontalArrangement = Arrangement.spacedBy(ChipGap), verticalArrangement = Arrangement.spacedBy(ChipGap)) {
                    DsChip(ChipKind.Provision, "${project.filesCount ?: 0} files", icon = Lucide.FileCode)
                    DsChip(ChipKind.Provision, backendLabel(project.backend), icon = Lucide.Database)
                    project.deploysCount?.takeIf { it > 0 }?.let { DsChip(ChipKind.Provision, "$it deploys", icon = Lucide.Rocket) }
                }
            }
            Column {
                Text("WHAT'S IN THIS PROJECT", style = Ds.labelCaps, color = Ds.text2)
                KVRow("Code", "${project.filesCount ?: 0} files", icon = Lucide.FileCode)
                KVRow("URL", project.host() ?: "not published", icon = Lucide.Globe)
                KVRow("Data", backendLabel(project.backend), icon = Lucide.Database)
                KVRow("Deploys", project.deploysCount?.toString() ?: "0", icon = Lucide.Rocket)
            }
            DsButton("Open workspace", onOpen, Modifier.fillMaxWidth(), iconEnd = Lucide.ArrowRight)
            if (live && project.deployUrl != null) {
                DsButton("Visit", { runCatching { uri.openUri(project.deployUrl!!) } }, Modifier.fillMaxWidth(), variant = DsButtonVariant.Secondary, iconEnd = Lucide.ExternalLink)
            }
        }
    }
}

private fun backendLabel(backend: String?) = when (backend) {
    "relay" -> "Relay"
    "drizzle" -> "PostgreSQL"
    else -> "None"
}
