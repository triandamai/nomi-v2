package ai.nomi.app.screens.projects

import ai.nomi.data.ProjectDeployInfo
import ai.nomi.data.ProjectFileEntry
import ai.nomi.data.ProjectInfo
import ai.nomi.data.ProjectIssueInfo
import ai.nomi.data.ProjectIssueList
import ai.nomi.data.ProjectPublishInfo
import ai.nomi.data.ProjectTeam
import ai.nomi.designsystem.component.ButtonGroupOption
import ai.nomi.designsystem.component.NomiAvatar
import ai.nomi.designsystem.component.NomiButton
import ai.nomi.designsystem.component.NomiButtonGroup
import ai.nomi.designsystem.component.NomiButtonSize
import ai.nomi.designsystem.component.NomiButtonVariant
import ai.nomi.designsystem.component.NomiFilterChip
import ai.nomi.designsystem.component.NomiLoadingIndicator
import ai.nomi.designsystem.component.NomiTextField
import ai.nomi.designsystem.icon.NomiGlyph
import ai.nomi.designsystem.icon.NomiIcon
import ai.nomi.designsystem.icon.NomiIcons
import ai.nomi.designsystem.shape.AgentShape
import ai.nomi.designsystem.shape.NomiIdentity
import ai.nomi.designsystem.theme.NomiTheme
import ai.nomi.feature.chat.ChatScreen
import ai.nomi.feature.chat.ChatViewModel
import ai.nomi.feature.chat.formatBytes
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.consumeWindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBars
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.IconButton
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
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.compose.runtime.LaunchedEffect
import org.koin.core.parameter.parametersOf
import org.koin.compose.viewmodel.koinViewModel

const val PROJECT_WORKSPACE_TAG = "project_workspace"

/** Keeps a project's tools view model apart from the workspace's on the same entry. */
private const val TOOLS_KEY_PREFIX = "tools:"

private const val TAB_CHAT = "chat"
private const val TAB_PREVIEW = "preview"
private const val TAB_CODE = "code"
private const val PROJECT_PLACEHOLDER = "Ask Koda to build or change something"
private const val URL_SCHEME = "://"
private const val FILL = 1f
private const val TITLE_WEIGHT = 700

private val BarPaddingH = 4.dp
private val BarPaddingV = 4.dp
private val BrandSize = 28.dp
private val StatusPaddingH = 7.dp
private val StatusPaddingV = 1.dp
private val StatusCorner = 4.dp
private val ToolInnerGap = 6.dp
private val NoticePaddingV = 8.dp
private const val ISSUE_BODY_LINES = 2
private val ToolsPaddingH = 12.dp
private val ToolsGap = 8.dp
private val ToolPaddingH = 12.dp
private val ToolPaddingV = 8.dp
private val ToolGlyph = 18.dp
private val CountPaddingH = 6.dp
private val TabsPadding = 12.dp
private val PanePadding = 16.dp
private val SectionGap = 12.dp
private val RowGap = 10.dp
private val RowPaddingV = 10.dp
private val FileGlyph = 18.dp
private val CodePadding = 12.dp
private val CardPadding = 16.dp
private val SheetPaddingBottom = 24.dp
private val MemberAvatar = 36.dp
private val EmptyMaxWidth = 320.dp

/** Which tool sheet is open over the workspace. */
enum class ProjectSheet { Issues, Deploy, Domain, Monitor, Database, Members, Settings }

const val PROJECT_TAB_CHAT = TAB_CHAT
const val PROJECT_TAB_PREVIEW = TAB_PREVIEW
const val PROJECT_TAB_CODE = TAB_CODE

/**
 * A chat as it opens: a project's chat in its workspace (like the web's project page, on a phone), anything else as
 * the chat itself. Projects already seen open straight into the workspace.
 */
