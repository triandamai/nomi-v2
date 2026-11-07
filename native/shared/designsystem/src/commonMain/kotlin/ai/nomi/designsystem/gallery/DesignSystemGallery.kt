package ai.nomi.designsystem.gallery

import ai.nomi.designsystem.ds.ActivityBlock
import ai.nomi.designsystem.ds.ActivityFile
import ai.nomi.designsystem.ds.ActivityState
import ai.nomi.designsystem.ds.AgentByline
import ai.nomi.designsystem.ds.AttachedFile
import ai.nomi.designsystem.ds.CheckMark
import ai.nomi.designsystem.ds.ChipKind
import ai.nomi.designsystem.ds.Crew
import ai.nomi.designsystem.ds.CrewRow
import ai.nomi.designsystem.ds.Ds
import ai.nomi.designsystem.ds.DsArrive
import ai.nomi.designsystem.ds.DsAvatar
import ai.nomi.designsystem.ds.DsButton
import ai.nomi.designsystem.ds.DsButtonVariant
import ai.nomi.designsystem.ds.DsCheckbox
import ai.nomi.designsystem.ds.DsChip
import ai.nomi.designsystem.ds.DsDialog
import ai.nomi.designsystem.ds.DsIconButton
import ai.nomi.designsystem.ds.DsIconButtonVariant
import ai.nomi.designsystem.ds.DsLink
import ai.nomi.designsystem.ds.DsMenu
import ai.nomi.designsystem.ds.DsSearch
import ai.nomi.designsystem.ds.DsSegmented
import ai.nomi.designsystem.ds.DsSidebar
import ai.nomi.designsystem.ds.DsStatus
import ai.nomi.designsystem.ds.DsTabs
import ai.nomi.designsystem.ds.DsTag
import ai.nomi.designsystem.ds.DsTextField
import ai.nomi.designsystem.ds.DsToast
import ai.nomi.designsystem.ds.DsToastRegion
import ai.nomi.designsystem.ds.DsToasts
import ai.nomi.designsystem.ds.DsToggle
import ai.nomi.designsystem.ds.DsTooltip
import ai.nomi.designsystem.ds.EmptyState
import ai.nomi.designsystem.ds.ErrorAction
import ai.nomi.designsystem.ds.ErrorCallout
import ai.nomi.designsystem.ds.ErrorDetail
import ai.nomi.designsystem.ds.EventKind
import ai.nomi.designsystem.ds.EventRow
import ai.nomi.designsystem.ds.FileState
import ai.nomi.designsystem.ds.KVRow
import ai.nomi.designsystem.ds.Kbd
import ai.nomi.designsystem.ds.KeyHint
import ai.nomi.designsystem.ds.KeyHintItem
import ai.nomi.designsystem.ds.LiveStep
import ai.nomi.designsystem.ds.Mascot
import ai.nomi.designsystem.ds.MenuEntry
import ai.nomi.designsystem.ds.MessageAgent
import ai.nomi.designsystem.ds.MessageUser
import ai.nomi.designsystem.ds.MessageVersion
import ai.nomi.designsystem.ds.Meter
import ai.nomi.designsystem.ds.NavGroupLabel
import ai.nomi.designsystem.ds.NavItem
import ai.nomi.designsystem.ds.ProjectRow
import ai.nomi.designsystem.ds.ProjectStatus
import ai.nomi.designsystem.ds.RecentItem
import ai.nomi.designsystem.ds.RecentKind
import ai.nomi.designsystem.ds.RecentsList
import ai.nomi.designsystem.ds.SectionLabel
import ai.nomi.designsystem.ds.SegmentOption
import ai.nomi.designsystem.ds.SendStatus
import ai.nomi.designsystem.ds.SettingsNav
import ai.nomi.designsystem.ds.SettingsNavGroup
import ai.nomi.designsystem.ds.SettingsNavItem
import ai.nomi.designsystem.ds.SkeletonRow
import ai.nomi.designsystem.ds.StatusKind
import ai.nomi.designsystem.ds.StepState
import ai.nomi.designsystem.ds.SuggestionRow
import ai.nomi.designsystem.ds.TabItem
import ai.nomi.designsystem.ds.TagKind
import ai.nomi.designsystem.ds.ToastKind
import ai.nomi.designsystem.ds.TooltipBubble
import ai.nomi.designsystem.ds.UsageCard
import ai.nomi.designsystem.ds.UserRow
import ai.nomi.designsystem.ds.VersionSwitcher
import ai.nomi.designsystem.ds.WorkspaceChip
import ai.nomi.designsystem.ds.builds.BuildDone
import ai.nomi.designsystem.ds.builds.BuildFile
import ai.nomi.designsystem.ds.builds.BuildInvite
import ai.nomi.designsystem.ds.builds.BuildPlan
import ai.nomi.designsystem.ds.builds.BuildRunning
import ai.nomi.designsystem.ds.builds.BuildSharePanel
import ai.nomi.designsystem.ds.builds.BuildingStep
import ai.nomi.designsystem.ds.builds.CodeQuote
import ai.nomi.designsystem.ds.builds.CodeView
import ai.nomi.designsystem.ds.builds.DeviceBar
import ai.nomi.designsystem.ds.builds.EntryKind
import ai.nomi.designsystem.ds.builds.ExportPanel
import ai.nomi.designsystem.ds.builds.FileList
import ai.nomi.designsystem.ds.builds.FirstRun
import ai.nomi.designsystem.ds.builds.FollowBar
import ai.nomi.designsystem.ds.builds.InviteRole
import ai.nomi.designsystem.ds.builds.JoinedNote
import ai.nomi.designsystem.ds.builds.PeerCursor
import ai.nomi.designsystem.ds.builds.PhoneFrame
import ai.nomi.designsystem.ds.builds.Presence
import ai.nomi.designsystem.ds.builds.PresenceMenu
import ai.nomi.designsystem.ds.builds.PresentPerson
import ai.nomi.designsystem.ds.builds.Presenter
import ai.nomi.designsystem.ds.builds.PreviewBuilding
import ai.nomi.designsystem.ds.builds.PublishButton
import ai.nomi.designsystem.ds.builds.PublishRequest
import ai.nomi.designsystem.ds.builds.PublishState
import ai.nomi.designsystem.ds.builds.RequestQueue
import ai.nomi.designsystem.ds.builds.ResizeHandle
import ai.nomi.designsystem.ds.builds.RoleMenu
import ai.nomi.designsystem.ds.builds.SharePreviewCard
import ai.nomi.designsystem.ds.builds.TeammateMessage
import ai.nomi.designsystem.ds.builds.TermLine
import ai.nomi.designsystem.ds.builds.TermLineKind
import ai.nomi.designsystem.ds.builds.Terminal
import ai.nomi.designsystem.ds.builds.TreeEntry
import ai.nomi.designsystem.ds.builds.Typing
import ai.nomi.designsystem.ds.builds.WorkspaceSheet
import ai.nomi.designsystem.ds.chat.Alternative
import ai.nomi.designsystem.ds.chat.ApprovalCard
import ai.nomi.designsystem.ds.chat.ApprovalStatus
import ai.nomi.designsystem.ds.chat.AttachMenu
import ai.nomi.designsystem.ds.chat.AttachmentChip
import ai.nomi.designsystem.ds.chat.BackgroundNote
import ai.nomi.designsystem.ds.chat.Banner
import ai.nomi.designsystem.ds.chat.BannerKind
import ai.nomi.designsystem.ds.chat.Boundary
import ai.nomi.designsystem.ds.chat.Cart
import ai.nomi.designsystem.ds.chat.CartItem
import ai.nomi.designsystem.ds.chat.Cell
import ai.nomi.designsystem.ds.chat.CellTone
import ai.nomi.designsystem.ds.chat.ChatMenu
import ai.nomi.designsystem.ds.chat.Citation
import ai.nomi.designsystem.ds.chat.ClarifyAnswered
import ai.nomi.designsystem.ds.chat.ClarifyCard
import ai.nomi.designsystem.ds.chat.ClarifyOption
import ai.nomi.designsystem.ds.chat.ClarifyQuestion
import ai.nomi.designsystem.ds.chat.ClarifyReview
import ai.nomi.designsystem.ds.chat.CodeBlock
import ai.nomi.designsystem.ds.chat.CodeToken
import ai.nomi.designsystem.ds.chat.CodeTokenKind
import ai.nomi.designsystem.ds.chat.ComposerLimit
import ai.nomi.designsystem.ds.chat.ConnectPrompt
import ai.nomi.designsystem.ds.chat.CopiedNote
import ai.nomi.designsystem.ds.chat.CrewWorking
import ai.nomi.designsystem.ds.chat.DataTable
import ai.nomi.designsystem.ds.chat.DateDivider
import ai.nomi.designsystem.ds.chat.DecisionCard
import ai.nomi.designsystem.ds.chat.DecisionOption
import ai.nomi.designsystem.ds.chat.DeleteReplyDialog
import ai.nomi.designsystem.ds.chat.DeletedReply
import ai.nomi.designsystem.ds.chat.Diff
import ai.nomi.designsystem.ds.chat.DiffHunk
import ai.nomi.designsystem.ds.chat.DiffLine
import ai.nomi.designsystem.ds.chat.DiffLineKind
import ai.nomi.designsystem.ds.chat.DiffRequest
import ai.nomi.designsystem.ds.chat.DislikeFeedback
import ai.nomi.designsystem.ds.chat.DraftPlanCard
import ai.nomi.designsystem.ds.chat.DraftStep
import ai.nomi.designsystem.ds.chat.FeedbackSent
import ai.nomi.designsystem.ds.chat.FollowUps
import ai.nomi.designsystem.ds.chat.FormCard
import ai.nomi.designsystem.ds.chat.Handoff
import ai.nomi.designsystem.ds.chat.Hyperlink
import ai.nomi.designsystem.ds.chat.JumpToLatest
import ai.nomi.designsystem.ds.chat.KeyValue
import ai.nomi.designsystem.ds.chat.LinkPreview
import ai.nomi.designsystem.ds.chat.LoadingOlder
import ai.nomi.designsystem.ds.chat.MemorySaved
import ai.nomi.designsystem.ds.chat.MemoryUsed
import ai.nomi.designsystem.ds.chat.MentionCrew
import ai.nomi.designsystem.ds.chat.MentionFile
import ai.nomi.designsystem.ds.chat.MentionOption
import ai.nomi.designsystem.ds.chat.MentionPicker
import ai.nomi.designsystem.ds.chat.MessageEditor
import ai.nomi.designsystem.ds.chat.MoveDialog
import ai.nomi.designsystem.ds.chat.MoveProject
import ai.nomi.designsystem.ds.chat.NewDivider
import ai.nomi.designsystem.ds.chat.PlanApproval
import ai.nomi.designsystem.ds.chat.PlanCard
import ai.nomi.designsystem.ds.chat.PlanStep
import ai.nomi.designsystem.ds.chat.QuickReplies
import ai.nomi.designsystem.ds.chat.Reasoning
import ai.nomi.designsystem.ds.chat.RegenerateMenu
import ai.nomi.designsystem.ds.chat.RegenerateModel
import ai.nomi.designsystem.ds.chat.Reminder
import ai.nomi.designsystem.ds.chat.RenameField
import ai.nomi.designsystem.ds.chat.ReplyFeedback
import ai.nomi.designsystem.ds.chat.ReplyStatus
import ai.nomi.designsystem.ds.chat.ReplyText
import ai.nomi.designsystem.ds.chat.ResponseActions
import ai.nomi.designsystem.ds.chat.ReviewAnswer
import ai.nomi.designsystem.ds.chat.RunOption
import ai.nomi.designsystem.ds.chat.SelectionCard
import ai.nomi.designsystem.ds.chat.SelectionItem
import ai.nomi.designsystem.ds.chat.SessionExpired
import ai.nomi.designsystem.ds.chat.ShareDialog
import ai.nomi.designsystem.ds.chat.SharePreview
import ai.nomi.designsystem.ds.chat.Source
import ai.nomi.designsystem.ds.chat.SourceKind
import ai.nomi.designsystem.ds.chat.Sources
import ai.nomi.designsystem.ds.chat.StepOwner
import ai.nomi.designsystem.ds.chat.StopHint
import ai.nomi.designsystem.ds.chat.StoppedTask
import ai.nomi.designsystem.ds.chat.SupervisorStopped
import ai.nomi.designsystem.ds.chat.TaskState
import ai.nomi.designsystem.ds.chat.Thought
import ai.nomi.designsystem.ds.chat.ThoughtStep
import ai.nomi.designsystem.ds.chat.ThoughtSteps
import ai.nomi.designsystem.ds.chat.VoiceRecorder
import ai.nomi.designsystem.ds.chat.VoiceTranscribed
import ai.nomi.designsystem.ds.mobile.ActionBar
import ai.nomi.designsystem.ds.mobile.Ask
import ai.nomi.designsystem.ds.mobile.AskBar
import ai.nomi.designsystem.ds.mobile.AskQueue
import ai.nomi.designsystem.ds.mobile.AskState
import ai.nomi.designsystem.ds.mobile.BuildPlanCard
import ai.nomi.designsystem.ds.mobile.BuildProgress
import ai.nomi.designsystem.ds.mobile.Byline
import ai.nomi.designsystem.ds.mobile.ChangeReady
import ai.nomi.designsystem.ds.mobile.ChatRow
import ai.nomi.designsystem.ds.mobile.DnsRecord
import ai.nomi.designsystem.ds.mobile.DnsState
import ai.nomi.designsystem.ds.mobile.ErrorCard
import ai.nomi.designsystem.ds.mobile.LiveCard
import ai.nomi.designsystem.ds.mobile.MobileApprovalCard
import ai.nomi.designsystem.ds.mobile.MobileComposer
import ai.nomi.designsystem.ds.mobile.MobileProgress
import ai.nomi.designsystem.ds.mobile.NavBar
import ai.nomi.designsystem.ds.mobile.Person
import ai.nomi.designsystem.ds.mobile.PhoneOption
import ai.nomi.designsystem.ds.mobile.PhoneToast
import ai.nomi.designsystem.ds.mobile.PhoneToastKind
import ai.nomi.designsystem.ds.mobile.ProjectNav
import ai.nomi.designsystem.ds.mobile.PublishReady
import ai.nomi.designsystem.ds.mobile.SheetHeader
import ai.nomi.designsystem.ds.mobile.SheetRow
import ai.nomi.designsystem.ds.mobile.StatList
import ai.nomi.designsystem.ds.mobile.StatRow
import ai.nomi.designsystem.ds.mobile.TeammateBubble
import ai.nomi.designsystem.ds.mobile.UserMessage
import ai.nomi.designsystem.ds.mobile.WorkspaceSwitch
import ai.nomi.designsystem.ds.mobile.WorkspaceView
import ai.nomi.designsystem.icon.Lucide
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

