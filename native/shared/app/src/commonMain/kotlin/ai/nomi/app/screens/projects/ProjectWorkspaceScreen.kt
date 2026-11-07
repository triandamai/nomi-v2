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
import ai.nomi.designsystem.ds.Crew
import ai.nomi.designsystem.ds.Ds
import ai.nomi.designsystem.ds.DsButton
import ai.nomi.designsystem.ds.DsButtonVariant
import ai.nomi.designsystem.ds.DsChevron
import ai.nomi.designsystem.ds.DsIcon
import ai.nomi.designsystem.ds.DsIconButton
import ai.nomi.designsystem.ds.DsIconButtonVariant
import ai.nomi.designsystem.ds.DsMenu
import ai.nomi.designsystem.ds.DsMotion
import ai.nomi.designsystem.ds.DsShapes
import ai.nomi.designsystem.ds.DsStatus
import ai.nomi.designsystem.ds.DsTabs
import ai.nomi.designsystem.ds.EmptyState
import ai.nomi.designsystem.ds.Mascot
import ai.nomi.designsystem.ds.MenuEntry
import ai.nomi.designsystem.ds.StatusKind
import ai.nomi.designsystem.ds.StepState
import ai.nomi.designsystem.ds.TabItem
import ai.nomi.designsystem.ds.builds.BuildingStep
import ai.nomi.designsystem.ds.builds.CodeView
import ai.nomi.designsystem.ds.builds.DeviceBar
import ai.nomi.designsystem.ds.builds.ExportPanel
import ai.nomi.designsystem.ds.builds.PhoneFrame
import ai.nomi.designsystem.ds.builds.PreviewBuilding
import ai.nomi.designsystem.ds.builds.PublishButton
import ai.nomi.designsystem.ds.builds.PublishState
import ai.nomi.designsystem.ds.builds.TermLine
import ai.nomi.designsystem.ds.builds.TermLineKind
import ai.nomi.designsystem.ds.builds.Terminal
import ai.nomi.designsystem.ds.chat.codeTokens
import ai.nomi.designsystem.icon.Lucide
import ai.nomi.designsystem.icon.NomiGlyph
import ai.nomi.designsystem.icon.NomiIcon
import ai.nomi.designsystem.icon.NomiIcons
import ai.nomi.designsystem.shape.AgentShape
import ai.nomi.designsystem.shape.NomiIdentity
import ai.nomi.designsystem.theme.NomiTheme
import ai.nomi.feature.chat.ChatScreen
import ai.nomi.feature.chat.ChatViewModel
import ai.nomi.feature.chat.formatBytes
import ai.nomi.network.ApiConfig
import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.expandVertically
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.shrinkVertically
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.consumeWindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBars
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.width
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
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
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
import org.koin.compose.koinInject
import org.koin.compose.viewmodel.koinViewModel
import org.koin.core.parameter.parametersOf

const val PROJECT_WORKSPACE_TAG = "project_workspace"

/** Keeps a project's tools view model apart from the workspace's on the same entry. */
private const val TOOLS_KEY_PREFIX = "tools:"

private const val TAB_CHAT = "chat"
private const val TAB_PREVIEW = "preview"
private const val TAB_CODE = "code"
private const val TAB_TERMINAL = "terminal"
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
private val StatusGap = 4.dp
private val UrlGlyph = 12.dp
private val SplitRule = 20.dp
private val Hairline = 1.dp
private val PhoneWidth = 300.dp
private val PreviewMascot = 48.dp
private val EmptyGlyph = 28.dp
/** Inside the phone frame the page is white, whatever the theme. */
private val PhoneInk = androidx.compose.ui.graphics.Color(0xFF111111)
private val PhoneInkSoft = androidx.compose.ui.graphics.Color(0xFF5F6368)

/** Which tool sheet is open over the workspace. */
enum class ProjectSheet { Issues, Deploy, Domain, Monitor, Database, Members, Settings, Export }

