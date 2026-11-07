package ai.nomi.designsystem.ds.builds

import ai.nomi.designsystem.ds.CopyGlyph
import ai.nomi.designsystem.ds.Crew
import ai.nomi.designsystem.ds.Ds
import ai.nomi.designsystem.ds.footerNote
import ai.nomi.designsystem.ds.DsBareIcon
import ai.nomi.designsystem.ds.DsButton
import ai.nomi.designsystem.ds.DsButtonVariant
import ai.nomi.designsystem.ds.DsCard
import ai.nomi.designsystem.ds.DsFold
import ai.nomi.designsystem.ds.DsIcon
import ai.nomi.designsystem.ds.DsIconButton
import ai.nomi.designsystem.ds.DsIconButtonVariant
import ai.nomi.designsystem.ds.DsMotion
import ai.nomi.designsystem.ds.DsSegmented
import ai.nomi.designsystem.ds.DsShapes
import ai.nomi.designsystem.ds.DsSpinner
import ai.nomi.designsystem.ds.DsStatus
import ai.nomi.designsystem.ds.DsTag
import ai.nomi.designsystem.ds.DsText
import ai.nomi.designsystem.ds.DsTextAction
import ai.nomi.designsystem.ds.DsTextField
import ai.nomi.designsystem.ds.FileRow
import ai.nomi.designsystem.ds.FileState
import ai.nomi.designsystem.ds.Hairline
import ai.nomi.designsystem.ds.LiveStep
import ai.nomi.designsystem.ds.Mascot
import ai.nomi.designsystem.ds.MinTouch
import ai.nomi.designsystem.ds.SectionLabel
import ai.nomi.designsystem.ds.SegmentOption
import ai.nomi.designsystem.ds.SegmentedSurface
import ai.nomi.designsystem.ds.StatusKind
import ai.nomi.designsystem.ds.StepMark
import ai.nomi.designsystem.ds.StepState
import ai.nomi.designsystem.ds.TagKind
import ai.nomi.designsystem.ds.chat.CodeToken
import ai.nomi.designsystem.ds.chat.CodeTokenKind
import ai.nomi.designsystem.ds.chat.RunOption
import ai.nomi.designsystem.ds.chat.RunOptions
import ai.nomi.designsystem.ds.dsOutline
import ai.nomi.designsystem.ds.mobile.PeerAvatar
import ai.nomi.designsystem.ds.mobile.Person
import ai.nomi.designsystem.ds.mobile.PhoneStatusBar
import ai.nomi.designsystem.ds.mobile.StepSegments
import ai.nomi.designsystem.ds.rememberCopier
import ai.nomi.designsystem.icon.Lucide
import ai.nomi.designsystem.icon.NomiGlyph
import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.scaleIn
import androidx.compose.animation.scaleOut
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.Orientation
import androidx.compose.foundation.gestures.draggable
import androidx.compose.foundation.gestures.rememberDraggableState
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsDraggedAsState
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.selection.selectable
import androidx.compose.foundation.selection.selectableGroup
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.semantics.LiveRegionMode
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.liveRegion
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.dp

private const val QUEUED_ALPHA = 0.45f
private const val TYPING_MS = 600
private const val TYPING_DOTS = 3
private const val TYPING_STAGGER = 0.33f
private const val LOW_ALPHA = 0.3f
private const val PHONE_RATIO = 0.48f

/** Koda's byline at the top of a build card: name, what this is, and its status on the right. */
@Composable
private fun androidx.compose.foundation.layout.RowScope.KodaHeader(meta: String, trailing: (@Composable () -> Unit)? = null) {
    Mascot(Crew.Koda, 20.dp, working = false)
    Text("Koda", style = Ds.ui.copy(fontWeight = FontWeight.Medium), color = Ds.text)
    DsText(meta, Ds.label, Ds.text3, Modifier.weight(1f), singleLine = true)
    trailing?.invoke()
}