/** The three sheets, as on the web's /design-system. */
enum class GallerySheet(val label: String) { Primitives("Primitives"), Chat("Chat responses"), Build("Build") }

private val SheetMaxWidth = 760.dp

/**
 * The design system on the phone: every component from the web's three sheets (Primitives, Chat responses, Build),
 * live, with the same story data, so both can be compared side by side. Documentation, not app text.
 */
@Composable
fun DesignSystemGallery(modifier: Modifier = Modifier, initial: GallerySheet = GallerySheet.Primitives, onBack: (() -> Unit)? = null) {
    var sheet by rememberSaveable { mutableStateOf(initial) }
    val toasts = remember { DsToasts() }
    Box(modifier.fillMaxSize().background(Ds.bg)) {
        Column(Modifier.fillMaxSize()) {
            Row(Modifier.fillMaxWidth().padding(start = 4.dp, end = 16.dp, top = 8.dp), verticalAlignment = Alignment.CenterVertically) {
                if (onBack != null) DsIconButton(Lucide.ArrowLeft, "Back", onBack, variant = DsIconButtonVariant.Ghost)
                Text("Design system", style = MaterialTheme.typography.titleLarge, color = Ds.text, modifier = Modifier.padding(start = if (onBack == null) 12.dp else 0.dp))
            }
            DsTabs("Design sheets", GallerySheet.entries.map { TabItem(it.name, it.label) }, sheet.name, { sheet = GallerySheet.valueOf(it) }) { current ->
                LazyColumn(
                    Modifier.fillMaxSize(),
                    contentPadding = PaddingValues(horizontal = 16.dp, vertical = 24.dp),
                    verticalArrangement = Arrangement.spacedBy(40.dp),
                    horizontalAlignment = Alignment.CenterHorizontally,
                ) {
                    items(gallerySections(GallerySheet.valueOf(current), toasts), key = { it.label }) { section ->
                        DsArrive { GallerySectionView(section) }
                    }
                }
            }
        }
        DsToastRegion(toasts, Modifier.align(Alignment.BottomCenter).padding(16.dp))
    }
}