@Composable
fun ChatDestination(
    workspace: ProjectWorkspaceViewModel,
    chat: ChatViewModel,
    knownProject: Boolean,
    onBack: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val lookup by workspace.lookup.collectAsStateWithLifecycle()
    when (val found = lookup) {
        is ProjectLookup.Project -> ProjectWorkspace(found.project, workspace, chat, onBack, modifier)
        ProjectLookup.Unknown -> if (knownProject) {
            Box(modifier.fillMaxSize().background(NomiTheme.extendedColors.ground), contentAlignment = Alignment.Center) {
                NomiLoadingIndicator(contentDescription = "Opening project")
            }
        } else {
            ChatScreen(chat, modifier, onBack = onBack)
        }
        ProjectLookup.NotAProject -> ChatScreen(chat, modifier, onBack = onBack)
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun ProjectWorkspace(
    project: ProjectInfo,
    viewModel: ProjectWorkspaceViewModel,
    chat: ChatViewModel,
    onBack: () -> Unit,
    modifier: Modifier,
) {
    val deploy by viewModel.deploy.collectAsStateWithLifecycle()
    val notice by viewModel.notice.collectAsStateWithLifecycle()
    val file by viewModel.file.collectAsStateWithLifecycle()
    var tab by rememberSaveable { mutableStateOf(TAB_CHAT) }
    var sheet by rememberSaveable { mutableStateOf<ProjectSheet?>(null) }
    val tools: ProjectToolsViewModel = koinViewModel(key = "$TOOLS_KEY_PREFIX${project.id}") {
        parametersOf(project.id, project.name, project.relay)
    }
    val left by tools.left.collectAsStateWithLifecycle()
    LaunchedEffect(left) {
        // Moved to another organization or deleted: nothing left to show here.
        if (left) onBack()
    }

    ProjectWorkspaceContent(
        project = project,
        deploy = deploy,
        notice = notice,
        openFile = file,
        tab = tab,
        onTabChange = { tab = it },
        onBack = onBack,
        onPublish = viewModel::publish,
        onRetryDeploy = viewModel::refreshDeploy,
        onOpenTool = { which ->
            sheet = which
            when (which) {
                ProjectSheet.Issues -> viewModel.refreshIssues()
                ProjectSheet.Deploy -> viewModel.refreshDeploy()
                ProjectSheet.Members -> viewModel.refreshTeam()
                ProjectSheet.Domain -> tools.loadAddress()
                ProjectSheet.Monitor -> tools.loadHealth()
                ProjectSheet.Database -> tools.loadData()
                ProjectSheet.Settings -> tools.loadSettings()
            }
        },
        onDismissNotice = viewModel::noticeShown,
        onOpenFile = viewModel::openFile,
        onCloseFile = viewModel::closeFile,
        modifier = modifier,
    ) {
        ChatScreen(chat, embedded = true, placeholder = PROJECT_PLACEHOLDER)
    }

    sheet?.let { which ->
        ModalBottomSheet(
            onDismissRequest = { sheet = null },
            sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true),
            containerColor = MaterialTheme.colorScheme.surfaceContainerLow,
        ) {
            Column(Modifier.fillMaxWidth().padding(horizontal = PanePadding).padding(bottom = SheetPaddingBottom).navigationBarsPadding()) {
                when (which) {
                    ProjectSheet.Issues -> IssuesSheet(project, viewModel)
                    ProjectSheet.Deploy -> DeploySheet(deploy, onPublish = viewModel::publish, onRetry = viewModel::refreshDeploy)
                    ProjectSheet.Members -> MembersSheet(viewModel)
                    ProjectSheet.Domain -> {
                        val state by tools.address.collectAsStateWithLifecycle()
                        DomainSheet(state, onEdit = tools::editAddress, onSave = tools::saveAddress, onRetry = tools::loadAddress)
                    }
                    ProjectSheet.Monitor -> {
                        val state by tools.health.collectAsStateWithLifecycle()
                        MonitorSheet(state, onRange = { tools.loadHealth(it) }, onRetry = { tools.loadHealth() })
                    }
                    ProjectSheet.Database -> {
                        val state by tools.data.collectAsStateWithLifecycle()
                        DatabaseSheet(
                            state,
                            relay = project.relay,
                            onSource = { tools.loadData(it) },
                            onSql = tools::editQuery,
                            onRun = tools::runQuery,
                            onRetry = { tools.loadData() },
                            onRetrySetup = tools::retryRelay,
                        )
                    }
                    ProjectSheet.Settings -> {
                        val state by tools.settings.collectAsStateWithLifecycle()
                        SettingsSheet(
                            state,
                            projectName = project.name,
                            canPublish = (deploy as? Loadable.Ready)?.value?.canPublish == true,
                            onBlurb = tools::editBlurb,
                            onShowcase = tools::setShowcase,
                            onTakeOffline = tools::takeOffline,
                            onSearch = tools::searchTargets,
                            onTransfer = tools::transfer,
                            onDelete = tools::delete,
                        )
                    }
                }
            }
        }
    }
}

/**
 * The project's page on a phone (`projects/session/[id]`): its bar (name, status, organization, live address,
 * Publish), its tools, and Chat / Preview / Code tabs. [chat] is the project's chat, shown in the Chat tab.
 */
@Composable
fun ProjectWorkspaceContent(
    project: ProjectInfo,
    deploy: Loadable<ProjectDeployInfo>,
    notice: String?,
    openFile: OpenFile?,
    tab: String,
    onTabChange: (String) -> Unit,
    onBack: () -> Unit,
    onPublish: () -> Unit,
    onRetryDeploy: () -> Unit,
    onOpenTool: (ProjectSheet) -> Unit,
    onDismissNotice: () -> Unit,
    onOpenFile: (String) -> Unit,
    onCloseFile: () -> Unit,
    modifier: Modifier = Modifier,
    chat: @Composable () -> Unit,
) {
    val ready = (deploy as? Loadable.Ready)?.value
    Column(modifier.fillMaxSize().background(NomiTheme.extendedColors.ground).testTag(PROJECT_WORKSPACE_TAG)) {
        Column(Modifier.statusBarsPadding()) {
            ProjectTopBar(project, ready, onBack, onPublish = onPublish)
            ToolsRow(openIssues = project.openIssues, onOpen = onOpenTool)
            NomiButtonGroup(
                options = listOf(
                    ButtonGroupOption(TAB_CHAT, "Chat"),
                    ButtonGroupOption(TAB_PREVIEW, "Preview"),
                    ButtonGroupOption(TAB_CODE, "Code"),
                ),
                selected = tab,
                onSelect = onTabChange,
                modifier = Modifier.fillMaxWidth().padding(horizontal = TabsPadding, vertical = BarPaddingV),
            )
            notice?.let { message ->
                Text(
                    message,
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onErrorContainer,
                    modifier = Modifier
                        .fillMaxWidth()
                        .background(MaterialTheme.colorScheme.errorContainer)
                        .clickable(onClickLabel = "Dismiss", onClick = onDismissNotice)
                        .padding(horizontal = PanePadding, vertical = NoticePaddingV),
                )
            }
        }
        // The bar above already sits below the status bar.
        Box(Modifier.weight(FILL).consumeWindowInsets(WindowInsets.statusBars)) {
            when (tab) {
                TAB_CHAT -> chat()
                TAB_PREVIEW -> PreviewPane(deploy, onPublish = onPublish, onRetry = onRetryDeploy)
                else -> CodePane(project.files, openFile, onOpenFile, onCloseFile)
            }
        }
    }
}

/** `.topbar`: back to the projects, the project's name and status, its organization and live address, Publish. */
@Composable
private fun ProjectTopBar(project: ProjectInfo, deploy: ProjectDeployInfo?, onBack: () -> Unit, onPublish: () -> Unit) {
    val colors = MaterialTheme.colorScheme
    Row(
        Modifier.fillMaxWidth().padding(horizontal = BarPaddingH, vertical = BarPaddingV),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(ToolsGap),
    ) {
        IconButton(onClick = onBack) { NomiIcon(NomiIcons.ArrowBack, contentDescription = "Back") }
        AgentShape(NomiIdentity, BrandSize, face = true)
        Column(Modifier.weight(FILL)) {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(ToolsGap)) {
                Text(
                    project.name,
                    style = MaterialTheme.typography.titleMedium.copy(fontWeight = FontWeight(TITLE_WEIGHT)),
                    color = colors.onSurface,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier.weight(FILL, fill = false).semantics { heading() },
                )
                StatusPill(project.status)
            }
            val liveUrl = deploy?.url
            val meta = buildString {
                append(project.orgName)
                when {
                    liveUrl != null -> append(" · ").append(liveUrl.substringAfter(URL_SCHEME))
                    deploy?.publishing == true -> append(" · Publishing…")
                }
            }
            Text(meta, style = NomiTheme.metaStyle, color = colors.onSurfaceVariant, maxLines = 1, overflow = TextOverflow.Ellipsis)
        }
        if (deploy?.canPublish == true) {
            NomiButton(onClick = onPublish, size = NomiButtonSize.Small, enabled = !deploy.publishing) {
                Text(if (deploy.publishing) "Publishing…" else "Publish")
            }
        }
    }
}