/** The plan before anything runs: what, the steps, the outcome, how to run it, and the cost range. */
@Composable
fun BuildPlan(project: String, summary: String, steps: List<String>, outcome: String, options: List<RunOption>, cost: String, onRun: (choice: Int?, keepPlanning: String?) -> Unit, modifier: Modifier = Modifier) {
    var value by remember { mutableIntStateOf(-1) }
    var changes by remember { mutableStateOf("") }
    val chosen = if (value >= 0) value else maxOf(0, options.indexOfFirst { it.recommended })
    val keep = chosen == options.size
    DsCard(
        modifier,
        header = { KodaHeader("plan") },
        footer = {
            Text(cost, style = Ds.label, color = Ds.text3, modifier = Modifier.footerNote(this))
            AnimatedContent(keep, label = "build-plan-run") { k ->
                DsButton(if (k) "Send changes" else "Build it", { if (k) onRun(null, changes.trim()) else onRun(chosen, null) }, iconEnd = if (k) Lucide.Send else Lucide.Play, enabled = !k || changes.isNotBlank())
            }
        },
    ) {
        Column(Modifier.padding(start = 16.dp, end = 16.dp, top = 14.dp, bottom = 4.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
            Text(project, style = Ds.label, color = Ds.text3)
            Text(summary, style = Ds.body.copy(fontWeight = FontWeight.SemiBold), color = Ds.text, modifier = Modifier.semantics { heading() })
        }
        Column(Modifier.padding(start = 16.dp, end = 16.dp, top = 6.dp, bottom = 14.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
            Text("Here’s what Koda will build", style = Ds.ui, color = Ds.text2)
            steps.forEachIndexed { index, step ->
                Row(horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                    Text("${index + 1}", style = Ds.mono, color = Ds.text3, modifier = Modifier.width(14.dp))
                    Text(step, style = Ds.ui, color = Ds.text)
                }
            }
        }
        Row(Modifier.padding(start = 16.dp, end = 16.dp, bottom = 14.dp).fillMaxWidth().clip(DsShapes.sm).background(Ds.accentBg).padding(10.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            DsIcon(Lucide.Sparkles, 14.dp, Ds.accent, Modifier.padding(top = 2.dp))
            Text(outcome, style = Ds.ui, color = Ds.text)
        }
        HorizontalDivider(thickness = Hairline, color = Ds.border)
        RunOptions(options, "Koda", value, { value = it }, changes, { changes = it }, Modifier.padding(14.dp))
    }
}

/** A build running: the real step it's on with the time so far and stop, its steps as segments, and the eta. */
@Composable
fun BuildRunning(started: String, step: String, current: Int, total: Int, label: String, modifier: Modifier = Modifier, elapsed: String? = null, eta: String? = null, onStop: (() -> Unit)? = null) {
    DsCard(
        modifier,
        header = { KodaHeader("running · started $started") { DsStatus(StatusKind.Working, "Koda is building", agent = Crew.Koda) } },
    ) {
        Box(Modifier.padding(horizontal = 16.dp, vertical = 12.dp)) { LiveStep(step, elapsed = elapsed, stopLabel = "Stop", onStop = onStop) }
        Column(Modifier.padding(start = 16.dp, end = 16.dp, top = 4.dp, bottom = 14.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
            StepSegments(current, total, nowColor = Ds.text2)
            Row(verticalAlignment = Alignment.CenterVertically) {
                Text("Step $current of $total · $label", style = Ds.caption, color = Ds.text2, modifier = Modifier.weight(1f).semantics { liveRegion = LiveRegionMode.Polite })
                if (eta != null) Text(eta, style = Ds.label, color = Ds.text3)
            }
        }
    }
}

/** A build that's live: its address (copy, open), what it did, details, and share. */
@Composable
fun BuildDone(meta: String, url: String, summary: String, onOpen: () -> Unit, modifier: Modifier = Modifier, onDetails: (() -> Unit)? = null, onShare: (() -> Unit)? = null, details: (@Composable ColumnScope.() -> Unit)? = null) {
    val copier = rememberCopier()
    var open by remember { mutableStateOf(false) }
    DsCard(modifier, header = { KodaHeader(meta) { DsStatus(StatusKind.Live, "Live", small = true) } }) {
        FlowRow(Modifier.padding(horizontal = 16.dp, vertical = 14.dp), horizontalArrangement = Arrangement.spacedBy(10.dp), itemVerticalAlignment = Alignment.CenterVertically) {
            DsIcon(Lucide.Globe, 14.dp, Ds.text2)
            Text(url.removePrefix("https://").removePrefix("http://"), style = Ds.mono.copy(fontWeight = FontWeight.Medium), color = Ds.text, modifier = Modifier.weight(1f))
            DsBareIcon(if (copier.copied) Lucide.Check else Lucide.Copy, if (copier.copied) "Copied" else "Copy link", { copier.copy(url) }, tint = Ds.text2)
            DsButton("Open", onOpen, variant = DsButtonVariant.Ghost, iconStart = Lucide.ArrowUpRight)
        }
        Row(Modifier.padding(start = 16.dp, end = 16.dp, bottom = 12.dp), verticalAlignment = Alignment.CenterVertically) {
            Text(summary, style = Ds.caption, color = Ds.text2, modifier = Modifier.weight(1f))
            if (onDetails != null || details != null) {
                Row(
                    Modifier.heightIn(min = MinTouch).clip(DsShapes.xs).clickable(role = Role.Button) { open = !open; onDetails?.invoke() },
                    horizontalArrangement = Arrangement.spacedBy(4.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Text("Details", style = Ds.caption, color = Ds.text2)
                    ai.nomi.designsystem.ds.DsChevron(open, 12.dp, Ds.text3)
                }
            }
        }
        if (details != null) DsFold(open) { Column(Modifier.padding(horizontal = 16.dp), content = details) }
        HorizontalDivider(thickness = Hairline, color = Ds.border)
        FlowRow(Modifier.padding(horizontal = 16.dp, vertical = 12.dp), horizontalArrangement = Arrangement.spacedBy(8.dp), itemVerticalAlignment = Alignment.CenterVertically) {
            if (onShare != null) DsButton("Share", onShare, variant = DsButtonVariant.Secondary, iconEnd = Lucide.Share)
            Text("Changes after this need Publish", style = Ds.caption, color = Ds.text3)
        }
    }
}

enum class InviteRole { Build, Prompt, View }

/** An invitation to build a project together: from whom, what you'll be able to do, the project, and join. */
@Composable
fun BuildInvite(from: String, project: String, role: InviteRole, sentTo: String, expires: String, onJoin: () -> Unit, modifier: Modifier = Modifier, detail: String? = null) {
    val first = from.split(" ").first()
    val body = when (role) {
        InviteRole.Prompt -> "You'll prompt Koda in this project's chat together with $first. You can ask for changes and see them in the preview; $first publishes."
        InviteRole.Build -> "You'll build with Koda in this project's chat together with $first, and you can publish changes."
        InviteRole.View -> "You can open the preview and the code of this project. Changes stay with $first."
    }
    Column(modifier.fillMaxWidth().widthIn(max = 480.dp).clip(DsShapes.md).background(Ds.surface).dsOutline(Ds.border, DsShapes.md).padding(24.dp), verticalArrangement = Arrangement.spacedBy(14.dp)) {
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
            Mascot(Crew.Nomi, 20.dp)
            Text("Nomi", style = Ds.ui.copy(fontWeight = FontWeight.SemiBold), color = Ds.text)
        }
        Text("$from invited you to build $project", style = Ds.heading.copy(fontWeight = FontWeight.SemiBold), color = Ds.text, modifier = Modifier.semantics { heading() })
        Text(body, style = Ds.ui, color = Ds.text2)
        Row(Modifier.fillMaxWidth().clip(DsShapes.sm).background(Ds.bg).dsOutline(Ds.border).padding(12.dp), horizontalArrangement = Arrangement.spacedBy(12.dp), verticalAlignment = Alignment.CenterVertically) {
            Box(Modifier.size(36.dp).clip(DsShapes.sm).background(Crew.Koda.tint), contentAlignment = Alignment.Center) { DsIcon(Lucide.Package, 20.dp, Ds.text) }
            Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
                Text(project, style = Ds.ui.copy(fontFamily = Ds.label.fontFamily), color = Ds.text)
                if (detail != null) Text(detail, style = Ds.label, color = Ds.text3)
            }
        }
        DsButton("Join and build", onJoin, iconEnd = Lucide.ArrowRight)
        Text("Sent to $sentTo · expires $expires · not you? ignore this email", style = Ds.label, color = Ds.text3)
    }
}

data class SharePreviewCard(val title: String, val description: String? = null)
enum class Visibility { Public, Unlisted }

/** Sharing a live build: the link to copy, its preview card, who can open it, building together, and apps to send to. */
@Composable
fun BuildSharePanel(
    url: String,
    preview: SharePreviewCard,
    onClose: () -> Unit,
    onDone: () -> Unit,
    modifier: Modifier = Modifier,
    visibility: Visibility = Visibility.Public,
    onVisibility: ((Visibility) -> Unit)? = null,
    onEditPreview: (() -> Unit)? = null,
    onInvite: (() -> Unit)? = null,
    onMore: (() -> Unit)? = null,
    onApp: ((String) -> Unit)? = null,
) {
    val copier = rememberCopier()
    val host = url.removePrefix("https://").removePrefix("http://")
    Column(modifier.fillMaxWidth().clip(DsShapes.md).background(Ds.surface).dsOutline(Ds.borderStrong, DsShapes.md)) {
        Row(Modifier.fillMaxWidth().padding(start = 20.dp, end = 8.dp, top = 8.dp, bottom = 8.dp), verticalAlignment = Alignment.CenterVertically) {
            Text("Share", style = Ds.body.copy(fontWeight = FontWeight.Medium), color = Ds.text, modifier = Modifier.weight(1f))
            DsIconButton(Lucide.X, "Close", onClose, variant = DsIconButtonVariant.Ghost)
        }
        HorizontalDivider(thickness = Hairline, color = Ds.border)
        Column(Modifier.padding(20.dp), verticalArrangement = Arrangement.spacedBy(16.dp)) {
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
                Box(Modifier.weight(1f).height(40.dp).clip(DsShapes.sm).background(Ds.bg).dsOutline(Ds.borderStrong).padding(horizontal = 12.dp), contentAlignment = Alignment.CenterStart) {
                    DsText(host, Ds.label, Ds.text, singleLine = true)
                }
                DsButton("Copy link", { copier.copy(url) }, variant = DsButtonVariant.Secondary, iconEnd = if (copier.copied) Lucide.Check else Lucide.Copy)
            }
            Row(Modifier.fillMaxWidth().clip(DsShapes.sm).dsOutline(Ds.border)) {
                Box(Modifier.width(120.dp).height(96.dp).background(Ds.surface2), contentAlignment = Alignment.Center) { DsIcon(Lucide.Image, 20.dp, Ds.text3) }
                Column(Modifier.weight(1f).padding(horizontal = 14.dp, vertical = 12.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                    DsText(host, Ds.label, Ds.text3, singleLine = true)
                    Text(preview.title, style = Ds.ui.copy(fontWeight = FontWeight.Medium), color = Ds.text)
                    if (preview.description != null) Text(preview.description, style = Ds.caption, color = Ds.text3)
                    if (onEditPreview != null) DsTextAction("Edit preview", onEditPreview, underline = true)
                }
            }
            Row(horizontalArrangement = Arrangement.spacedBy(12.dp), verticalAlignment = Alignment.CenterVertically) {
                Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
                    Text("Who can open it", style = Ds.ui.copy(fontWeight = FontWeight.Medium), color = Ds.text)
                    Text("Public can appear in search. Unlisted only opens with the link.", style = Ds.caption, color = Ds.text3)
                }
                DsSegmented(
                    "Who can open it", listOf(SegmentOption("public", "Public"), SegmentOption("unlisted", "Unlisted")),
                    if (visibility == Visibility.Public) "public" else "unlisted",
                    { onVisibility?.invoke(if (it == "public") Visibility.Public else Visibility.Unlisted) },
                    surface = SegmentedSurface.Bg,
                )
            }
            if (onInvite != null) {
                Row(Modifier.fillMaxWidth().clip(DsShapes.sm).background(Ds.bg).padding(12.dp), horizontalArrangement = Arrangement.spacedBy(10.dp), verticalAlignment = Alignment.CenterVertically) {
                    DsIcon(Lucide.Users, 14.dp, Ds.text)
                    Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
                        Text("Build together", style = Ds.ui.copy(fontWeight = FontWeight.Medium), color = Ds.text)
                        Text("Invite people to prompt Koda with you.", style = Ds.caption, color = Ds.text3)
                    }
                    DsButton("Invite people", onInvite, variant = DsButtonVariant.Secondary)
                }
            }
            Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                SectionLabel("Send to")
                FlowRow(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    listOf("WhatsApp" to Lucide.Message, "Telegram" to Lucide.Send, "X" to Lucide.Share, "Email" to Lucide.Inbox).forEach { (app, icon) ->
                        Row(
                            Modifier.height(36.dp).clip(DsShapes.full).border(Hairline, Ds.borderStrong, DsShapes.full).clickable(role = Role.Button) { onApp?.invoke(app) }.padding(horizontal = 12.dp),
                            horizontalArrangement = Arrangement.spacedBy(6.dp),
                            verticalAlignment = Alignment.CenterVertically,
                        ) {
                            DsIcon(icon, 14.dp, Ds.text2)
                            Text(app, style = Ds.ui, color = Ds.text)
                        }
                    }
                }
            }
            if (onMore != null) {
                Row(Modifier.fillMaxWidth().heightIn(min = MinTouch).clip(DsShapes.xs).clickable(role = Role.Button, onClick = onMore), horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
                    DsIcon(Lucide.ChevronRight, 14.dp, Ds.text2)
                    Text("More ways to share", style = Ds.ui, color = Ds.text2)
                    Text("QR code · embed", style = Ds.label, color = Ds.text3)
                }
            }
        }
        HorizontalDivider(thickness = Hairline, color = Ds.border)
        Row(Modifier.fillMaxWidth().padding(horizontal = 20.dp, vertical = 12.dp), horizontalArrangement = Arrangement.End) { DsButton("Done", onDone, variant = DsButtonVariant.Inverse) }
    }
}