/** One section of a sheet: its label, how many it shows, and the components. */
class GallerySection(val label: String, val count: Int?, val content: @Composable ColumnScope.() -> Unit)

/** Collects a sheet's sections in order. */
class SectionsBuilder {
    internal val sections = mutableListOf<GallerySection>()
    fun section(label: String, count: Int? = null, content: @Composable ColumnScope.() -> Unit) {
        sections += GallerySection(label, count, content)
    }
}

/** Every section of [sheet], in the web's order; [toasts] is where the primitives' toast demo shows. */
fun gallerySections(sheet: GallerySheet, toasts: DsToasts): List<GallerySection> = SectionsBuilder().apply {
    when (sheet) {
        GallerySheet.Primitives -> primitives(toasts)
        GallerySheet.Chat -> chatResponses()
        GallerySheet.Build -> buildSheet()
    }
}.sections

/** A section drawn as the gallery shows it: its label over its components. */
@Composable
fun GallerySectionView(section: GallerySection, modifier: Modifier = Modifier) {
    Column(modifier.widthIn(max = SheetMaxWidth).fillMaxWidth(), verticalArrangement = Arrangement.spacedBy(16.dp)) {
        SectionLabel(section.label, count = section.count?.toString())
        section.content(this)
    }
}

@Composable
private fun Wrap(content: @Composable () -> Unit) {
    FlowRow(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalArrangement = Arrangement.spacedBy(8.dp), itemVerticalAlignment = Alignment.CenterVertically) { content() }
}