@Composable
private fun StatusPill(status: String) {
    val colors = MaterialTheme.colorScheme
    val ready = status == STATUS_READY
    Text(
        statusLabel(status),
        style = MaterialTheme.typography.labelSmall.copy(fontWeight = FontWeight.SemiBold),
        color = if (ready) colors.onSecondaryContainer else colors.onSurfaceVariant,
        modifier = Modifier
            .background(if (ready) colors.secondaryContainer else colors.surfaceContainerHighest, RoundedCornerShape(StatusCorner))
            .padding(horizontal = StatusPaddingH, vertical = StatusPaddingV),
    )
}

private const val STATUS_READY = "ready"

private fun statusLabel(status: String) = when (status) {
    "planning" -> "Planning"
    "building" -> "Building"
    STATUS_READY -> "Ready"
    "failed" -> "Failed"
    else -> status.replaceFirstChar { it.uppercase() }
}

/** `.topbar__tools`: the project's tools, each opening its sheet. */
@Composable
private fun ToolsRow(openIssues: Long, onOpen: (ProjectSheet) -> Unit) {
    Row(
        Modifier.fillMaxWidth().horizontalScroll(rememberScrollState()).padding(horizontal = ToolsPaddingH),
        horizontalArrangement = Arrangement.spacedBy(ToolsGap),
    ) {
        // In the web toolbar's order.
        Tool("Issues", NomiIcons.Bug, count = openIssues) { onOpen(ProjectSheet.Issues) }
        Tool("Deploy", NomiIcons.Rocket) { onOpen(ProjectSheet.Deploy) }
        Tool("Domain", NomiIcons.Globe) { onOpen(ProjectSheet.Domain) }
        Tool("Monitor", NomiIcons.Pulse) { onOpen(ProjectSheet.Monitor) }
        Tool("Database", NomiIcons.Database) { onOpen(ProjectSheet.Database) }
        Tool("Members", NomiIcons.Person) { onOpen(ProjectSheet.Members) }
        Tool("Settings", NomiIcons.Settings) { onOpen(ProjectSheet.Settings) }
    }
}