/** A quote of some code going with the message: file and lines, the snippet, and remove. */
@Composable
fun CodeQuote(file: String, range: String, snippet: String, onRemove: () -> Unit, modifier: Modifier = Modifier) {
    Row(
        modifier.fillMaxWidth().clip(DsShapes.sm).background(Ds.surface).dsOutline(Ds.border).padding(start = 10.dp),
        horizontalArrangement = Arrangement.spacedBy(8.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        DsIcon(Lucide.FileCode, 13.dp, Ds.text2)
        Text("$file · lines $range", style = Ds.mono, color = Ds.text)
        DsText(snippet, Ds.mono, Ds.text3, Modifier.weight(1f), singleLine = true)
        DsBareIcon(Lucide.X, "Remove quote", onRemove, size = 12.dp, tint = Ds.text2)
    }
}

enum class EntryKind(val glyph: NomiGlyph) { Folder(Lucide.Folder), File(Lucide.FileCode), Database(Lucide.Database), Text(Lucide.FileText) }
data class TreeEntry(val name: String, val depth: Int, val kind: EntryKind, val active: Boolean = false)

/**
 * The project's code, read-only: the file tree, the open file with its lines, and asking Koda about the file or
 * the lines you pick (press a line number; press another to make a range).
 */
@Composable
fun CodeView(
    tree: List<TreeEntry>,
    path: String,
    lines: List<List<CodeToken>>,
    onAsk: () -> Unit,
    onAskLines: (from: Int, to: Int) -> Unit,
    modifier: Modifier = Modifier,
    branch: String? = null,
    meta: String? = null,
    selection: Pair<Int, Int>? = null,
    onOpen: ((Int) -> Unit)? = null,
) {
    var range by remember { mutableStateOf(selection) }
    var anchor by remember { mutableStateOf(selection?.first) }
    Row(modifier.fillMaxWidth().height(420.dp).clip(DsShapes.md).background(Ds.bg).dsOutline(Ds.borderStrong, DsShapes.md)) {
        Column(Modifier.width(200.dp).fillMaxHeight().background(Ds.surface).verticalScroll(rememberScrollState()).padding(10.dp), verticalArrangement = Arrangement.spacedBy(2.dp)) {
            Text("FILES${if (branch != null) " · $branch" else ""}", style = Ds.labelCaps, color = Ds.text3, modifier = Modifier.padding(bottom = 8.dp))
            tree.forEachIndexed { index, entry ->
                val bg by animateColorAsState(if (entry.active) Ds.surface2 else Color.Transparent, DsMotion.fast(), label = "tree-active")
                Row(
                    Modifier.fillMaxWidth().height(26.dp).clip(DsShapes.sm).background(bg).clickable(role = Role.Button) { onOpen?.invoke(index) }
                        .padding(start = (8 + entry.depth * 14).dp, end = 8.dp),
                    horizontalArrangement = Arrangement.spacedBy(6.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    DsIcon(entry.kind.glyph, 13.dp, if (entry.active) Ds.text else Ds.text3)
                    DsText(entry.name, Ds.mono, if (entry.active) Ds.text else Ds.text2, singleLine = true)
                }
            }
        }
        Box(Modifier.width(Hairline).fillMaxHeight().background(Ds.border))
        Column(Modifier.weight(1f)) {
            Row(Modifier.fillMaxWidth().background(Ds.surface).padding(horizontal = 14.dp, vertical = 6.dp), horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
                DsIcon(Lucide.FileCode, 14.dp, Ds.text2)
                DsText(path, Ds.mono, Ds.text, singleLine = true)
                DsText(meta ?: "", Ds.label, Ds.text3, Modifier.weight(1f), singleLine = true)
                Row(Modifier.heightIn(min = 36.dp).clip(DsShapes.sm).clickable(role = Role.Button, onClick = onAsk).padding(horizontal = 8.dp), horizontalArrangement = Arrangement.spacedBy(6.dp), verticalAlignment = Alignment.CenterVertically) {
                    DsIcon(Lucide.Message, 14.dp, Ds.text2)
                    Text("Ask Koda about this file", style = Ds.caption, color = Ds.text2)
                }
            }
            HorizontalDivider(thickness = Hairline, color = Ds.border)
            Column(Modifier.weight(1f).verticalScroll(rememberScrollState()).horizontalScroll(rememberScrollState()).padding(vertical = 12.dp)) {
                lines.forEachIndexed { index, line ->
                    val n = index + 1
                    val picked = range?.let { n in it.first..it.second } == true
                    val bg by animateColorAsState(if (picked) Ds.accentBg else Color.Transparent, DsMotion.fast(), label = "line-pick")
                    Row(Modifier.height(22.dp).background(bg), verticalAlignment = Alignment.CenterVertically) {
                        Text(
                            n.toString(), style = Ds.mono, color = if (picked) Ds.accent else Ds.text3, textAlign = TextAlign.End,
                            modifier = Modifier.width(44.dp).clickable(role = Role.Button) {
                                val start = anchor
                                if (start != null && range != null && n != start) range = minOf(start, n) to maxOf(start, n)
                                else { anchor = n; range = n to n }
                            }.semantics { contentDescription = "Line $n" },
                        )
                        Box(Modifier.width(16.dp))
                        Text(
                            buildAnnotatedString { line.forEach { t -> withStyle(SpanStyle(color = t.kind.tint())) { append(t.text) } } },
                            style = Ds.mono, softWrap = false, modifier = Modifier.padding(end = 16.dp),
                        )
                    }
                    if (range != null && n == range!!.second) {
                        val (from, to) = range!!
                        AnimatedVisibility(true, enter = scaleIn(DsMotion.spatial()) + fadeIn(DsMotion.fast())) {
                            Row(
                                Modifier.padding(start = 60.dp, top = 2.dp, bottom = 2.dp).clip(DsShapes.full).background(Ds.text).clickable(role = Role.Button) { onAskLines(from, to) }.padding(horizontal = 10.dp, vertical = 4.dp),
                                horizontalArrangement = Arrangement.spacedBy(6.dp),
                                verticalAlignment = Alignment.CenterVertically,
                            ) {
                                DsIcon(Lucide.Message, 13.dp, Ds.bg)
                                Text("Ask Koda about lines ${if (from == to) "$from" else "$from–$to"}", style = Ds.caption, color = Ds.bg)
                            }
                        }
                    }
                }
            }
            HorizontalDivider(thickness = Hairline, color = Ds.border)
            Row(Modifier.fillMaxWidth().padding(horizontal = 14.dp, vertical = 8.dp), horizontalArrangement = Arrangement.spacedBy(6.dp), verticalAlignment = Alignment.CenterVertically) {
                DsIcon(Lucide.Lock, 12.dp, Ds.text3)
                Text("Read-only. Ask Koda in the chat to change anything; every change is a commit you can roll back.", style = Ds.caption, color = Ds.text3)
            }
        }
    }
}

@Composable
private fun CodeTokenKind.tint(): Color = when (this) {
    CodeTokenKind.Keyword, CodeTokenKind.Type -> Ds.codeKeyword
    CodeTokenKind.String -> Ds.codeString
    CodeTokenKind.Fn -> Ds.codeFn
    CodeTokenKind.Number -> Ds.codeNumber
    CodeTokenKind.Comment -> Ds.text3
    CodeTokenKind.Punct -> Ds.text2
    CodeTokenKind.Plain -> Ds.text
}

/** Which device the preview shows, and sending it to your phone. */
@Composable
fun DeviceBar(device: String, onDevice: () -> Unit, modifier: Modifier = Modifier, onPhone: (() -> Unit)? = null) {
    FlowRow(modifier, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        Pill(onDevice) {
            DsIcon(Lucide.Smartphone, 14.dp, Ds.text2)
            Text(device, style = Ds.caption, color = Ds.text)
            DsIcon(Lucide.ChevronDown, 12.dp, Ds.text3)
        }
        if (onPhone != null) {
            Pill(onPhone) {
                DsIcon(Lucide.Send, 14.dp, Ds.text2)
                Text("Open on your phone", style = Ds.caption, color = Ds.text)
            }
        }
    }
}

@Composable
private fun Pill(onClick: () -> Unit, content: @Composable androidx.compose.foundation.layout.RowScope.() -> Unit) {
    Row(
        Modifier.height(32.dp).clip(DsShapes.full).border(Hairline, Ds.borderStrong, DsShapes.full).clickable(role = Role.Button, onClick = onClick).padding(horizontal = 12.dp),
        horizontalArrangement = Arrangement.spacedBy(6.dp),
        verticalAlignment = Alignment.CenterVertically,
        content = content,
    )
}

/** Everything that's yours to take: the repo, the database dump and env names, to GitHub or as a zip. */
@Composable
fun ExportPanel(project: String, repo: String, onPush: () -> Unit, onDownload: () -> Unit, onClose: () -> Unit, modifier: Modifier = Modifier, database: String? = null, env: String? = null) {
    val items = listOfNotNull(
        Triple(Lucide.GitBranch, "Repo", repo),
        database?.let { Triple(Lucide.Database, "Database dump", it) },
        env?.let { Triple(Lucide.KeyRound, "Env var names", it) },
    )
    Column(modifier.fillMaxWidth().clip(DsShapes.md).background(Ds.surface).dsOutline(Ds.borderStrong, DsShapes.md)) {
        Row(Modifier.fillMaxWidth().padding(start = 20.dp, end = 8.dp, top = 12.dp, bottom = 12.dp), verticalAlignment = Alignment.CenterVertically) {
            Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
                Text("Export $project", style = Ds.body.copy(fontWeight = FontWeight.Medium), color = Ds.text)
                Text("Everything Koda built is yours. Included on Free, no fee.", style = Ds.ui, color = Ds.text3)
            }
            DsIconButton(Lucide.X, "Close", onClose, variant = DsIconButtonVariant.Ghost)
        }
        HorizontalDivider(thickness = Hairline, color = Ds.border)
        Column(Modifier.padding(20.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
            SectionLabel("Included")
            items.forEach { (icon, title, detail) ->
                Row(Modifier.fillMaxWidth().padding(top = 10.dp, bottom = 9.dp), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                    Box(Modifier.size(28.dp).clip(DsShapes.sm).background(Ds.surface2), contentAlignment = Alignment.Center) { DsIcon(icon, 14.dp, Ds.text2) }
                    Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(3.dp)) {
                        Text(title, style = Ds.ui.copy(fontWeight = FontWeight.Medium), color = Ds.text)
                        Text(detail, style = Ds.label, color = Ds.text3)
                    }
                    DsIcon(Lucide.Check, 14.dp, Ds.accent, Modifier.padding(top = 6.dp))
                }
                HorizontalDivider(thickness = Hairline, color = Ds.border)
            }
            if (env != null) Text("Env var values are not exported. Set them again wherever you deploy.", style = Ds.ui, color = Ds.text3, modifier = Modifier.padding(top = 8.dp))
        }
        HorizontalDivider(thickness = Hairline, color = Ds.border)
        FlowRow(Modifier.fillMaxWidth().padding(horizontal = 20.dp, vertical = 12.dp), horizontalArrangement = Arrangement.spacedBy(8.dp, Alignment.End)) {
            DsButton("Push to GitHub", onPush, variant = DsButtonVariant.Secondary, iconEnd = Lucide.GitBranch)
            DsButton("Download zip", onDownload, variant = DsButtonVariant.Secondary, iconEnd = Lucide.Download)
        }
    }
}

data class BuildFile(val path: String, val state: FileState, val change: String? = null)

/** The files a build writes, as it writes them: written, being written, still to come (faded). */
@Composable
fun FileList(files: List<BuildFile>, total: Int, modifier: Modifier = Modifier) {
    val done = files.count { it.state == FileState.Done }
    Column(modifier.fillMaxWidth(), verticalArrangement = Arrangement.spacedBy(2.dp)) {
        Row(Modifier.fillMaxWidth().padding(horizontal = 12.dp, vertical = 4.dp)) {
            Text("FILES", style = Ds.labelCaps, color = Ds.text3, modifier = Modifier.weight(1f))
            AnimatedContent(done, label = "files-done") { Text("$it of $total written", style = Ds.label, color = Ds.text3) }
        }
        files.forEach { file -> FileRow(file.path, Modifier.alpha(if (file.state == FileState.Queued) QUEUED_ALPHA else 1f), file.change, file.state) }
    }
}

/** The first time in: what Nomi does, and what to call you. */
@Composable
fun FirstRun(onContinue: (String) -> Unit, modifier: Modifier = Modifier, name: String = "") {
    var value by remember { mutableStateOf(name) }
    Column(modifier.fillMaxWidth().widthIn(max = 440.dp), verticalArrangement = Arrangement.spacedBy(20.dp)) {
        Mascot(Crew.Nomi, 32.dp)
        Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
            Text("What should we call you?", style = MaterialTheme.typography.headlineSmall.copy(fontWeight = FontWeight.Medium), color = Ds.text, modifier = Modifier.semantics { heading() })
            Text("Nomi builds real web apps from a description. You get a live link, and the code is yours to export.", style = Ds.body, color = Ds.text2)
        }
        DsTextField("Your name", value, { value = it }, helper = "Shown on your projects. Change it in Settings › Profile.", mono = true)
        DsButton("Continue", { onContinue(value.trim()) }, iconEnd = Lucide.ArrowRight, enabled = value.isNotBlank())
    }
}