private fun SectionsBuilder.primitives(toasts: DsToasts) {
    section("Actions", 5) {
        Wrap {
            DsButton("Deploy", {})
            DsButton("Publish", {}, variant = DsButtonVariant.Inverse, iconStart = Lucide.Send)
            DsButton("Export", {}, variant = DsButtonVariant.Secondary, iconStart = Lucide.Download)
            DsButton("Cancel", {}, variant = DsButtonVariant.Ghost)
            DsButton("Stop all", {}, variant = DsButtonVariant.GhostDanger)
            DsButton("Delete", {}, variant = DsButtonVariant.Danger, iconStart = Lucide.Trash)
            DsButton("Deploying · 00:14", {}, loading = true)
            DsButton("Disabled", {}, enabled = false)
        }
        Wrap {
            DsIconButton(Lucide.Pencil, "Rename", {})
            DsIconButton(Lucide.X, "Close", {}, variant = DsIconButtonVariant.Ghost)
            DsIconButton(Lucide.Send, "Send", {}, variant = DsIconButtonVariant.Send)
            Kbd("⌘K")
            DsLink("Upgrade to Pro", {})
        }
        KeyHint(listOf(KeyHintItem("↵", "send"), KeyHintItem("⇧↵", "new line"), KeyHintItem("@", "address crew")))
    }
    section("Crew", Crew.entries.size) {
        Wrap { Crew.entries.forEach { Mascot(it, 32.dp, label = it.displayName) } }
        AgentByline(Crew.Koda, "Koda", meta = "15:37 · 4m 12s")
        Wrap {
            DsAvatar("Trian Damai")
            DsAvatar("Ana")
        }
        CrewRow(Crew.Dana, "Dana", "Money · budgets and bills", state = "idle", onClick = {})
        CrewRow(Crew.Koda, "Koda", "Builds web apps", state = "working")
    }
    section("Status") {
        Wrap {
            DsStatus(StatusKind.Live, "Live")
            DsStatus(StatusKind.Ready, "Ready")
            DsStatus(StatusKind.Idle, "Idle")
            DsStatus(StatusKind.Failed, "Failed")
            DsStatus(StatusKind.Attention, "Needs you")
            DsStatus(StatusKind.Working, "Koda is building", agent = Crew.Koda)
        }
        Wrap {
            DsTag(TagKind.Go, "Go")
            DsTag(TagKind.NoGo, "No-go")
            DsTag(TagKind.Neutral, "Standby")
        }
        var used by remember { mutableFloatStateOf(0.22f) }
        Meter("Free", used, caption = "21.8K / 1M tokens this month")
        Wrap { DsButton("Use more", { used = (used + 0.18f).coerceAtMost(1f) }, variant = DsButtonVariant.Secondary) }
    }
    section("Inputs") {
        var name by remember { mutableStateOf("warung-kopi-menu") }
        var search by remember { mutableStateOf("") }
        var view by remember { mutableStateOf("all") }
        var tab by remember { mutableStateOf("chat") }
        var checked by remember { mutableStateOf(true) }
        var toggled by remember { mutableStateOf(false) }
        DsTextField("Project name", name, { name = it }, helper = "Used in the address: $name.nomi.app", mono = true)
        DsTextField("Email", "ana@", {}, error = "Add the part after @, like ana@example.com")
        DsSearch(search, { search = it }, "Search chats", shortcut = "⌘K")
        DsSegmented("View", listOf(SegmentOption("all", "All", "12"), SegmentOption("live", "Live", "3"), SegmentOption("drafts", "Drafts")), view, { view = it })
        DsTabs("Workspace", listOf(TabItem("chat", "Chat"), TabItem("preview", "Preview"), TabItem("code", "Code")), tab, { tab = it }) {
            Text("The $it tab", style = Ds.ui, color = Ds.text2, modifier = Modifier.padding(vertical = 12.dp))
        }
        Wrap {
            DsCheckbox("Remember this device", checked, { checked = it })
            DsToggle("Notifications", toggled, { toggled = it })
            CheckMark(true, {})
        }
        Wrap {
            DsChip(ChipKind.Agent, "Koda", agent = Crew.Koda, onClick = {})
            DsChip(ChipKind.Model, "deepseek-flash", onClick = {})
            DsChip(ChipKind.Option, "in 7 days", icon = Lucide.Hourglass, onClick = {})
            DsChip(ChipKind.Provision, "postgres", icon = Lucide.Database)
        }
    }
    section("Overlays") {
        var dialog by remember { mutableStateOf(false) }
        Wrap {
            DsButton("Open dialog", { dialog = true }, variant = DsButtonVariant.Secondary)
            DsMenu(
                listOf(
                    MenuEntry.Item("Rename", Lucide.Pencil, shortcut = "R"),
                    MenuEntry.Item("Pin", Lucide.Pin),
                    MenuEntry.Item("Share", Lucide.Share),
                    MenuEntry.Divider,
                    MenuEntry.Item("Delete", Lucide.Trash, danger = true),
                ),
            ) { open, _ -> DsButton("Open menu", open, variant = DsButtonVariant.Secondary, iconEnd = Lucide.ChevronDown) }
            DsButton("Show toast", { toasts.show(ToastKind.Success, "Deployed", "warung-kopi-menu is live", "View", {}) }, variant = DsButtonVariant.Secondary)
            DsTooltip("New chat", shortcut = "⌘K") { DsIconButton(Lucide.Plus, "New chat", {}) }
        }
        DsDialog(dialog, { dialog = false }, "Delete project?", description = "warung-kopi-menu and its deploys go away. Its code stays in your export.", actions = {
            DsButton("Cancel", { dialog = false }, variant = DsButtonVariant.Secondary)
            DsButton("Delete", { dialog = false }, variant = DsButtonVariant.Danger, iconStart = Lucide.Trash)
        })
        DsDialog(true, {}, "Delete project?", description = "warung-kopi-menu and its deploys go away. Its code stays in your export.", inline = true, actions = {
            DsButton("Cancel", {}, variant = DsButtonVariant.Secondary)
            DsButton("Delete", {}, variant = DsButtonVariant.Danger, iconStart = Lucide.Trash)
        })
        DsMenu(listOf(MenuEntry.Item("Rename", Lucide.Pencil, shortcut = "R"), MenuEntry.Item("Pin", Lucide.Pin), MenuEntry.Divider, MenuEntry.Item("Delete", Lucide.Trash, danger = true)), inline = true)
        DsToast(ToastKind.Success, "Deployed", detail = "warung-kopi-menu is live", actionLabel = "View", onAction = {}, onDismiss = {})
        DsToast(ToastKind.Error, "Couldn't deploy", detail = "Vercel returned 402 · payment required")
        TooltipBubble("New chat", shortcut = "⌘K")
    }
    section("App chrome") {
        var recents by remember { mutableStateOf(true) }
        var query by remember { mutableStateOf("") }
        Box(Modifier.background(Ds.surface, MaterialTheme.shapes.extraLarge)) {
            DsSidebar(
                onNewChat = {},
                onCollapse = {},
                recents = {
                    RecentsList(
                        listOf(
                            RecentItem(RecentKind.Chat, "Kopi menu prices", "2m", active = true),
                            RecentItem(RecentKind.Project, "warung-kopi-menu", "1h"),
                            RecentItem(RecentKind.Chat, "November budget", "3d"),
                        ),
                        recents, { recents = !recents }, {},
                    )
                },
                footer = {
                    UsageCard("Free", 0.22f, "21.8K / 1M tokens this month", {})
                    UserRow("Trian Damai", {})
                },
            ) {
                NavItem(Lucide.Home, "Home", {}, active = true)
                NavItem(Lucide.Message, "Chats", {}, count = "4")
                NavGroupLabel("Build") { WorkspaceChip("Trian's space", {}) }
                NavItem(Lucide.Folder, "Projects", {})
                NavGroupLabel("Life")
                NavItem(Lucide.Wallet, "Money", {})
                NavItem(Lucide.Bell, "Reminders", {})
            }
        }
        Box(Modifier.background(Ds.surface, MaterialTheme.shapes.extraLarge)) {
            SettingsNav(
                groups = listOf(
                    SettingsNavGroup("Account", listOf(SettingsNavItem(Lucide.User, "Profile"), SettingsNavItem(Lucide.Sliders, "Preferences"), SettingsNavItem(Lucide.CreditCard, "Plan & billing"))),
                    SettingsNavGroup("Workspace", listOf(SettingsNavItem(Lucide.Users, "Members", "3"), SettingsNavItem(Lucide.Plug, "Connections"), SettingsNavItem(Lucide.Shield, "Security"))),
                ),
                active = "Profile", userName = "Trian Damai", query = query, onQuery = { query = it }, onBack = {}, userDetail = "trian@nomi.build", version = "v2.4.0", onLogout = {},
            )
        }
    }
    section("Lists") {
        ProjectRow("warung-kopi-menu", Crew.Koda, ProjectStatus(StatusKind.Live, "Live"), "08:41", {}, description = "Menu and prices for the coffee stall", url = "warung-kopi-menu.nomi.app")
        ProjectRow("bakery-orders", Crew.Dana, ProjectStatus(StatusKind.Failed, "Failed"), "yesterday", {}, description = "Pre-orders for Saturday")
        SkeletonRow("loading 4 projects…")
        SuggestionRow(Crew.Dana, "How much did I spend on coffee this month?", {}, route = "→ dana")
        SuggestionRow(Crew.Kala, "Remind me to pay rent on the 25th", {}, route = "→ kala")
        EmptyState(Lucide.Folder, "No projects yet", "Describe an app in the chat and Koda builds it here.", count = 0, actionLabel = "Start a project", onAction = {})
    }
    conversation()
}