const val PROJECT_TAB_CHAT = TAB_CHAT
const val PROJECT_TAB_PREVIEW = TAB_PREVIEW
const val PROJECT_TAB_CODE = TAB_CODE
const val PROJECT_TAB_TERMINAL = TAB_TERMINAL

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
    val health by tools.health.collectAsStateWithLifecycle()
    val webUrl = koinInject<ApiConfig>().webUrl
    val terminal = when (val loaded = health.health) {
        Loadable.Loading -> TerminalLog(Loadable.Loading)
        is Loadable.Failed -> TerminalLog(Loadable.Failed(loaded.message))
        is Loadable.Ready -> TerminalLog(Loadable.Ready(loaded.value.logLines), loaded.value.logNote)
    }
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
                ProjectSheet.Export -> Unit
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
        terminal = terminal,
        onLoadTerminal = { tools.loadHealth() },
        onAskKoda = { text ->
            // Into the chat's composer to finish and send, like the web's code quote.
            chat.onDraftChange(text)
            tab = TAB_CHAT
        },
        onExport = { sheet = ProjectSheet.Export },
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
                    ProjectSheet.Export -> ExportSheet(project, webUrl, onClose = { sheet = null })
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

/** The project's deploy logs for the Terminal tab: loading, the lines, or why they couldn't load. */
data class TerminalLog(val lines: Loadable<List<String>>, val note: String? = null)

/**
 * The project's page (`projects/session/[id]`): its bar (brand, name and status, organization and live address,
 * Issues, Share with the other tools, Publish), then Chat / Preview / Code / Terminal. [chat] is the project's chat.
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
    terminal: TerminalLog = TerminalLog(Loadable.Loading),
    onLoadTerminal: () -> Unit = {},
    onAskKoda: (String) -> Unit = {},
    onExport: () -> Unit = {},
    chat: @Composable () -> Unit,
) {
    val ready = (deploy as? Loadable.Ready)?.value
    Column(modifier.fillMaxSize().background(NomiTheme.extendedColors.ground).testTag(PROJECT_WORKSPACE_TAG)) {
        Column(Modifier.statusBarsPadding()) {
            ProjectTopBar(project, ready, onBack, onPublish = onPublish, onOpenTool = onOpenTool, onExport = onExport)
            DsTabs(
                "Project",
                listOf(
                    TabItem(TAB_CHAT, "Chat"),
                    TabItem(TAB_PREVIEW, "Preview"),
                    TabItem(TAB_CODE, "Code"),
                    TabItem(TAB_TERMINAL, "Terminal"),
                ),
                tab,
                { next ->
                    if (next == TAB_TERMINAL) onLoadTerminal()
                    onTabChange(next)
                },
                Modifier.fillMaxWidth(),
            )
            AnimatedVisibility(notice != null, enter = expandVertically(DsMotion.spatial()) + fadeIn(), exit = shrinkVertically(DsMotion.fast()) + fadeOut()) {
                Text(
                    notice.orEmpty(),
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
        // The bar above already sits below the status bar. Tabs cross-fade like the web's pane.
        Box(Modifier.weight(FILL).consumeWindowInsets(WindowInsets.statusBars)) {
            val fadeIn = fadeIn(DsMotion.effects())
            val fadeOut = fadeOut(DsMotion.fast())
            AnimatedContent(tab, transitionSpec = { fadeIn togetherWith fadeOut }, label = "project-tab") { shown ->
                when (shown) {
                    TAB_CHAT -> chat()
                    TAB_PREVIEW -> PreviewPane(project, deploy, onPublish = onPublish, onRetry = onRetryDeploy)
                    TAB_TERMINAL -> TerminalPane(project, terminal, onRetry = onLoadTerminal, onAskKoda = onAskKoda)
                    else -> CodePane(project.files, openFile, onOpenFile, onCloseFile, onAskKoda)
                }
            }
        }
    }
}

/** The web's PublishButton state from the latest publish. */
private fun ProjectDeployInfo.publishState(): PublishState = when {
    publishing -> PublishState.Publishing
    publishes.firstOrNull()?.status == PUBLISH_FAILED -> PublishState.Failed
    canPublish -> PublishState.Ready
    else -> PublishState.Idle
}

/**
 * `.topbar`: back, the brand, the project's name with its status, its organization and live address; then the
 * tools: Issues with its count, Share with the rest of the tools under its arrow, and Publish.
 */