/** Following a teammate around the project: whose view you're seeing and where, and stop. */
@Composable
fun FollowBar(name: String, peer: Int, where: String, onStop: () -> Unit, modifier: Modifier = Modifier) {
    val bg = if (peer == 1) Ds.peer1 else Ds.peer2
    Row(modifier.fillMaxWidth().height(32.dp).background(bg).padding(horizontal = 10.dp).semantics { liveRegion = LiveRegionMode.Polite }, horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
        DsIcon(Lucide.Eye, 14.dp, Ds.bg)
        DsText("Following $name · $where", Ds.caption.copy(fontWeight = FontWeight.Medium), Ds.bg, Modifier.weight(1f), singleLine = true)
        DsTextAction("Stop following", onStop, color = Ds.bg, underline = true)
    }
}

/** Someone joined the project, what they can do, and who publishes: a divider in the thread. */
@Composable
fun JoinedNote(name: String, role: InviteRole, modifier: Modifier = Modifier, publisher: String? = null) {
    val can = when (role) { InviteRole.Build -> "can build"; InviteRole.Prompt -> "can prompt"; InviteRole.View -> "can view" }
    val label = listOfNotNull("$name joined", can, publisher?.let { "$it publishes" }).joinToString(" · ")
    Row(modifier.fillMaxWidth().semantics(mergeDescendants = true) { contentDescription = label }, horizontalArrangement = Arrangement.spacedBy(10.dp), verticalAlignment = Alignment.CenterVertically) {
        Box(Modifier.weight(1f).height(Hairline).background(Ds.border))
        Row(horizontalArrangement = Arrangement.spacedBy(6.dp), verticalAlignment = Alignment.CenterVertically) {
            DsIcon(Lucide.UserPlus, 12.dp, Ds.text3)
            Text(label, style = Ds.label, color = Ds.text3)
        }
        Box(Modifier.weight(1f).height(Hairline).background(Ds.border))
    }
}