@Composable
private fun Tool(label: String, glyph: NomiGlyph, count: Long = 0, onClick: () -> Unit) {
    val colors = MaterialTheme.colorScheme
    Row(
        Modifier
            .clip(CircleShape)
            .background(colors.surfaceContainerLow)
            .clickable(role = Role.Button, onClickLabel = label, onClick = onClick)
            .heightIn(min = TOUCH_TARGET)
            .padding(horizontal = ToolPaddingH, vertical = ToolPaddingV),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(ToolInnerGap),
    ) {
        NomiIcon(glyph, contentDescription = null, size = ToolGlyph, tint = colors.onSurfaceVariant)
        Text(label, style = MaterialTheme.typography.labelLarge, color = colors.onSurface)
        if (count > 0) {
            Text(
                count.toString(),
                style = MaterialTheme.typography.labelSmall.copy(fontWeight = FontWeight.Bold),
                color = colors.onPrimary,
                modifier = Modifier.background(colors.primary, CircleShape).padding(horizontal = CountPaddingH),
            )
        }
    }
}

private val TOUCH_TARGET = 40.dp

/** The Preview tab: where the project runs for everyone, or how to get it there. */
@Composable
private fun PreviewPane(deploy: Loadable<ProjectDeployInfo>, onPublish: () -> Unit, onRetry: () -> Unit) {
    Column(Modifier.fillMaxSize().verticalScroll(rememberScrollState()).padding(PanePadding), verticalArrangement = Arrangement.spacedBy(SectionGap)) {
        when (deploy) {
            Loadable.Loading -> CenteredLoading("Loading preview")
            is Loadable.Failed -> RetryNote(deploy.message, onRetry)
            is Loadable.Ready -> {
                val info = deploy.value
                val uri = LocalUriHandler.current
                val url = info.url
                if (url != null) {
                    Card {
                        Text("Live", style = MaterialTheme.typography.labelLarge, color = MaterialTheme.colorScheme.primary)
                        Text(
                            url.substringAfter(URL_SCHEME),
                            style = MaterialTheme.typography.titleMedium.copy(fontWeight = FontWeight(TITLE_WEIGHT)),
                            color = MaterialTheme.colorScheme.onSurface,
                        )
                        info.publishes.firstOrNull()?.let { PublishLine(it) }
                        NomiButton(onClick = { uri.openUri(url) }, variant = NomiButtonVariant.Tonal) { Text("Open site") }
                    }
                } else {
                    EmptyNote(
                        title = if (info.publishing) "Publishing…" else "Not published yet",
                        body = when {
                            info.publishing -> "It'll be live in a minute or two."
                            info.canPublish -> "Publish to put it online and see it here."
                            else -> "A project admin can publish it."
                        },
                    ) {
                        if (info.canPublish && !info.publishing) NomiButton(onClick = onPublish) { Text("Publish") }
                    }
                }
                info.backendSetupStep?.let { step ->
                    Text(
                        "Setting up its backend: $step",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
                info.backendSetupError?.let { Text(it, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.error) }
            }
        }
    }
}

/** The Code tab: the project's files; one opens read-only. */
@Composable
private fun CodePane(files: List<ProjectFileEntry>, current: OpenFile?, onOpen: (String) -> Unit, onClose: () -> Unit) {
    if (current != null) {
        Column(Modifier.fillMaxSize()) {
            Row(Modifier.fillMaxWidth().padding(horizontal = BarPaddingH), verticalAlignment = Alignment.CenterVertically) {
                IconButton(onClick = onClose) { NomiIcon(NomiIcons.ArrowBack, contentDescription = "All files") }
                Text(
                    current.path,
                    style = MaterialTheme.typography.labelLarge.copy(fontFamily = FontFamily.Monospace),
                    color = MaterialTheme.colorScheme.onSurface,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }
            when (val text = current.text) {
                Loadable.Loading -> CenteredLoading("Loading file")
                is Loadable.Failed -> RetryNote(text.message) { onOpen(current.path) }
                is Loadable.Ready -> Text(
                    text.value,
                    style = MaterialTheme.typography.bodySmall.copy(fontFamily = FontFamily.Monospace),
                    color = MaterialTheme.colorScheme.onSurface,
                    softWrap = false,
                    modifier = Modifier
                        .fillMaxSize()
                        .background(MaterialTheme.colorScheme.surfaceContainerLowest)
                        .verticalScroll(rememberScrollState())
                        .horizontalScroll(rememberScrollState())
                        .padding(CodePadding),
                )
            }
        }
        return
    }
    if (files.isEmpty()) {
        Box(Modifier.fillMaxSize().padding(PanePadding), contentAlignment = Alignment.Center) {
            EmptyNote("No files yet", "Koda writes the project's files as it builds.") {}
        }
        return
    }
    LazyColumn(Modifier.fillMaxSize(), contentPadding = PaddingValues(vertical = BarPaddingV)) {
        items(files, key = { it.path }) { file ->
            Row(
                Modifier
                    .fillMaxWidth()
                    .clickable(role = Role.Button, onClickLabel = "Open ${file.path}") { onOpen(file.path) }
                    .padding(horizontal = PanePadding, vertical = RowPaddingV),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(RowGap),
            ) {
                NomiIcon(NomiIcons.File, contentDescription = null, size = FileGlyph, tint = MaterialTheme.colorScheme.onSurfaceVariant)
                Column(Modifier.weight(FILL)) {
                    Text(
                        file.path.substringAfterLast('/'),
                        style = MaterialTheme.typography.bodyMedium.copy(fontFamily = FontFamily.Monospace),
                        color = MaterialTheme.colorScheme.onSurface,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                    val folder = file.path.substringBeforeLast('/', missingDelimiterValue = "")
                    if (folder.isNotEmpty()) {
                        Text(folder, style = NomiTheme.metaStyle, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 1, overflow = TextOverflow.Ellipsis)
                    }
                }
                Text(formatBytes(file.sizeBytes), style = NomiTheme.metaStyle, color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
        }
    }
}

private const val KIND_BUG = "bug"
private const val SEVERITY_MEDIUM = "medium"
private val ISSUE_KINDS = listOf("bug" to "Bug", "feature" to "Feature", "task" to "Task")
private val ISSUE_SEVERITIES = listOf("low" to "Low", "medium" to "Medium", "high" to "High", "critical" to "Critical")

/** `IssuesPanel.svelte`: open or closed issues, each closed or reopened in place, and a form to file one. */
@Composable
private fun ColumnScope.IssuesSheet(project: ProjectInfo, viewModel: ProjectWorkspaceViewModel) {
    val issues by viewModel.issues.collectAsStateWithLifecycle()
    val closed by viewModel.showClosed.collectAsStateWithLifecycle()
    var filing by rememberSaveable { mutableStateOf(false) }
    SheetTitle("Issues", project.name) {
        if (!filing) NomiButton(onClick = { filing = true }, variant = NomiButtonVariant.Tonal) { Text("New issue") }
    }
    if (filing) {
        NewIssueForm(
            onFile = { title, body, kind, severity ->
                viewModel.fileIssue(title, body, kind, severity)
                filing = false
            },
            onCancel = { filing = false },
        )
        return
    }
    val counts = (issues as? Loadable.Ready<ProjectIssueList>)?.value
    NomiButtonGroup(
        options = listOf(
            ButtonGroupOption("open", counts?.let { "Open (${it.open})" } ?: "Open"),
            ButtonGroupOption("closed", counts?.let { "Closed (${it.closed})" } ?: "Closed"),
        ),
        selected = if (closed) "closed" else "open",
        onSelect = { viewModel.showClosedIssues(it == "closed") },
        modifier = Modifier.fillMaxWidth().padding(vertical = NoticePaddingV),
    )
    when (val list = issues) {
        Loadable.Loading -> CenteredLoading("Loading issues")
        is Loadable.Failed -> RetryNote(list.message, viewModel::refreshIssues)
        is Loadable.Ready -> if (list.value.issues.isEmpty()) {
            EmptyNote(if (closed) "No closed issues" else "No open issues", "Bugs and to-dos for this project show up here.") {}
        } else {
            Column(Modifier.verticalScroll(rememberScrollState())) {
                list.value.issues.forEach { issue ->
                    IssueRow(issue, onToggle = { viewModel.setIssueOpen(issue.number, !issue.open) })
                    HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
                }
            }
        }
    }
}

@Composable
private fun IssueRow(issue: ProjectIssueInfo, onToggle: () -> Unit) {
    val colors = MaterialTheme.colorScheme
    Row(Modifier.fillMaxWidth().padding(vertical = RowPaddingV), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(RowGap)) {
        Column(Modifier.weight(FILL)) {
            Text(issue.title, style = MaterialTheme.typography.bodyLarge.copy(fontWeight = FontWeight.SemiBold), color = colors.onSurface)
            if (issue.body.isNotBlank()) {
                Text(issue.body, style = MaterialTheme.typography.bodySmall, color = colors.onSurfaceVariant, maxLines = ISSUE_BODY_LINES, overflow = TextOverflow.Ellipsis)
            }
            val meta = listOfNotNull("#${issue.number}", issue.kind.replaceFirstChar { it.uppercase() }, issue.severity, issue.createdBy).joinToString(" · ")
            Text(meta, style = NomiTheme.metaStyle, color = colors.onSurfaceVariant)
        }
        NomiButton(onClick = onToggle, variant = NomiButtonVariant.Text, size = NomiButtonSize.Small) {
            Text(if (issue.open) "Close" else "Reopen")
        }
    }
}

@Composable
private fun NewIssueForm(onFile: (title: String, body: String, kind: String, severity: String) -> Unit, onCancel: () -> Unit) {
    var title by rememberSaveable { mutableStateOf("") }
    var body by rememberSaveable { mutableStateOf("") }
    var kind by rememberSaveable { mutableStateOf(KIND_BUG) }
    var severity by rememberSaveable { mutableStateOf(SEVERITY_MEDIUM) }
    Column(verticalArrangement = Arrangement.spacedBy(SectionGap)) {
        NomiTextField(value = title, onValueChange = { title = it }, label = "Title", modifier = Modifier.fillMaxWidth())
        NomiTextField(value = body, onValueChange = { body = it }, label = "Details", singleLine = false, modifier = Modifier.fillMaxWidth())
        ChoiceRow(ISSUE_KINDS, kind) { kind = it }
        ChoiceRow(ISSUE_SEVERITIES, severity) { severity = it }
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(ToolsGap, Alignment.End)) {
            NomiButton(onClick = onCancel, variant = NomiButtonVariant.Text) { Text("Cancel") }
            NomiButton(onClick = { onFile(title, body, kind, severity) }, enabled = title.isNotBlank()) { Text("File issue") }
        }
    }
}

@Composable
private fun ChoiceRow(choices: List<Pair<String, String>>, selected: String, onSelect: (String) -> Unit) {
    Row(Modifier.horizontalScroll(rememberScrollState()), horizontalArrangement = Arrangement.spacedBy(ToolsGap)) {
        choices.forEach { (value, label) -> NomiFilterChip(label = label, selected = value == selected, onClick = { onSelect(value) }) }
    }
}

/** `DeployPanel.svelte`: the live address, Publish, and the publishes so far. */
@Composable
private fun ColumnScope.DeploySheet(deploy: Loadable<ProjectDeployInfo>, onPublish: () -> Unit, onRetry: () -> Unit) {
    val ready = (deploy as? Loadable.Ready)?.value
    SheetTitle("Deploy", ready?.url?.substringAfter(URL_SCHEME) ?: "Not published yet") {
        if (ready?.canPublish == true) {
            NomiButton(onClick = onPublish, enabled = !ready.publishing) { Text(if (ready.publishing) "Publishing…" else "Publish") }
        }
    }
    when (deploy) {
        Loadable.Loading -> CenteredLoading("Loading publishes")
        is Loadable.Failed -> RetryNote(deploy.message, onRetry)
        is Loadable.Ready -> if (deploy.value.publishes.isEmpty()) {
            EmptyNote("No publishes yet", "Each publish shows here with how it went.") {}
        } else {
            Column(Modifier.verticalScroll(rememberScrollState())) {
                deploy.value.publishes.forEach { publish ->
                    Column(Modifier.fillMaxWidth().padding(vertical = RowPaddingV)) {
                        Text(publishLabel(publish.status), style = MaterialTheme.typography.bodyLarge.copy(fontWeight = FontWeight.SemiBold), color = MaterialTheme.colorScheme.onSurface)
                        PublishLine(publish)
                        publish.error?.let { Text(it, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.error) }
                    }
                    HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
                }
            }
        }
    }
}

@Composable
private fun PublishLine(publish: ProjectPublishInfo) {
    val meta = listOfNotNull(publishLabel(publish.status), publish.createdBy, publish.createdAt.take(DATE_LENGTH)).joinToString(" · ")
    Text(meta, style = NomiTheme.metaStyle, color = MaterialTheme.colorScheme.onSurfaceVariant)
}

/** `YYYY-MM-DD` at the start of an RFC 3339 timestamp. */
private const val DATE_LENGTH = 10

private fun publishLabel(status: String) = when (status) {
    "running" -> "Publishing"
    "succeeded" -> "Live"
    "failed" -> "Failed"
    else -> status.replaceFirstChar { it.uppercase() }
}

/** `MembersPanel.svelte`: who works on the project. */
@Composable
private fun ColumnScope.MembersSheet(viewModel: ProjectWorkspaceViewModel) {
    val team by viewModel.team.collectAsStateWithLifecycle()
    val ready = (team as? Loadable.Ready<ProjectTeam>)?.value
    SheetTitle("Members", if (ready?.orgWide == true) "Everyone in the organization" else "Only the people added") {}
    when (val loaded = team) {
        Loadable.Loading -> CenteredLoading("Loading members")
        is Loadable.Failed -> RetryNote(loaded.message, viewModel::refreshTeam)
        is Loadable.Ready -> Column(Modifier.verticalScroll(rememberScrollState())) {
            loaded.value.people.forEach { person ->
                Row(Modifier.fillMaxWidth().padding(vertical = RowPaddingV), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(RowGap)) {
                    NomiAvatar(name = person.name, avatarUrl = person.avatarUrl, size = MemberAvatar)
                    Column(Modifier.weight(FILL)) {
                        Text(person.name, style = MaterialTheme.typography.bodyLarge, color = MaterialTheme.colorScheme.onSurface)
                        person.email?.let { Text(it, style = NomiTheme.metaStyle, color = MaterialTheme.colorScheme.onSurfaceVariant) }
                    }
                    Text(
                        if (person.isOwner) "Owner" else person.role.replaceFirstChar { it.uppercase() },
                        style = MaterialTheme.typography.labelMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }
        }
    }
}

@Composable
private fun SheetTitle(title: String, subtitle: String, actions: @Composable () -> Unit) {
    Row(Modifier.fillMaxWidth().padding(bottom = SectionGap), verticalAlignment = Alignment.CenterVertically) {
        Column(Modifier.weight(FILL)) {
            Text(title, style = MaterialTheme.typography.titleLarge.copy(fontWeight = FontWeight(TITLE_WEIGHT)), color = MaterialTheme.colorScheme.onSurface, modifier = Modifier.semantics { heading() })
            Text(subtitle, style = NomiTheme.metaStyle, color = MaterialTheme.colorScheme.onSurfaceVariant, maxLines = 1, overflow = TextOverflow.Ellipsis)
        }
        actions()
    }
}

@Composable
private fun Card(content: @Composable ColumnScope.() -> Unit) {
    Column(
        Modifier
            .fillMaxWidth()
            .clip(MaterialTheme.shapes.large)
            .background(MaterialTheme.colorScheme.surfaceContainerLowest)
            .padding(CardPadding),
        verticalArrangement = Arrangement.spacedBy(RowGap),
        content = content,
    )
}

@Composable
private fun EmptyNote(title: String, body: String, action: @Composable () -> Unit) {
    Column(
        Modifier.fillMaxWidth().padding(vertical = PanePadding),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(RowGap),
    ) {
        Text(title, style = MaterialTheme.typography.titleMedium.copy(fontWeight = FontWeight(TITLE_WEIGHT)), color = MaterialTheme.colorScheme.onSurface)
        Text(body, style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant, textAlign = TextAlign.Center, modifier = Modifier.widthIn(max = EmptyMaxWidth))
        action()
    }
}

@Composable
private fun RetryNote(message: String, onRetry: () -> Unit) {
    EmptyNote("Something went wrong", message) {
        NomiButton(onClick = onRetry, variant = NomiButtonVariant.Tonal) { Text("Try again") }
    }
}

@Composable
private fun CenteredLoading(label: String) {
    Box(Modifier.fillMaxWidth().padding(PanePadding), contentAlignment = Alignment.Center) {
        NomiLoadingIndicator(contentDescription = label)
    }
}