@Composable
private fun ProjectTopBar(
    project: ProjectInfo,
    deploy: ProjectDeployInfo?,
    onBack: () -> Unit,
    onPublish: () -> Unit,
    onOpenTool: (ProjectSheet) -> Unit,
    onExport: () -> Unit,
) {
    val uri = LocalUriHandler.current
    Column(Modifier.fillMaxWidth().padding(horizontal = BarPaddingH, vertical = BarPaddingV)) {
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(ToolsGap)) {
            DsIconButton(Lucide.ArrowLeft, "All projects", onBack, variant = DsIconButtonVariant.Ghost)
            Mascot(Crew.Nomi, BrandSize)
            Column(Modifier.weight(FILL)) {
                Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(ToolsGap)) {
                    Text(
                        project.name,
                        style = MaterialTheme.typography.titleMedium.copy(fontWeight = FontWeight(TITLE_WEIGHT)),
                        color = Ds.text,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                        modifier = Modifier.weight(FILL, fill = false).semantics { heading() },
                    )
                    StatusChip(project.status)
                }
                val liveUrl = deploy?.url
                Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(StatusGap)) {
                    Text(project.orgName, style = Ds.caption, color = Ds.text2, maxLines = 1)
                    when {
                        liveUrl != null -> Row(
                            Modifier.clip(DsShapes.xs).clickable(role = Role.Button, onClickLabel = "Open the live app") { runCatching { uri.openUri(liveUrl) } },
                            verticalAlignment = Alignment.CenterVertically,
                            horizontalArrangement = Arrangement.spacedBy(StatusGap),
                        ) {
                            Text("· ${liveUrl.substringAfter(URL_SCHEME).trimEnd('/')}", style = Ds.caption, color = Ds.accent, maxLines = 1, overflow = TextOverflow.Ellipsis)
                            DsIcon(Lucide.ExternalLink, UrlGlyph, Ds.accent)
                        }
                        deploy?.publishing == true -> Text("· Publishing…", style = Ds.caption, color = Ds.text2)
                    }
                }
            }
        }
        Row(
            Modifier.fillMaxWidth().padding(start = ToolsPaddingH, end = ToolsPaddingH, top = BarPaddingV),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(ToolsGap),
        ) {
            DsButton(
                if (project.openIssues > 0) "Issues · ${project.openIssues}" else "Issues",
                { onOpenTool(ProjectSheet.Issues) },
                variant = DsButtonVariant.Secondary,
                iconStart = Lucide.Alert,
            )
            ShareSplit(onShare = { onOpenTool(ProjectSheet.Members) }, onOpenTool = onOpenTool, onExport = onExport)
            Spacer(Modifier.weight(FILL))
            if (deploy != null && (deploy.canPublish || deploy.publishing)) {
                PublishButton(deploy.publishState(), onPublish = onPublish, onRetry = onPublish)
            }
        }
    }
}

/** The web's Share split button: Share opens the people; its arrow holds the project's other tools and Export. */
@Composable
private fun ShareSplit(onShare: () -> Unit, onOpenTool: (ProjectSheet) -> Unit, onExport: () -> Unit) {
    Row(Modifier.clip(DsShapes.full).border(Hairline, Ds.borderStrong, DsShapes.full), verticalAlignment = Alignment.CenterVertically) {
        Row(
            Modifier.heightIn(min = TOUCH_TARGET).clickable(role = Role.Button, onClick = onShare).padding(start = ToolPaddingH, end = ToolInnerGap),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(ToolInnerGap),
        ) {
            DsIcon(Lucide.Share, ToolGlyph, Ds.text2)
            Text("Share", style = MaterialTheme.typography.labelLarge, color = Ds.text)
        }
        Box(Modifier.width(Hairline).height(SplitRule).background(Ds.borderStrong))
        DsMenu(
            listOf(
                MenuEntry.Item("Deploy", Lucide.Rocket, onSelect = { onOpenTool(ProjectSheet.Deploy) }),
                MenuEntry.Item("Domain", Lucide.Globe, onSelect = { onOpenTool(ProjectSheet.Domain) }),
                MenuEntry.Item("Monitor", Lucide.Activity, onSelect = { onOpenTool(ProjectSheet.Monitor) }),
                MenuEntry.Item("Database", Lucide.Database, onSelect = { onOpenTool(ProjectSheet.Database) }),
                MenuEntry.Divider,
                MenuEntry.Item("Export", Lucide.Download, onSelect = onExport),
                MenuEntry.Item("Settings", Lucide.Settings, onSelect = { onOpenTool(ProjectSheet.Settings) }),
            ),
        ) { open, expanded ->
            Box(
                Modifier.size(TOUCH_TARGET).clickable(role = Role.Button, onClickLabel = "More tools", onClick = open),
                contentAlignment = Alignment.Center,
            ) { DsChevron(expanded, ToolGlyph, Ds.text2) }
        }
    }
}