/** A teammate's pointer over the preview, with their name, in their colour. */
@Composable
fun PeerCursor(name: String, peer: Int, modifier: Modifier = Modifier) {
    val ink = if (peer == 1) Ds.peer1 else Ds.peer2
    Column(modifier) {
        DsIcon(Lucide.Cursor, 18.dp, ink)
        Text(name, style = Ds.caption.copy(fontWeight = FontWeight.Medium), color = Ds.bg, modifier = Modifier.padding(start = 12.dp).clip(DsShapes.full).background(ink).padding(horizontal = 8.dp, vertical = 2.dp))
    }
}

/** A phone around a preview: the bezel, its status bar, and the app inside. */
@Composable
fun PhoneFrame(modifier: Modifier = Modifier, width: androidx.compose.ui.unit.Dp = 300.dp, content: @Composable ColumnScope.() -> Unit) {
    val shape = androidx.compose.foundation.shape.RoundedCornerShape(44.dp)
    Box(modifier.width(width).aspectRatio(PHONE_RATIO).clip(shape).background(Ds.bezel).padding(10.dp)) {
        Column(Modifier.fillMaxWidth().fillMaxHeight().clip(androidx.compose.foundation.shape.RoundedCornerShape(34.dp)).background(Color.White)) {
            PhoneStatusBar(ink = Color.Black)
            Column(Modifier.weight(1f).fillMaxWidth(), content = content)
        }
    }
}