private fun SectionsBuilder.conversation() {
    section("Conversation") {
        MessageUser("Make the menu page show prices in rupiah, and add a note that oat milk is +5k.", time = "14:02", onEdit = {})
        MessageUser("Is the stall open on Sunday?", status = SendStatus.Sending)
        MessageUser("Add the new pastry", status = SendStatus.Queued, onCancel = {})
        MessageUser("Publish it", status = SendStatus.Failed, onRetry = {}, onEdit = {})
        var version by remember { mutableIntStateOf(2) }
        MessageUser("Show prices in rupiah", edited = true, time = "14:05", version = MessageVersion(version, 2, { version-- }, { version++ }), files = listOf(AttachedFile("menu.pdf", "PDF · 1.2 MB")))
        MessageAgent(Crew.Koda, "Koda", meta = "14:02 · deepseek-flash", text = "Done. Prices now show as Rp 18.000, and oat milk carries a +5k note on every drink that takes milk.") {
            ActivityBlock("Wrote 3 files · +424", listOf(ActivityFile("src/routes/menu/+page.svelte", "+214"), ActivityFile("src/lib/price.ts", "+38"), ActivityFile("src/lib/menu.json", "+172")), total = "+424")
        }
        LiveStep("Writing src/routes/menu/+page.svelte", elapsed = "00:14", stopLabel = "Stop", onStop = {})
        ActivityBlock("Writing files", listOf(ActivityFile("src/app.css")), state = ActivityState.Working)
        ErrorCallout(
            "Deploy failed", code = "402 · payment_required", requestId = "req_8f2a",
            details = listOf(ErrorDetail("provider", "Vercel"), ErrorDetail("reason", "Your Vercel team is over its free build minutes.", strong = true)),
            actions = listOf(ErrorAction("Open Vercel billing", Lucide.ArrowUpRight) {}, ErrorAction("Retry", Lucide.Refresh) {}),
        )
        EventRow(EventKind.Deploy, "deploy #14 · main@a3f9c21 · built in 41s", "15:38")
        EventRow(EventKind.Domain, "warung-kopi.id verified", "15:40")
        KVRow("Code", "23 files · main@a3f9c21", icon = Lucide.Code)
        KVRow("Database", "postgres · 4 tables", icon = Lucide.Database)
        FileList(listOf(BuildFile("src/app.css", FileState.Done, "+48"), BuildFile("src/routes/+page.svelte", FileState.Writing), BuildFile("src/lib/menu.json", FileState.Queued)), total = 3)
    }
}

private val Story = SharePreview(Crew.Dana, "Dana", "14:06", "You spent Rp 1.284.000 on coffee in November, up 18% from October.")