/** `.topbar__status`: planning, Koda building (his shape moving), ready. */
@Composable
private fun StatusChip(status: String) {
    when (status) {
        "building" -> DsStatus(StatusKind.Working, statusLabel(status), agent = Crew.Koda, small = true)
        STATUS_READY -> DsStatus(StatusKind.Live, statusLabel(status), small = true)
        "failed" -> DsStatus(StatusKind.Failed, statusLabel(status), small = true)
        else -> DsStatus(StatusKind.Idle, statusLabel(status), small = true)
    }
}

private const val STATUS_READY = "ready"
private const val PUBLISH_FAILED = "failed"

private fun statusLabel(status: String) = when (status) {
    "planning" -> "Planning"
    "building" -> "Building"
    STATUS_READY -> "Ready"
    "failed" -> "Failed"
    else -> status.replaceFirstChar { it.uppercase() }
}

private val TOUCH_TARGET = 44.dp

/**
 * The Preview tab: the web's address bar (where it runs, open in the browser) and device bar, then the app in a
 * phone frame: live, being published (its steps), or not yet published.
 */
@Composable
private fun PreviewPane(project: ProjectInfo, deploy: Loadable<ProjectDeployInfo>, onPublish: () -> Unit, onRetry: () -> Unit) {
    val uri = LocalUriHandler.current
    Column(Modifier.fillMaxSize().verticalScroll(rememberScrollState()).padding(PanePadding), verticalArrangement = Arrangement.spacedBy(SectionGap)) {
        when (deploy) {
            Loadable.Loading -> CenteredLoading("Loading preview")
            is Loadable.Failed -> RetryNote(deploy.message, onRetry)
            is Loadable.Ready -> {
                val info = deploy.value
                val url = info.url
                AddressBar(
                    status = when {
                        info.publishing -> "Publishing"
                        url != null -> "Live"
                        else -> "Stopped"
                    },
                    running = url != null,
                    address = url?.substringAfter(URL_SCHEME)?.trimEnd('/') ?: "not published",
                    onOpen = url?.let { { runCatching { uri.openUri(it) } } },
                )
                DeviceBar("Phone", onDevice = {})
                Box(Modifier.fillMaxWidth(), contentAlignment = Alignment.TopCenter) {
                    PhoneFrame(width = PhoneWidth) {
                        when {
                            info.publishing -> PreviewBuilding(
                                listOf(
                                    BuildingStep("Writing code", StepState.Done),
                                    BuildingStep("Building", StepState.Running),
                                    BuildingStep("Publishing", StepState.Waiting),
                                ),
                            )
                            url != null -> LivePreview(project, url, info.publishes.firstOrNull(), onOpen = { runCatching { uri.openUri(url) } })
                            else -> NotPublished(info, onPublish)
                        }
                    }
                }
                info.backendSetupStep?.let { step -> Text("Setting up its backend: $step", style = Ds.caption, color = Ds.text2) }
                info.backendSetupError?.let { Text(it, style = Ds.caption, color = Ds.danger) }
            }
        }
    }
}