data class Presenter(val name: String, val initials: String, val peer: Int? = null, val online: Boolean = false)

/** Who's in the project now: overlapping initials, a dot for who's online. */
@Composable
fun Presence(people: List<Presenter>, modifier: Modifier = Modifier) {
    Row(modifier.semantics { contentDescription = "People here" }, horizontalArrangement = Arrangement.spacedBy((-6).dp)) {
        people.forEach { person ->
            Box {
                PeerAvatar(person.initials, peer = person.peer, ring = true)
                if (person.online) Box(Modifier.align(Alignment.BottomEnd).size(8.dp).clip(DsShapes.full).background(Ds.accent).border(1.5.dp, Ds.bg, DsShapes.full))
            }
        }
    }
}

data class PresentPerson(val name: String, val initials: String, val where: String, val peer: Int? = null, val you: Boolean = false)

/** Who's here and where each of them is, with follow. */
@Composable
fun PresenceMenu(people: List<PresentPerson>, onFollow: (Int) -> Unit, modifier: Modifier = Modifier) {
    Column(modifier.width(300.dp).clip(DsShapes.md).background(Ds.surface).dsOutline(Ds.borderStrong, DsShapes.md).padding(vertical = 8.dp)) {
        Text("IN THIS PROJECT · ${people.size}", style = Ds.labelCaps, color = Ds.text3, modifier = Modifier.padding(horizontal = 14.dp, vertical = 6.dp))
        people.forEachIndexed { index, person ->
            Row(Modifier.fillMaxWidth().padding(horizontal = 14.dp, vertical = 6.dp), horizontalArrangement = Arrangement.spacedBy(10.dp), verticalAlignment = Alignment.CenterVertically) {
                PeerAvatar(person.initials, peer = person.peer, ring = !person.you && person.peer != null)
                Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
                    DsText(person.name, Ds.ui.copy(fontWeight = FontWeight.Medium), Ds.text, singleLine = true)
                    Text(person.where, style = Ds.caption, color = Ds.text3)
                }
                if (!person.you) DsButton("Follow", { onFollow(index) }, variant = DsButtonVariant.Secondary)
            }
        }
    }
}

data class BuildingStep(val label: String, val state: StepState)