private fun SectionsBuilder.chatResponses() {
    section("Thinking") {
        var open by remember { mutableStateOf(false) }
        var steps by remember { mutableStateOf(true) }
        Reasoning("Thinking", "The menu has 14 drinks.\n\nOat milk applies to the 9 that take milk.", false, {}, live = true, elapsed = "4s")
        Reasoning("Thought for 12s", "The menu has 14 drinks.\n\nOat milk applies to the 9 that take milk, so the note goes on those only.", open, { open = !open })
        Thought("Prices are whole thousands, so I dropped the decimals.")
        ThoughtSteps(
            "4 steps", listOf(
                ThoughtStep("Read the menu", StepState.Done, "menu.json · 14 items"),
                ThoughtStep("Price in rupiah", StepState.Done),
                ThoughtStep("Add the oat-milk note", StepState.Running),
                ThoughtStep("Deploy", StepState.Waiting),
            ),
            steps, { steps = !steps }, meta = "00:21",
        )
        PlanCard(
            "Saturday pre-orders", listOf(
                PlanStep("Collect orders from the form", StepState.Done, StepOwner(Crew.Koda, "Koda"), "12 orders"),
                PlanStep("Total the ingredients", StepState.Running, StepOwner(Crew.Dana, "Dana")),
                PlanStep("Remind me Friday 18:00", StepState.Waiting, StepOwner(Crew.Kala, "Kala")),
            ),
            onStop = {},
        )
        var draft by remember { mutableStateOf(listOf(DraftStep("Collect orders", StepOwner(Crew.Koda, "Koda")), DraftStep("Total ingredients", StepOwner(Crew.Dana, "Dana")), DraftStep("Remind me Friday", StepOwner(Crew.Kala, "Kala")))) }
        DraftPlanCard(
            "Here's how I'd run Saturday. Reorder or reassign anything first.", draft, "About 3 min · 3 crew",
            onMove = { from, to -> if (to in draft.indices) draft = draft.toMutableList().apply { add(to, removeAt(from)) } },
            onRemove = { index -> draft = draft.filterIndexed { at, _ -> at != index } },
            onAdd = { draft = draft + DraftStep("New step", StepOwner(Crew.Nomi, "Nomi")) },
        )
    }
    section("Rich") {
        Wrap {
            Text("Oat milk is the most ordered add-on", style = Ds.body, color = Ds.text2)
            Citation(1, "orders.csv")
            Hyperlink("see the orders", {})
            MentionCrew(Crew.Dana, "Dana")
            MentionFile("menu.json")
        }
        Sources(listOf(Source("orders.csv", SourceKind.File, "142 rows"), Source("Coffee prices in Jakarta, 2026", SourceKind.Web, "kompas.com", onOpen = {})))
        CodeBlock(
            "export const rupiah = (n: number) =>\n  `Rp ${'$'}{n.toLocaleString('id-ID')}`;", language = "ts", file = "src/lib/price.ts",
            tokens = listOf(
                listOf(CodeToken("export", CodeTokenKind.Keyword), CodeToken(" "), CodeToken("const", CodeTokenKind.Keyword), CodeToken(" "), CodeToken("rupiah", CodeTokenKind.Fn), CodeToken(" = (n: "), CodeToken("number", CodeTokenKind.Type), CodeToken(") =>")),
                listOf(CodeToken("  "), CodeToken("`Rp \${n.toLocaleString('id-ID')}`", CodeTokenKind.String), CodeToken(";", CodeTokenKind.Punct)),
            ),
        )
        Diff(
            "src/routes/menu/+page.svelte",
            listOf(DiffHunk("@@ -12,4 +12,5 @@", listOf(
                DiffLine(DiffLineKind.Context, "<li>{item.name}</li>", 12, 12),
                DiffLine(DiffLineKind.Del, "<span>{item.price}</span>", 13, null),
                DiffLine(DiffLineKind.Add, "<span>{rupiah(item.price)}</span>", null, 13),
                DiffLine(DiffLineKind.Add, "{#if item.milk}<small>oat milk +5k</small>{/if}", null, 14),
            ))),
            request = DiffRequest(Crew.Koda, "Koda", 2), onApply = {}, onReject = {},
        )
        DataTable(
            "Coffee, by month", listOf("Month", "Spent", "Change"),
            listOf(listOf(Cell("September"), Cell("Rp 982.000"), Cell("—")), listOf(Cell("October"), Cell("Rp 1.088.000"), Cell("+11%", CellTone.Bad)), listOf(Cell("November"), Cell("Rp 1.284.000"), Cell("+18%", CellTone.Bad))),
            total = listOf(Cell("Total"), Cell("Rp 3.354.000"), Cell("")), source = "Rp · from Dana · 142 transactions", linkLabel = "Open in Sheets", onLink = {}, onDownload = {},
        )
        LinkPreview("kompas.com", "Coffee prices in Jakarta, 2026", {}, description = "What a cup costs across the city, from kaki lima to cafés.")
        var active by remember { mutableIntStateOf(0) }
        MentionPicker(
            listOf(MentionOption.CrewMember(Crew.Koda, "Koda", "builds web apps"), MentionOption.CrewMember(Crew.Dana, "Dana", "money"), MentionOption.FileRef("menu.json", "warung-kopi-menu")),
            active, { active = it }, {},
        )
        Reminder("Pay the coffee supplier", "25 Nov · 09:00 · every month", owner = StepOwner(Crew.Kala, "Kala"), onEdit = {})
        var due by remember { mutableStateOf(true) }
        Reminder("Pay rent", "25 Nov · 09:00", due = due, onSnooze = { due = false }, onDone = { due = false })
        var eggs by remember { mutableIntStateOf(2) }
        Cart(
            "Saturday baking", "Segari",
            listOf(CartItem("Eggs", eggs, "Rp ${58 * eggs}.000", "1 tray · 10 butir"), CartItem("Butter", 1, "Rp 42.000", "250 g"), CartItem("Almond flour", 0, "—", available = false, swap = "Swap for wheat flour")),
            subtotal = "Rp 158.000", delivery = "Rp 12.000", total = "Rp 170.000", onCheckout = {}, note = "Prices from Segari at 08:13", onQuantity = { _, q -> eggs = q }, onSwap = {}, onSave = {},
        )
    }
    section("Actions") {
        var status by remember { mutableStateOf(ApprovalStatus.Pending) }
        ApprovalCard(
            Crew.Dana, "Send the November report to Bu Sri", status = status,
            fields = listOf(KeyValue("to", "sri@warungkopi.id"), KeyValue("subject", "November coffee report")),
            body = "Hi Bu Sri,\nNovember's report is attached. Coffee spend was up 18%.", attachment = "november-report.pdf",
            rememberLabel = "Don't ask again for Bu Sri", approveLabel = "Send", approveIcon = Lucide.Send,
            outcome = if (status == ApprovalStatus.Approved) "Approved by you · sent 14:06" else "Declined · nothing was sent",
            actionLabel = "Undo", onAction = { status = ApprovalStatus.Pending },
            onApprove = { status = ApprovalStatus.Approved }, onReject = { status = ApprovalStatus.Rejected }, onEdit = { status = ApprovalStatus.Editing },
            onSave = { status = ApprovalStatus.Approved }, onCancel = { status = ApprovalStatus.Pending },
        )
        val questions = listOf(
            ClarifyQuestion("Getting there", "How will people get their order?", listOf(ClarifyOption("Pick up at the stall", recommended = true), ClarifyOption("Delivery", "GoSend or Grab")), other = true),
            ClarifyQuestion("Paying", "How will they pay?", listOf(ClarifyOption("QRIS"), ClarifyOption("Cash"), ClarifyOption("Transfer")), multiple = true),
        )
        var reviewed by remember { mutableStateOf(false) }
        if (!reviewed) {
            ClarifyCard(Crew.Koda, "Two questions before I build", questions, { reviewed = true }, { reviewed = true })
        } else {
            ClarifyReview(listOf(ReviewAnswer("Getting there", "Pick up at the stall", isDefault = true), ReviewAnswer("Paying", "QRIS, Cash")), {}, { reviewed = false }, {}, assumption = "Pick-up was the default because the stall has no delivery partner yet.")
        }
        ClarifyAnswered(2, "Pick up · QRIS, Cash", {})
        DecisionCard("Where should the menu live?", listOf(DecisionOption("warung-kopi-menu.nomi.app", "Free, ready now", suggested = true), DecisionOption("warung-kopi.id", "Your domain · DNS needed")), {})
        PlanApproval(listOf(RunOption("Build and publish", "Live as soon as it's done", recommended = true), RunOption("Build, ask before publishing")), { _, _ -> }, who = "Koda")
        FormCard(Crew.Kala, "New reminder", "Save", {}, meta = "prefilled from chat", note = "Kala pings you in Inbox and on your phone", onCancel = {}) {
            DsTextField("What", "Pay the coffee supplier", {})
            DsTextField("When", "25 Nov · 09:00 · every month", {}, mono = true)
        }
        QuickReplies(listOf("Yes, publish", "Not yet", "Show me first"), {})
        FollowUps(listOf("Add a pastry section", "Make it work offline", "Share it with Bu Sri"), {})
        SelectionCard(
            Crew.Dana, "Which subscriptions should I track?",
            listOf(SelectionItem("Spotify", "monthly · BCA", "Rp 54.990"), SelectionItem("Netflix", "monthly · BCA", "Rp 186.000"), SelectionItem("iCloud", "monthly · Visa", "Rp 45.000")),
            "Track", {}, initial = listOf(true, true, false), total = "Rp 240.990 / month", onSkip = {},
        )
    }
    section("Feedback") {
        var feedback by remember { mutableStateOf<ReplyFeedback?>(null) }
        ResponseActions(
            "Done. Prices now show in rupiah.", meta = "deepseek-flash · 1.4s · 812 tokens", feedback = feedback,
            onLike = { feedback = if (feedback == ReplyFeedback.Liked) null else ReplyFeedback.Liked },
            onDislike = { feedback = if (feedback == ReplyFeedback.Disliked) null else ReplyFeedback.Disliked },
            onShare = {}, onRegenerate = {}, onDelete = {},
        )
        DislikeFeedback({ _, _, _ -> }, {})
        FeedbackSent()
        FeedbackSent(withChat = true, retryLabel = "Retry", onRetry = {})
        DeletedReply("Koda", onUndo = {}, onRegenerate = {})
        DeleteReplyDialog(true, "Koda", {}, {}, inline = true)
        ShareDialog(true, {}, Story, 24, "https://nomi.build/s/k3f9a", "in 7 days", inline = true)
    }
    section("States") {
        var text by remember { mutableStateOf(ReplyStatus.Streaming) }
        ReplyText("You spent Rp 1.284.000 on coffee in November", status = text)
        Wrap {
            ReplyStatus.entries.forEach { s -> DsButton(s.name, { text = s }, variant = if (s == text) DsButtonVariant.Inverse else DsButtonVariant.Secondary) }
        }
        ReplyText("You spent Rp 1.284.000 on", status = ReplyStatus.Stopped, elapsed = "00:03", onContinue = {})
        ReplyText("You spent Rp 1.284.000 on", status = ReplyStatus.Interrupted, reason = "Reply cut off after 18s. DeepSeek closed the stream.", onContinue = {}, onRestart = {})
        BannerKind.entries.forEach { kind ->
            Banner(
                kind,
                when (kind) {
                    BannerKind.Offline -> "You're offline. Messages wait in the outbox."
                    BannerKind.Online -> "Back online. 2 messages sent."
                    BannerKind.Slow -> "DeepSeek is slow right now. Replies may take longer."
                    BannerKind.Expiring -> "Your OpenAI key expires in 3 days."
                    BannerKind.Quota -> "80% of this month's tokens used."
                },
                actionLabel = if (kind == BannerKind.Expiring) "Update key" else null, onAction = {},
            )
        }
    }
    section("Controls") {
        Wrap {
            AttachMenu("20 MB", {}, driveBy = "Tara", inline = true)
            RegenerateMenu({}, models = listOf(RegenerateModel("claude-sonnet", "Anthropic"), RegenerateModel("gpt-5-mini", "OpenAI")), inline = true)
        }
        var progress by remember { mutableFloatStateOf(0.62f) }
        Wrap {
            AttachmentChip("menu-2026.pdf", "${(progress * 100).toInt()}% · 3,1 of 5 MB", progress = progress, onRemove = {})
            AttachmentChip("menu.pdf", "PDF · 4,2 MB", onRemove = {})
            DsButton("Upload more", { progress = (progress + 0.2f).coerceAtMost(1f) }, variant = DsButtonVariant.Ghost)
        }
        ComposerLimit("12,480", "10,000", onFile = {})
        Wrap {
            CopiedNote()
            StopHint({})
            var at by remember { mutableIntStateOf(1) }
            VersionSwitcher(at, 2, { at-- }, { at++ })
        }
        MessageEditor("Show prices in rupiah", {}, {}, version = "1 / 2")
        VoiceRecorder("0:14", listOf(0.2f, 0.5f, 0.8f, 0.4f, 0.9f, 0.3f, 0.6f, 0.7f, 0.2f, 0.5f, 0.4f, 0.3f), "Bahasa Indonesia", {}, {}, fading = 3)
        VoiceTranscribed("0:14")
    }
    section("Threads") {
        DateDivider("Today")
        NewDivider(3)
        LoadingOlder()
        Wrap { JumpToLatest(3, {}) }
        RenameField("Kopi menu prices", {}, {})
        ChatMenu({}, inline = true)
        MoveDialog(true, listOf(MoveProject("1", "warung-kopi-menu", "Trian's space"), MoveProject("2", "bakery-orders", "Family", shared = "Family")), {}, {}, inline = true, initialChosen = "2")
        SkeletonRow("loading 24 chats…")
    }
    section("Crew") {
        CrewWorking(listOf(Crew.Koda, Crew.Dana, Crew.Kala), onStop = {})
        BackgroundNote(eta = "~3 min")
        Handoff(StepOwner(Crew.Nomi, "Nomi"), StepOwner(Crew.Dana, "Dana"), reason = "money question", onBack = {})
        MemorySaved("you take oat milk", onEdit = {}, onUndo = {})
        MemoryUsed("the stall closes on Mondays", onManage = {})
        Boundary("I can't move money", "Transfers stay with you and your bank. I can get everything ready to send.", alternatives = listOf(Alternative(Crew.Dana, "Draft the transfer", "amount and account filled in"), Alternative(Crew.Nomi, "Open BCA mobile", "bca.co.id", external = true)))
        ConnectPrompt(Crew.Tara, Lucide.Calendar, "Google Calendar", "Tara needs it to check your weekends", "Read your events and free/busy times", {}, alternative = "Or tell me your free weekends", onLater = {})
        SessionExpired({}, draft = "Can you add the Saturday specials?")
        SupervisorStopped(
            listOf(StoppedTask(Crew.Koda, "Publish the menu", TaskState.Stopped, "at deploy"), StoppedTask(Crew.Dana, "Total November", TaskState.Done), StoppedTask(Crew.Kala, "Remind Bu Sri", TaskState.Waiting)),
            meta = "14:09 · loop detected", onDiscard = {}, onResume = {},
        )
    }
    section("Mobile") {
        var text by remember { mutableStateOf("") }
        var option by remember { mutableIntStateOf(0) }
        var view by remember { mutableStateOf(WorkspaceView.Chat) }
        NavBar("Kopi menu prices", {}, who = Crew.Koda to "Koda", onMenu = {})
        ChatRow(Crew.Dana, "November budget", {}, time = "14:06", preview = "You spent Rp 1.284.000 on coffee…", unread = true)
        ChatRow(Crew.Koda, "warung-kopi-menu", {}, time = "08:41", preview = "It's live at warung-kopi-menu.nomi.app")
        Byline(Crew.Koda, "Koda", meta = "14:02")
        UserMessage("Make the menu show prices in rupiah")
        UserMessage("Publish it", failed = true, onRetry = {})
        ActionBar("Done.", feedback = ReplyFeedback.Liked, onLike = {}, onDislike = {}, onShare = {}, onMore = {})
        MobileApprovalCard(Crew.Dana, "Send the November report", "Send", "Decline", {}, {}, to = "sri@warungkopi.id", subject = "November coffee report", excerpt = "Hi Bu Sri, November's report is attached.", attachment = "november-report.pdf", approveIcon = Lucide.Send, onOpen = {})
        ErrorCard("Couldn't publish", "Vercel", "Team is over its free build minutes.", code = "402", meta = "req_8f2a · 14:08", primaryLabel = "Open Vercel billing", onPrimary = {}, secondaryLabel = "Try again", onSecondary = {})
        MobileProgress(1, 3)
        PhoneOption("Pick up at the stall", option == 0, { option = 0 }, key = 1, detail = "Ready in 10 minutes", recommended = true)
        PhoneOption("Delivery", option == 1, { option = 1 }, key = 2, detail = "GoSend or Grab")
        PhoneToast(PhoneToastKind.Success, "Published", detail = "warung-kopi-menu is live", actionLabel = "Open", onAction = {})
        StatList("Coffee", listOf(StatRow("October", "Rp 1.088.000", Cell("+11%", CellTone.Bad)), StatRow("November", "Rp 1.284.000", Cell("+18%", CellTone.Bad), "Most on oat lattes")), sub = "Rp", total = StatRow("Total", "Rp 2.372.000"))
        Column(Modifier.background(Ds.surface, MaterialTheme.shapes.extraLarge)) {
            SheetHeader("Chat options", {}, subtitle = "Kopi menu prices")
            SheetRow(Lucide.Pencil, "Rename", {})
            SheetRow(Lucide.FolderInput, "Move to a project", {}, more = true)
            SheetRow(Lucide.Download, "Export", {}, meta = "md · pdf")
            SheetRow(Lucide.Trash, "Delete chat", {}, danger = true)
        }
        WorkspaceSwitch(view, { view = it }, previewCount = 2)
        MobileComposer(text, { text = it }, { text = "" }, onAttach = {}, onRecord = {})
        MobileComposer("", {}, {}, streaming = true, onStop = {}, notice = "transcribed from 0:14 of audio")
    }
}