/** `PreviewAddressBar`: where the preview runs (a dot for running), the address, and open in the browser. */
@Composable
private fun AddressBar(status: String, running: Boolean, address: String, onOpen: (() -> Unit)?) {
    Row(
        Modifier.fillMaxWidth().clip(DsShapes.full).background(Ds.surface).border(Hairline, Ds.border, DsShapes.full).padding(start = ToolPaddingH, end = BarPaddingH),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(ToolInnerGap),
    ) {
        DsStatus(if (running) StatusKind.Live else StatusKind.Idle, status, small = true)
        Text(address, style = Ds.mono, color = Ds.text2, maxLines = 1, overflow = TextOverflow.Ellipsis, modifier = Modifier.weight(FILL))
        if (onOpen != null) DsIconButton(Lucide.ExternalLink, "Open in the browser", onOpen, variant = DsIconButtonVariant.Ghost, small = true)
    }
}

/** The live app in the frame: its name, address, latest publish, and open it. */
@Composable
private fun ColumnScope.LivePreview(project: ProjectInfo, url: String, latest: ProjectPublishInfo?, onOpen: () -> Unit) {
    Column(
        Modifier.weight(FILL).fillMaxWidth().padding(PanePadding),
        verticalArrangement = Arrangement.spacedBy(RowGap, Alignment.CenterVertically),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        Mascot(Crew.Koda, PreviewMascot)
        Text(project.name, style = MaterialTheme.typography.titleMedium, color = PhoneInk)
        Text(url.substringAfter(URL_SCHEME).trimEnd('/'), style = Ds.mono, color = PhoneInkSoft)
        latest?.let { Text(publishLabel(it.status), style = Ds.caption, color = PhoneInkSoft) }
        DsButton("Open the app", onOpen, iconEnd = Lucide.ExternalLink)
    }
}

@Composable
private fun ColumnScope.NotPublished(info: ProjectDeployInfo, onPublish: () -> Unit) {
    Column(
        Modifier.weight(FILL).fillMaxWidth().padding(PanePadding),
        verticalArrangement = Arrangement.spacedBy(RowGap, Alignment.CenterVertically),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        DsIcon(Lucide.Rocket, EmptyGlyph, PhoneInkSoft)
        Text("Not published yet", style = MaterialTheme.typography.titleSmall, color = PhoneInk)
        Text(
            if (info.canPublish) "Publish to put it online and see it here." else "A project admin can publish it.",
            style = Ds.caption,
            color = PhoneInkSoft,
            textAlign = TextAlign.Center,
        )
        if (info.canPublish) DsButton("Publish", onPublish, iconStart = Lucide.Rocket)
    }
}

/** The Terminal tab: the app's latest logs, read-only, with asking Koda to fix what failed. */
@Composable
private fun TerminalPane(project: ProjectInfo, log: TerminalLog, onRetry: () -> Unit, onAskKoda: (String) -> Unit) {
    Box(Modifier.fillMaxSize().padding(PanePadding)) {
        when (val lines = log.lines) {
            Loadable.Loading -> CenteredLoading("Loading logs")
            is Loadable.Failed -> RetryNote(lines.message, onRetry)
            is Loadable.Ready -> {
                val shown = lines.value.map(::termLine).ifEmpty { listOf(TermLine(TermLineKind.Out, log.note ?: "No logs yet. They show once the app is published.")) }
                val failed = shown.any { it.kind == TermLineKind.Error }
                Terminal(
                    run = "logs · ${project.name}",
                    note = "read-only",
                    lines = shown,
                    modifier = Modifier.fillMaxSize(),
                    failureTitle = if (failed) "The app logged errors" else null,
                    onFix = if (failed) ({ onAskKoda("The app is logging errors. Look at its latest logs and fix what's failing.") }) else null,
                )
            }
        }
    }
}

private val ErrorWords = Regex("\\b(error|fatal|panic|exception|failed)\\b", RegexOption.IGNORE_CASE)
private val WarnWords = Regex("\\b(warn|warning)\\b", RegexOption.IGNORE_CASE)

private fun termLine(text: String): TermLine = TermLine(
    when {
        text.isBlank() -> TermLineKind.Blank
        text.startsWith("$ ") -> TermLineKind.Command
        ErrorWords.containsMatchIn(text) -> TermLineKind.Error
        WarnWords.containsMatchIn(text) -> TermLineKind.Warn
        else -> TermLineKind.Out
    },
    text.removePrefix("$ "),
)