/** The preview before the first deploy: building, and where each step is. */
@Composable
fun PreviewBuilding(steps: List<BuildingStep>, modifier: Modifier = Modifier) {
    Column(modifier.fillMaxWidth().padding(32.dp), horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(20.dp)) {
        Column(horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(6.dp)) {
            DsSpinner(22.dp, Ds.text2)
            Text("Building your app", style = Ds.body.copy(fontWeight = FontWeight.Medium), color = Ds.text)
            Text("The preview opens here after the first deploy", style = Ds.caption, color = Ds.text3)
        }
        Column(Modifier.width(280.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
            steps.forEach { step ->
                Row(horizontalArrangement = Arrangement.spacedBy(10.dp), verticalAlignment = Alignment.CenterVertically) {
                    AnimatedContent(step.state, label = "building-step") { StepMark(it) }
                    Text(step.label, style = Ds.ui, color = if (step.state == StepState.Waiting) Ds.text3 else Ds.text)
                }
            }
        }
    }
}

enum class PublishState { Idle, Ready, Publishing, Failed }

/** Publish, as it is: up to date, ready (with how many changes), publishing (with the time), or failed (retry). */
@Composable
fun PublishButton(state: PublishState, modifier: Modifier = Modifier, count: Int = 0, elapsed: String = "", onPublish: (() -> Unit)? = null, onRetry: (() -> Unit)? = null) {
    val (bg, ink, edge) = when (state) {
        PublishState.Idle -> Triple(Color.Transparent, Ds.text2, Ds.borderStrong)
        PublishState.Ready -> Triple(Ds.accent, Ds.accentInk, Ds.accent)
        PublishState.Publishing -> Triple(Ds.accentBg, Ds.accent, Ds.accentBg)
        PublishState.Failed -> Triple(Ds.dangerBg, Ds.danger, Ds.dangerBorder)
    }
    val fill by animateColorAsState(bg, DsMotion.effects(), label = "publish-fill")
    val enabled = state == PublishState.Ready || state == PublishState.Failed
    Row(
        modifier.height(30.dp).clip(DsShapes.sm).background(fill).border(Hairline, edge, DsShapes.sm)
            .clickable(enabled = enabled, role = Role.Button) { if (state == PublishState.Failed) onRetry?.invoke() else onPublish?.invoke() }
            .padding(horizontal = 12.dp),
        horizontalArrangement = Arrangement.spacedBy(6.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        AnimatedContent(state, label = "publish-state") { s ->
            Row(horizontalArrangement = Arrangement.spacedBy(6.dp), verticalAlignment = Alignment.CenterVertically) {
                when (s) {
                    PublishState.Idle -> { DsIcon(Lucide.Check, 14.dp, ink); Text("Up to date", style = Ds.ui, color = ink) }
                    PublishState.Ready -> {
                        DsIcon(Lucide.Rocket, 14.dp, ink)
                        Text("Publish", style = Ds.ui.copy(fontWeight = FontWeight.Medium), color = ink)
                        if (count > 0) Text(count.toString(), style = Ds.label, color = Ds.accent, modifier = Modifier.clip(DsShapes.full).background(Ds.accentInk).padding(horizontal = 6.dp))
                    }
                    PublishState.Publishing -> { DsSpinner(14.dp, ink); Text("Publishing… $elapsed", style = Ds.ui.copy(fontWeight = FontWeight.Medium), color = ink) }
                    PublishState.Failed -> { DsIcon(Lucide.Alert, 14.dp, ink); Text("Publish failed · Retry", style = Ds.ui.copy(fontWeight = FontWeight.Medium), color = ink) }
                }
            }
        }
    }
}

/** A teammate's change waiting for the person who publishes. */
@Composable
fun PublishRequest(who: Person, onPublish: () -> Unit, modifier: Modifier = Modifier, meta: String? = null, onReview: (() -> Unit)? = null) {
    DsCard(
        modifier,
        footer = {
            DsButton("Publish", onPublish, iconEnd = Lucide.ArrowRight)
            if (onReview != null) DsButton("Review in preview", onReview, variant = DsButtonVariant.Ghost, iconStart = Lucide.Users)
        },
    ) {
        Row(Modifier.padding(horizontal = 14.dp, vertical = 12.dp), horizontalArrangement = Arrangement.spacedBy(10.dp), verticalAlignment = Alignment.CenterVertically) {
            PeerAvatar(who.initials, peer = who.peer)
            Column(verticalArrangement = Arrangement.spacedBy(2.dp)) {
                Text("${who.first}'s change is ready to publish", style = Ds.ui.copy(fontWeight = FontWeight.Medium), color = Ds.text)
                if (meta != null) Text(meta, style = Ds.label, color = Ds.text3)
            }
        }
    }
}

/** The asks in line for Koda in a shared build, numbered, with who asked and which is running and next. */
@Composable
fun RequestQueue(asks: List<ai.nomi.designsystem.ds.mobile.Ask>, modifier: Modifier = Modifier, onFirst: ((Int) -> Unit)? = null) {
    val next = asks.indexOfFirst { it.state == ai.nomi.designsystem.ds.mobile.AskState.Next }
    DsCard(
        modifier,
        header = {
            Mascot(Crew.Koda, 20.dp)
            Text(if (asks.size == 1) "1 ask in line" else "${asks.size} asks in line", style = Ds.ui.copy(fontWeight = FontWeight.Medium), color = Ds.text)
            Text("one change at a time", style = Ds.label, color = Ds.text3, modifier = Modifier.weight(1f))
        },
        footer = {
            Text("Koda finishes one change before starting the next, so edits never collide.", style = Ds.caption, color = Ds.text3, modifier = Modifier.footerNote(this))
            if (onFirst != null && next > 0) DsButton("Do ${asks[next].who.first}'s first", { onFirst(next) }, variant = DsButtonVariant.Ghost, iconStart = Lucide.Users)
        },
    ) {
        asks.forEachIndexed { index, ask ->
            if (index > 0) HorizontalDivider(thickness = Hairline, color = Ds.border)
            Row(Modifier.fillMaxWidth().padding(horizontal = 14.dp, vertical = 10.dp), horizontalArrangement = Arrangement.spacedBy(10.dp), verticalAlignment = Alignment.CenterVertically) {
                Text("${index + 1}", style = Ds.label, color = Ds.text3, modifier = Modifier.width(12.dp))
                PeerAvatar(ask.who.initials, peer = ask.who.peer, small = true)
                Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
                    Text(ask.text, style = Ds.ui, color = Ds.text)
                    if (ask.meta != null) Text(ask.meta, style = Ds.label, color = Ds.text3)
                }
                when (ask.state) {
                    ai.nomi.designsystem.ds.mobile.AskState.Running -> DsStatus(StatusKind.Working, "Koda is building", agent = Crew.Koda)
                    ai.nomi.designsystem.ds.mobile.AskState.Next -> DsTag(TagKind.Neutral, "Next")
                    else -> {}
                }
            }
        }
    }
}

private const val SPLIT_MIN = 320f
private const val PREVIEW_MIN = 480f

/** The chat/preview split's handle: drag it (the grip lights up), or double-press to reset. */
@Composable
fun ResizeHandle(width: Float, available: Float, onResize: (Float) -> Unit, modifier: Modifier = Modifier, onReset: (() -> Unit)? = null, showActive: Boolean = false) {
    val density = LocalDensity.current
    val interaction = remember { MutableInteractionSource() }
    val dragged by interaction.collectIsDraggedAsState()
    val active = dragged || showActive
    val grip by animateColorAsState(if (active) Ds.accent else Ds.surface2, DsMotion.fast(), label = "grip")
    val dot by animateColorAsState(if (active) Ds.accentInk else Ds.text3, DsMotion.fast(), label = "grip-dot")
    val max = (available - PREVIEW_MIN).coerceAtLeast(SPLIT_MIN)
    Box(
        modifier.width(9.dp).fillMaxHeight().heightIn(min = 40.dp)
            .draggable(rememberDraggableState { delta -> onResize((width + with(density) { delta.toDp().value }).coerceIn(SPLIT_MIN, max)) }, Orientation.Horizontal, interactionSource = interaction)
            .clickable(enabled = onReset != null, role = Role.Button) { onReset?.invoke() }
            .semantics { contentDescription = "Resize chat" },
        contentAlignment = Alignment.Center,
    ) {
        Box(Modifier.align(Alignment.CenterStart).width(if (active) 2.dp else Hairline).fillMaxHeight().background(if (active) Ds.accent else Ds.border))
        Column(Modifier.clip(DsShapes.xs).background(grip).padding(horizontal = 2.dp, vertical = 6.dp), verticalArrangement = Arrangement.spacedBy(3.dp)) {
            repeat(TYPING_DOTS) { Box(Modifier.size(3.dp).clip(DsShapes.full).background(dot)) }
        }
    }
}

/** What a teammate can do in a project: build, prompt, or view; the chosen one is checked. */
@Composable
fun RoleMenu(value: InviteRole, onChange: (InviteRole) -> Unit, modifier: Modifier = Modifier) {
    val roles = listOf(
        Triple(InviteRole.Build, "Can build", "Prompt Koda, edit and publish") to Lucide.Zap,
        Triple(InviteRole.Prompt, "Can prompt", "Ask Koda for changes; you publish") to Lucide.Message,
        Triple(InviteRole.View, "Can view", "Open the preview and the code") to Lucide.Eye,
    )
    Column(modifier.width(300.dp).clip(DsShapes.md).background(Ds.surface).dsOutline(Ds.borderStrong, DsShapes.md).padding(6.dp).selectableGroup().semantics { contentDescription = "Role" }) {
        roles.forEach { (role, icon) ->
            val (key, title, detail) = role
            val chosen = key == value
            val bg by animateColorAsState(if (chosen) Ds.surface2 else Color.Transparent, DsMotion.fast(), label = "role-bg")
            Row(
                Modifier.fillMaxWidth().clip(DsShapes.sm).background(bg).selectable(chosen, role = Role.RadioButton) { onChange(key) }.padding(horizontal = 10.dp, vertical = 8.dp),
                horizontalArrangement = Arrangement.spacedBy(10.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                DsIcon(icon, 14.dp, if (chosen) Ds.text else Ds.text2)
                Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
                    Text(title, style = Ds.ui.copy(fontWeight = FontWeight.Medium), color = Ds.text)
                    Text(detail, style = Ds.caption, color = Ds.text3)
                }
                AnimatedVisibility(chosen, enter = scaleIn(DsMotion.spatial()) + fadeIn(DsMotion.fast()), exit = scaleOut(DsMotion.fast()) + fadeOut(DsMotion.fast())) {
                    DsIcon(Lucide.Check, 14.dp, Ds.accent)
                }
            }
        }
    }
}

/** A teammate's message in a shared build's chat (the web bubble, not the phone's). */
@Composable
fun TeammateMessage(who: Person, text: String, modifier: Modifier = Modifier, time: String? = null, to: String? = null) =
    ai.nomi.designsystem.ds.mobile.TeammateBubble(who, text, modifier, time, to, phone = false)

enum class TermLineKind { Command, Out, Ok, Fail, Error, Warn, Blank }
data class TermLine(val kind: TermLineKind, val text: String = "", val time: String? = null)

/** What Koda ran, read-only: the run, its log (following the end as it grows), copy, and asking Koda to fix a failure. */
@Composable
fun Terminal(run: String, lines: List<TermLine>, modifier: Modifier = Modifier, failureTitle: String? = null, failureDetail: String? = null, onFix: (() -> Unit)? = null) {
    val copier = rememberCopier()
    val scroll = rememberScrollState()
    LaunchedEffect(lines.size) { scroll.animateScrollTo(scroll.maxValue) }
    Column(modifier.fillMaxWidth().clip(DsShapes.md).background(Ds.bg).dsOutline(Ds.borderStrong, DsShapes.md)) {
        Row(Modifier.fillMaxWidth().background(Ds.surface).padding(start = 14.dp, end = 6.dp), horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
            DsIcon(Lucide.Terminal, 14.dp, Ds.text)
            DsText(run, Ds.mono, Ds.text, singleLine = true)
            Text("Koda ran these · read-only", style = Ds.label, color = Ds.text3, modifier = Modifier.weight(1f))
            Row(Modifier.heightIn(min = MinTouch).clip(DsShapes.xs).clickable(role = Role.Button) { copier.copy(lines.joinToString("\n") { it.text }) }.padding(horizontal = 8.dp), horizontalArrangement = Arrangement.spacedBy(6.dp), verticalAlignment = Alignment.CenterVertically) {
                CopyGlyph(copier.copied, 14.dp, Ds.text2)
                Text(if (copier.copied) "Copied" else "Copy log", style = Ds.caption, color = Ds.text2)
            }
        }
        HorizontalDivider(thickness = Hairline, color = Ds.border)
        Column(Modifier.fillMaxWidth().height(220.dp).verticalScroll(scroll).padding(horizontal = 14.dp, vertical = 10.dp).semantics { liveRegion = LiveRegionMode.Polite }) {
            lines.forEach { line ->
                Row(
                    Modifier.fillMaxWidth().heightIn(min = 20.dp).background(if (line.kind == TermLineKind.Fail) Ds.dangerBg else Color.Transparent),
                    horizontalArrangement = Arrangement.spacedBy(12.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Text(
                        line.text, modifier = Modifier.weight(1f),
                        style = Ds.mono.copy(fontWeight = if (line.kind == TermLineKind.Command) FontWeight.Medium else FontWeight.Normal),
                        color = when (line.kind) {
                            TermLineKind.Command -> Ds.text
                            TermLineKind.Ok -> Ds.accent
                            TermLineKind.Fail, TermLineKind.Error -> Ds.danger
                            TermLineKind.Warn -> Ds.warning
                            else -> Ds.text2
                        },
                    )
                    if (line.time != null) Text(line.time, style = Ds.mono, color = Ds.text3)
                }
            }
        }
        HorizontalDivider(thickness = Hairline, color = Ds.border)
        if (failureTitle != null) {
            FlowRow(Modifier.fillMaxWidth().background(Ds.surface).padding(horizontal = 14.dp, vertical = 10.dp), horizontalArrangement = Arrangement.spacedBy(10.dp), itemVerticalAlignment = Alignment.CenterVertically) {
                DsIcon(Lucide.Alert, 14.dp, Ds.danger)
                Column(Modifier.weight(1f).widthIn(min = 180.dp), verticalArrangement = Arrangement.spacedBy(2.dp)) {
                    Text(failureTitle, style = Ds.ui.copy(fontWeight = FontWeight.Medium), color = Ds.text)
                    if (failureDetail != null) Text(failureDetail, style = Ds.caption, color = Ds.text2)
                }
                if (onFix != null) DsButton("Ask Koda to fix", onFix, iconEnd = Lucide.ArrowRight)
            }
        } else {
            Row(Modifier.fillMaxWidth().padding(horizontal = 14.dp, vertical = 8.dp), horizontalArrangement = Arrangement.spacedBy(6.dp), verticalAlignment = Alignment.CenterVertically) {
                DsIcon(Lucide.Info, 12.dp, Ds.text3)
                Text("Koda runs the commands. Want something run? Ask in the chat.", style = Ds.caption, color = Ds.text3)
            }
        }
    }
}

/** A teammate is typing: their initials, the words, and three dots that rise in turn. */
@Composable
fun Typing(who: Person, modifier: Modifier = Modifier) {
    val transition = rememberInfiniteTransition(label = "typing")
    val phase by transition.animateFloat(0f, 1f, infiniteRepeatable(tween(TYPING_MS * TYPING_DOTS, easing = LinearEasing), RepeatMode.Restart), label = "typing-phase")
    val still = ai.nomi.designsystem.theme.NomiTheme.reducedMotion
    Row(modifier.semantics { liveRegion = LiveRegionMode.Polite }, horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
        PeerAvatar(who.initials, peer = who.peer, small = true)
        Text("${who.first} is typing", style = Ds.caption, color = Ds.text3)
        Row(horizontalArrangement = Arrangement.spacedBy(3.dp), verticalAlignment = Alignment.CenterVertically) {
            repeat(TYPING_DOTS) { index ->
                val local = ((phase - index * TYPING_STAGGER) % 1f + 1f) % 1f
                val lift = if (still) 0f else if (local < TYPING_STAGGER) kotlin.math.sin(local / TYPING_STAGGER * kotlin.math.PI).toFloat() else 0f
                Box(Modifier.offset(y = (-3 * lift).dp).size(4.dp).alpha(LOW_ALPHA + (1 - LOW_ALPHA) * lift).clip(DsShapes.full).background(Ds.text3))
            }
        }
    }
}

/** A sheet over the workspace (domain, database, settings): its title and project, expand, close, and that esc closes it. */
@Composable
fun WorkspaceSheet(icon: NomiGlyph, title: String, onClose: () -> Unit, modifier: Modifier = Modifier, meta: String? = null, onExpand: (() -> Unit)? = null, content: @Composable ColumnScope.() -> Unit) {
    Column(modifier.fillMaxWidth().clip(DsShapes.md).background(Ds.surface).dsOutline(Ds.borderStrong, DsShapes.md)) {
        Row(Modifier.fillMaxWidth().padding(start = 14.dp, end = 4.dp, top = 4.dp, bottom = 4.dp), horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
            DsIcon(icon, 14.dp, Ds.text2)
            Text(title, style = Ds.ui.copy(fontWeight = FontWeight.Medium), color = Ds.text, modifier = Modifier.semantics { heading() })
            DsText(meta ?: "", Ds.label, Ds.text3, Modifier.weight(1f), singleLine = true)
            if (onExpand != null) DsIconButton(Lucide.Maximize, "Expand", onExpand, variant = DsIconButtonVariant.Ghost)
            DsIconButton(Lucide.X, "Close", onClose, variant = DsIconButtonVariant.Ghost)
        }
        HorizontalDivider(thickness = Hairline, color = Ds.border)
        Column(Modifier.weight(1f, fill = false).padding(16.dp), content = content)
        HorizontalDivider(thickness = Hairline, color = Ds.border)
        Text("esc closes · preview keeps running", style = Ds.label, color = Ds.text3, modifier = Modifier.padding(horizontal = 14.dp, vertical = 8.dp))
    }
}