private fun SectionsBuilder.buildSheet() {
    section("Estimate") {
        BuildPlan(
            "warung-kopi-menu", "A menu site for the coffee stall",
            listOf("A menu page with prices in rupiah", "An oat-milk note on drinks that take milk", "Publish to warung-kopi-menu.nomi.app"),
            "You get a live link and the code.",
            listOf(RunOption("Build and publish", "Live as soon as it's done", recommended = true), RunOption("Build, ask before publishing")),
            "About 5 min · included in your free plan", { _, _ -> },
        )
        BuildRunning("08:13", "Writing src/routes/menu/+page.svelte", 3, 5, "Menu page", elapsed = "00:14", eta = "~2 min left", onStop = {})
        BuildDone("built in 4m 12s · 08:17", "https://warung-kopi-menu.nomi.app", "3 pages · menu, about, order", {}, onShare = {}) {
            KVRow("Code", "23 files · main@a3f9c21")
            KVRow("Deploy", "#1 · 41s")
        }
    }
    section("Building") {
        PreviewBuilding(listOf(BuildingStep("Set up the project", StepState.Done), BuildingStep("Write the menu page", StepState.Running), BuildingStep("Deploy", StepState.Waiting)))
        FileList(listOf(BuildFile("src/app.css", FileState.Done, "+48"), BuildFile("src/routes/menu/+page.svelte", FileState.Writing), BuildFile("src/lib/menu.json", FileState.Queued), BuildFile("src/lib/price.ts", FileState.Queued)), total = 4)
    }
    section("Share") {
        BuildSharePanel("https://warung-kopi-menu.nomi.app", SharePreviewCard("Warung Kopi · Menu", "Kopi susu, pastries and today's prices."), {}, {}, onEditPreview = {}, onInvite = {}, onMore = {})
        ExportPanel("warung-kopi-menu", "9 files · git history · main@4be1c07", {}, {}, {}, database = "postgres · 4 tables · 2.1 MB", env = "3 names · DATABASE_URL, …")
    }
    section("First run") { FirstRun({}) }
    section("Workspace") {
        var state by remember { mutableStateOf(PublishState.Ready) }
        Wrap {
            PublishState.entries.forEach { s -> PublishButton(s, count = 3, elapsed = "00:08", onPublish = { state = PublishState.Publishing }, onRetry = {}) }
        }
        Wrap { PublishButton(state, count = 3, elapsed = "00:08", onPublish = { state = PublishState.Publishing }, onRetry = { state = PublishState.Ready }) }
        var width by remember { mutableFloatStateOf(420f) }
        Row(Modifier.fillMaxWidth().height(120.dp)) {
            Box(Modifier.weight(1f).background(Ds.surface))
            ResizeHandle(width, 1200f, { width = it })
            Box(Modifier.weight(1f).background(Ds.surface2))
        }
        WorkspaceSheet(Lucide.Link, "Domain", {}, meta = "warung-kopi-menu", onExpand = {}) {
            DnsRecord("CNAME", "www", "cname.nomi.app", DnsState.Waiting)
            Box(Modifier.height(8.dp))
            DnsRecord("A", "@", "76.76.21.21", DnsState.Verified)
        }
    }
    section("Code") {
        DeviceBar("iPhone 15", {}, onPhone = {})
        CodeView(
            listOf(TreeEntry("src", 0, EntryKind.Folder), TreeEntry("routes", 1, EntryKind.Folder), TreeEntry("+page.svelte", 2, EntryKind.File, active = true), TreeEntry("menu.json", 1, EntryKind.Text), TreeEntry("schema.sql", 0, EntryKind.Database)),
            "src/routes/+page.svelte",
            listOf(
                listOf(CodeToken("<script", CodeTokenKind.Keyword), CodeToken(">")),
                listOf(CodeToken("  "), CodeToken("import", CodeTokenKind.Keyword), CodeToken(" menu "), CodeToken("from", CodeTokenKind.Keyword), CodeToken(" "), CodeToken("'\$lib/menu.json'", CodeTokenKind.String), CodeToken(";")),
                listOf(CodeToken("</script", CodeTokenKind.Keyword), CodeToken(">")),
                listOf(CodeToken("<h1>Warung Kopi</h1>")),
            ),
            {}, { _, _ -> }, branch = "main@9d2e41a", meta = "read-only · edited by Koda 08:41", selection = 2 to 2,
        )
        CodeQuote("+page.svelte", "2", "import menu from '\$lib/menu.json';", {})
        Terminal(
            "run #2 · publish deploy #2",
            listOf(
                TermLine(TermLineKind.Command, "$ npm run build", "08:15"), TermLine(TermLineKind.Out, "vite v6 building for production…"),
                TermLine(TermLineKind.Warn, "(!) menu.json is 412 kB"), TermLine(TermLineKind.Ok, "✓ built in 3.1s"),
                TermLine(TermLineKind.Command, "$ nomi deploy", "08:16"), TermLine(TermLineKind.Fail, "✗ 402 payment_required"),
            ),
            failureTitle = "Deploy failed", failureDetail = "Vercel team is over its free build minutes.", onFix = {},
        )
        PhoneFrame(width = 260.dp) {
            Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                Text("Warung Kopi", style = MaterialTheme.typography.titleLarge, color = androidx.compose.ui.graphics.Color.Black)
                Text("Kopi susu · Rp 18.000", style = Ds.ui, color = androidx.compose.ui.graphics.Color.DarkGray)
                Text("Oat milk +5k", style = Ds.caption, color = androidx.compose.ui.graphics.Color.Gray)
            }
        }
    }
    section("Together") {
        val ana = Person("Ana Putri", "AP", 1)
        val budi = Person("Budi Santoso", "BS", 2)
        BuildInvite("Trian Damai", "warung-kopi-menu", InviteRole.Prompt, "ana@example.com", "in 7 days", {}, detail = "3 pages · live")
        Wrap {
            Presence(listOf(Presenter("Trian", "TD", online = true), Presenter("Ana", "AP", 1, true), Presenter("Budi", "BS", 2)))
            PeerCursor("Ana", 1)
            PeerCursor("Budi", 2)
        }
        PresenceMenu(listOf(PresentPerson("Trian Damai", "TD", "Chat", you = true), PresentPerson("Ana Putri", "AP", "Preview · /menu", 1), PresentPerson("Budi Santoso", "BS", "Code · price.ts", 2)), {})
        FollowBar("Ana", 1, "Preview · /menu", {})
        var role by remember { mutableStateOf(InviteRole.Prompt) }
        RoleMenu(role, { role = it })
        JoinedNote("Ana", InviteRole.Prompt, publisher = "Trian")
        TeammateMessage(ana, "Can the oat-milk note be green?", time = "08:44", to = "Koda")
        Typing(budi)
        RequestQueue(listOf(Ask("Make the note green", ana, AskState.Running), Ask("Add a pastry section", budi, AskState.Next, "queued 08:45"), Ask("Bigger prices", ana, AskState.Queued)), onFirst = {})
        PublishRequest(ana, {}, meta = "1 change · 08:46", onReview = {})
    }
    section("Mobile build") {
        AskBar({}, {})
        BuildPlanCard("warung-kopi-menu", "A menu site for the coffee stall", listOf("Menu page in rupiah", "Oat-milk note", "Publish"), "You get a live link and the code.", "About 5 min · free", {}, {}, {})
        BuildProgress("warung-kopi-menu", "00:42", "Writing src/routes/menu/+page.svelte", 3, 5, "Menu page", "~2 min left", {})
        LiveCard("built in 4m 12s", "https://warung-kopi-menu.nomi.app", "3 pages · menu, about, order", {}, {})
        ChangeReady(Person("Ana Putri", "AP", 1), "1 change · 08:46", {}, {})
        AskQueue(listOf(Ask("Make the note green", Person("Ana Putri", "AP", 1), AskState.Running), Ask("Add a pastry section", Person("Budi Santoso", "BS", 2), AskState.Next)), onFirst = {})
        TeammateBubble(Person("Ana Putri", "AP", 1), "Can the oat-milk note be green?", time = "08:44", to = "Koda")
        ProjectNav("warung-kopi-menu", "Live · warung-kopi-menu.nomi.app", PublishReady.Ready, {}, {}, count = 2, onPublish = {})
        ProjectNav("warung-kopi-menu", "Live", PublishReady.Idle, {}, {})
    }
}