/**
 * The Code tab: the project's files; one opens in the design system's code view, read-only, where a file or the
 * lines picked can be taken to Koda in the chat.
 */
@Composable
private fun CodePane(files: List<ProjectFileEntry>, current: OpenFile?, onOpen: (String) -> Unit, onClose: () -> Unit, onAskKoda: (String) -> Unit) {
    if (current != null) {
        Column(Modifier.fillMaxSize()) {
            Row(Modifier.fillMaxWidth().padding(horizontal = BarPaddingH), verticalAlignment = Alignment.CenterVertically) {
                DsIconButton(Lucide.ArrowLeft, "All files", onClose, variant = DsIconButtonVariant.Ghost)
                Text("FILES", style = Ds.labelCaps, color = Ds.text3)
            }
            when (val text = current.text) {
                Loadable.Loading -> CenteredLoading("Loading file")
                is Loadable.Failed -> RetryNote(text.message) { onOpen(current.path) }
                is Loadable.Ready -> {
                    val lines = remember(text.value) { codeTokens(text.value) }
                    CodeView(
                        tree = emptyList(),
                        path = current.path.substringAfterLast('/'),
                        lines = lines,
                        onAsk = { onAskKoda("About `${current.path}`: ") },
                        onAskLines = { from, to ->
                            onAskKoda("About `${current.path}` lines ${if (from == to) "$from" else "$from–$to"}: ")
                        },
                        modifier = Modifier.fillMaxSize().padding(horizontal = CodePadding).padding(bottom = CodePadding),
                        meta = "read-only",
                        showTree = false,
                    )
                }
            }
        }
        return
    }
    if (files.isEmpty()) {
        Box(Modifier.fillMaxSize().padding(PanePadding), contentAlignment = Alignment.Center) {
            EmptyState(Lucide.FileCode, "No files yet", "Koda writes the project's files as it builds.")
        }
        return
    }
    LazyColumn(Modifier.fillMaxSize(), contentPadding = PaddingValues(vertical = BarPaddingV)) {
        item { Text("FILES · ${files.size}", style = Ds.labelCaps, color = Ds.text3, modifier = Modifier.padding(horizontal = PanePadding, vertical = BarPaddingV)) }
        items(files, key = { it.path }) { file ->
            Row(
                Modifier
                    .fillMaxWidth()
                    .animateItem(fadeInSpec = DsMotion.effects(), placementSpec = DsMotion.spatial(), fadeOutSpec = DsMotion.fast())
                    .clickable(role = Role.Button, onClickLabel = "Open ${file.path}") { onOpen(file.path) }
                    .padding(horizontal = PanePadding, vertical = RowPaddingV),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(RowGap),
            ) {
                DsIcon(Lucide.FileCode, FileGlyph, Ds.text3)
                Column(Modifier.weight(FILL)) {
                    Text(file.path.substringAfterLast('/'), style = Ds.mono, color = Ds.text, maxLines = 1, overflow = TextOverflow.Ellipsis)
                    val folder = file.path.substringBeforeLast('/', missingDelimiterValue = "")
                    if (folder.isNotEmpty()) Text(folder, style = Ds.caption, color = Ds.text3, maxLines = 1, overflow = TextOverflow.Ellipsis)
                }
                Text(formatBytes(file.sizeBytes), style = Ds.label, color = Ds.text3)
            }
        }
    }
}

/** Exporting the project (`ExportDialog`): its repo, database and env names; download the zip or push to GitHub. */
@Composable
internal fun ExportSheet(project: ProjectInfo, webUrl: String, onClose: () -> Unit) {
    val uri = LocalUriHandler.current
    val bytes = project.files.sumOf { it.sizeBytes }
    ExportPanel(
        project = project.name,
        repo = "${project.files.size} files · ${formatBytes(bytes)}",
        database = if (project.relay) "Relay · schema and data" else "PostgreSQL · schema and data",
        onPush = { runCatching { uri.openUri("${webUrl.trimEnd('/')}/projects/session/${project.sessionId}") } },
        onDownload = { runCatching { uri.openUri("${webUrl.trimEnd('/')}/projects/${project.id}/api/export/zip") } },
        onClose = onClose,
    )
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
