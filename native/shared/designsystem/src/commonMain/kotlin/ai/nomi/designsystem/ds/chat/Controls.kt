package ai.nomi.designsystem.ds.chat

import ai.nomi.designsystem.ds.Crew
import ai.nomi.designsystem.ds.DsCard
import ai.nomi.designsystem.ds.DsCardTitle
import ai.nomi.designsystem.ds.DsRule
import ai.nomi.designsystem.ds.Ds
import ai.nomi.designsystem.ds.footerNote
import ai.nomi.designsystem.ds.DsBareField
import ai.nomi.designsystem.ds.DsBareIcon
import ai.nomi.designsystem.ds.DsButton
import ai.nomi.designsystem.ds.DsButtonVariant
import ai.nomi.designsystem.ds.DsDialog
import ai.nomi.designsystem.ds.DsIcon
import ai.nomi.designsystem.ds.DsIconButton
import ai.nomi.designsystem.ds.DsIconButtonVariant
import ai.nomi.designsystem.ds.DsMenu
import ai.nomi.designsystem.ds.DsMotion
import ai.nomi.designsystem.ds.DsSearch
import ai.nomi.designsystem.ds.DsShapes
import ai.nomi.designsystem.ds.DsSpinner
import ai.nomi.designsystem.ds.DsText
import ai.nomi.designsystem.ds.DsTextAction
import ai.nomi.designsystem.ds.Hairline
import ai.nomi.designsystem.ds.Mascot
import ai.nomi.designsystem.ds.MascotStack
import ai.nomi.designsystem.ds.MenuEntry
import ai.nomi.designsystem.ds.MinTouch
import ai.nomi.designsystem.ds.blinkAlpha
import ai.nomi.designsystem.ds.dsOutline
import ai.nomi.designsystem.icon.Lucide
import ai.nomi.designsystem.icon.NomiGlyph
import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.scaleIn
import androidx.compose.animation.scaleOut
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutVertically
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.selection.selectable
import androidx.compose.foundation.selection.selectableGroup
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.isCtrlPressed
import androidx.compose.ui.input.key.isMetaPressed
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.onPreviewKeyEvent
import androidx.compose.ui.input.key.type
import androidx.compose.ui.semantics.LiveRegionMode
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.liveRegion
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp

private const val LEVEL_MIN_DP = 4
private const val LEVEL_RANGE_DP = 24
private const val PERCENT = 100

enum class AttachSource { File, Photo, Drive, Link }

/** What you can attach: a file, a photo, Drive (when someone reaches it for you), a link; and the limits. */
@Composable
fun AttachMenu(max: String, onPick: (AttachSource) -> Unit, modifier: Modifier = Modifier, driveBy: String? = null, inline: Boolean = false, trigger: @Composable (open: () -> Unit, expanded: Boolean) -> Unit = { _, _ -> }) {
    DsMenu(
        listOfNotNull(
            MenuEntry.Item("Upload a file", Lucide.Paperclip, onSelect = { onPick(AttachSource.File) }),
            MenuEntry.Item("Photo or screenshot", Lucide.Image, onSelect = { onPick(AttachSource.Photo) }),
            driveBy?.let { MenuEntry.Item("From Google Drive", Lucide.HardDrive, meta = it, onSelect = { onPick(AttachSource.Drive) }) },
            MenuEntry.Item("Paste a link", Lucide.Link, onSelect = { onPick(AttachSource.Link) }),
        ),
        modifier, footer = "PDF, images, sheets, zip · max $max", wide = true, inline = inline, trigger = trigger,
    )
}

/** A file going with the message: its name and detail; while uploading a thin bar fills as it goes. */
@Composable
fun AttachmentChip(name: String, detail: String, modifier: Modifier = Modifier, progress: Float? = null, onRemove: (() -> Unit)? = null) {
    val uploading = progress != null
    val fill by animateFloatAsState((progress ?: 0f).coerceIn(0f, 1f), DsMotion.spatial(), label = "upload-fill")
    Row(
        modifier.then(if (uploading) Modifier.width(250.dp) else Modifier.widthIn(max = 320.dp)).clip(DsShapes.sm).background(Ds.surface).dsOutline(Ds.borderStrong)
            .padding(start = 10.dp, end = 4.dp, top = 6.dp, bottom = 6.dp),
        horizontalArrangement = Arrangement.spacedBy(10.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        DsIcon(Lucide.FileText, 20.dp, Ds.text2)
        Column(Modifier.weight(1f, fill = uploading), verticalArrangement = Arrangement.spacedBy(2.dp)) {
            DsText(name, Ds.mono, Ds.text, singleLine = true)
            if (uploading) {
                Box(Modifier.padding(vertical = 2.dp).fillMaxWidth().height(3.dp).clip(DsShapes.xs).background(Ds.surface2).semantics { contentDescription = "$name ${(fill * PERCENT).toInt()}%" }) {
                    Box(Modifier.fillMaxWidth(fill).fillMaxHeight().clip(DsShapes.xs).background(Ds.accent))
                }
            }
            DsText(detail, Ds.label, Ds.text3, singleLine = true)
        }
        if (onRemove != null) DsBareIcon(Lucide.X, if (uploading) "Cancel upload of $name" else "Remove $name", onRemove, tint = Ds.text2)
    }
}

/** A picture going with the message, as a thumbnail, with a remove badge. */
@Composable
fun AttachmentImage(description: String, modifier: Modifier = Modifier, onRemove: (() -> Unit)? = null, image: @Composable () -> Unit) {
    Box(modifier.size(44.dp)) {
        Box(Modifier.size(44.dp).clip(DsShapes.sm).background(Ds.surface2).dsOutline(Ds.borderStrong).semantics { contentDescription = description }) { image() }
        if (onRemove != null) {
            Box(
                Modifier.align(Alignment.TopEnd).offset(6.dp, (-6).dp).size(20.dp).clip(DsShapes.full).background(Ds.surface).border(Hairline, Ds.borderStrong, DsShapes.full)
                    .clickable(role = Role.Button, onClick = onRemove).semantics { contentDescription = "Remove $description" },
                contentAlignment = Alignment.Center,
            ) { DsIcon(Lucide.X, 12.dp, Ds.text2) }
        }
    }
}

/** The message is too long for one: how long against the limit, and sending it as a file instead. */
@Composable
fun ComposerLimit(count: String, max: String, modifier: Modifier = Modifier, onFile: (() -> Unit)? = null) {
    FlowRow(
        modifier.fillMaxWidth().semantics { liveRegion = LiveRegionMode.Assertive },
        horizontalArrangement = Arrangement.spacedBy(8.dp),
        itemVerticalAlignment = Alignment.CenterVertically,
    ) {
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
            DsIcon(Lucide.Alert, 12.dp, Ds.danger)
            Text("$count / $max", style = Ds.label, color = Ds.danger)
        }
        Text("Too long for one message.", style = Ds.caption, color = Ds.text2, modifier = Modifier.weight(1f))
        if (onFile != null) DsTextAction("Send as a file instead", onFile, color = Ds.text)
    }
}

/** "Copied", in a small inverse chip that pops in. */
@Composable
fun CopiedNote(modifier: Modifier = Modifier, label: String = "Copied") {
    Row(
        modifier.clip(DsShapes.sm).background(Ds.text).padding(horizontal = 8.dp, vertical = 4.dp).semantics { liveRegion = LiveRegionMode.Polite },
        horizontalArrangement = Arrangement.spacedBy(6.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        DsIcon(Lucide.Check, 12.dp, Ds.bg)
        Text(label, style = Ds.caption, color = Ds.bg)
    }
}

/**
 * Editing a sent message in place: the field takes focus at the end, Esc cancels, ⌘↵ saves. It sends as a new
 * version; the old reply stays in [version].
 */
@Composable
fun MessageEditor(value: String, onSave: (String) -> Unit, onCancel: () -> Unit, modifier: Modifier = Modifier, version: String? = null) {
    var text by remember { mutableStateOf(value) }
    val changed = text.isNotBlank() && text.trim() != value.trim()
    val focus = remember { FocusRequester() }
    LaunchedEffect(Unit) { focus.requestFocus() }
    Row(modifier.fillMaxWidth(), horizontalArrangement = Arrangement.End) {
        Column(
            Modifier.widthIn(max = 576.dp).fillMaxWidth().clip(DsShapes.md).background(Ds.surface2).border(Hairline, Ds.accent, DsShapes.md).padding(12.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            DsBareField(
                text, { text = it }, "", Ds.body,
                Modifier.fillMaxWidth().focusRequester(focus).onPreviewKeyEvent { event ->
                    when {
                        event.type != KeyEventType.KeyDown -> false
                        event.key == Key.Escape -> { onCancel(); true }
                        event.key == Key.Enter && (event.isMetaPressed || event.isCtrlPressed) && changed -> { onSave(text.trim()); true }
                        else -> false
                    }
                },
                singleLine = false,
            )
            FlowRow(horizontalArrangement = Arrangement.spacedBy(8.dp), itemVerticalAlignment = Alignment.CenterVertically) {
                Text(
                    if (version != null) "Sends as a new version. The old reply stays in $version." else "Sends as a new version. The old reply stays.",
                    style = Ds.label, color = Ds.text3, modifier = Modifier.footerNote(this),
                )
                DsButton("Cancel", onCancel, variant = DsButtonVariant.Ghost)
                DsButton("Save & send", { onSave(text.trim()) }, iconEnd = Lucide.ArrowUp, enabled = changed)
            }
        }
    }
}

sealed interface RegenerateChoice {
    enum class Style { Again, Shorter, Longer, List }
    data class WithStyle(val style: Style) : RegenerateChoice
    data class WithModel(val model: String) : RegenerateChoice
}

data class RegenerateModel(val name: String, val source: String, val icon: NomiGlyph? = null)

/** How to try again: the same, shorter, longer, as a list, or with another model. */
@Composable
fun RegenerateMenu(onPick: (RegenerateChoice) -> Unit, modifier: Modifier = Modifier, models: List<RegenerateModel> = emptyList(), inline: Boolean = false, trigger: @Composable (open: () -> Unit, expanded: Boolean) -> Unit = { _, _ -> }) {
    val styles = listOf(
        MenuEntry.Item("Try again", Lucide.Refresh, shortcut = "⌘R", onSelect = { onPick(RegenerateChoice.WithStyle(RegenerateChoice.Style.Again)) }),
        MenuEntry.Item("Shorter", Lucide.Minimize, onSelect = { onPick(RegenerateChoice.WithStyle(RegenerateChoice.Style.Shorter)) }),
        MenuEntry.Item("More detail", Lucide.Maximize, onSelect = { onPick(RegenerateChoice.WithStyle(RegenerateChoice.Style.Longer)) }),
        MenuEntry.Item("As a list", Lucide.List, onSelect = { onPick(RegenerateChoice.WithStyle(RegenerateChoice.Style.List)) }),
    )
    val others = if (models.isEmpty()) emptyList() else listOf(MenuEntry.Divider, MenuEntry.Group("With another model")) +
        models.map { MenuEntry.Item(it.name, it.icon, meta = it.source, onSelect = { onPick(RegenerateChoice.WithModel(it.name)) }) }
    DsMenu(styles + others, modifier, wide = true, inline = inline, trigger = trigger)
}

/** Stops the reply being written, with its key (esc). */
@Composable
fun StopHint(onStop: () -> Unit, modifier: Modifier = Modifier) {
    Row(
        modifier.heightIn(min = 32.dp).clip(DsShapes.full).border(Hairline, Ds.borderStrong, DsShapes.full).clickable(role = Role.Button, onClick = onStop).padding(horizontal = 12.dp),
        horizontalArrangement = Arrangement.spacedBy(8.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text("Stop reply", style = Ds.caption, color = Ds.text)
        Text("esc", style = Ds.label, color = Ds.text3)
    }
}

/**
 * Recording a voice note: discard, a live red dot, the time, the input level as bars (the newest still settling,
 * dimmer), the language, and done. Each bar grows to its level.
 */
@Composable
fun VoiceRecorder(elapsed: String, levels: List<Float>, language: String, onCancel: () -> Unit, onDone: () -> Unit, modifier: Modifier = Modifier, fading: Int = 0) {
    Row(
        modifier.fillMaxWidth().heightIn(min = 56.dp).clip(DsShapes.md).background(Ds.surface).dsOutline(Ds.borderStrong, DsShapes.md).padding(horizontal = 6.dp)
            .semantics { contentDescription = "Recording a voice note" },
        horizontalArrangement = Arrangement.spacedBy(8.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        DsIconButton(Lucide.X, "Discard recording", onCancel, variant = DsIconButtonVariant.Ghost)
        Box(Modifier.size(8.dp).alpha(blinkAlpha()).clip(DsShapes.full).background(Ds.danger))
        Text(elapsed, style = Ds.label, color = Ds.text)
        Row(Modifier.weight(1f).height(28.dp), horizontalArrangement = Arrangement.spacedBy(2.dp), verticalAlignment = Alignment.CenterVertically) {
            levels.forEachIndexed { index, level ->
                val height by animateDpAsState((LEVEL_MIN_DP + level.coerceIn(0f, 1f) * LEVEL_RANGE_DP).dp, DsMotion.spatial(), label = "level")
                Box(Modifier.width(3.dp).height(height).clip(DsShapes.full).background(if (index >= levels.size - fading) Ds.borderStrong else Ds.text2))
            }
        }
        Text(language, style = Ds.label, color = Ds.text3)
        DsIconButton(Lucide.Check, "Use recording", onDone, variant = DsIconButtonVariant.Send)
    }
}

/** Under a transcribed message: how long the audio was, and that it can be edited. */
@Composable
fun VoiceTranscribed(duration: String, modifier: Modifier = Modifier) {
    Row(modifier, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
        DsIcon(Lucide.Mic, 12.dp, Ds.text3, Modifier.padding(top = 1.dp))
        Text("transcribed from $duration of audio · edit before sending", style = Ds.label, color = Ds.text3)
    }
}

enum class BannerKind { Offline, Online, Slow, Expiring, Quota }

/** A state the whole chat is in (offline, back online, slow, key expiring, quota), with what to do about it. */
@Composable
fun Banner(kind: BannerKind, text: String, modifier: Modifier = Modifier, actionLabel: String? = null, onAction: (() -> Unit)? = null) {
    val bg by animateColorAsState(
        when (kind) {
            BannerKind.Offline -> Ds.warningBg
            BannerKind.Online -> Ds.accentBg
            else -> Ds.surface
        },
        DsMotion.effects(), label = "banner-bg",
    )
    val icon = when (kind) {
        BannerKind.Offline -> Lucide.WifiOff
        BannerKind.Online -> Lucide.Wifi
        BannerKind.Slow -> Lucide.Activity
        BannerKind.Expiring -> Lucide.KeyRound
        BannerKind.Quota -> Lucide.Gauge
    }
    val tint = when (kind) {
        BannerKind.Online -> Ds.accent
        BannerKind.Quota -> Ds.text2
        else -> Ds.warning
    }
    NoteRow(modifier.semantics { liveRegion = LiveRegionMode.Polite }, bg, outline = kind != BannerKind.Offline && kind != BannerKind.Online) {
        AnimatedContent(icon, label = "banner-icon") { DsIcon(it, 14.dp, tint) }
        Text(text, style = Ds.caption, color = Ds.text, modifier = Modifier.weight(1f))
        if (actionLabel != null && onAction != null) DsTextAction(actionLabel, onAction)
    }
}

enum class ReplyStatus { Done, Streaming, Stopped, Interrupted }

/**
 * A reply's prose: while it streams a caret blinks at the end; stopped says when, with continue; interrupted names
 * why it was cut off, with start over and continue.
 */
@Composable
fun ReplyText(
    text: String,
    modifier: Modifier = Modifier,
    status: ReplyStatus = ReplyStatus.Done,
    elapsed: String = "",
    reason: String = "",
    onContinue: (() -> Unit)? = null,
    onRestart: (() -> Unit)? = null,
) {
    Column(modifier.fillMaxWidth(), verticalArrangement = Arrangement.spacedBy(if (status == ReplyStatus.Interrupted) 10.dp else 8.dp)) {
        Row(verticalAlignment = Alignment.Bottom) {
            Text(text, style = Ds.body, color = Ds.text2, modifier = Modifier.weight(1f, fill = false))
            if (status == ReplyStatus.Streaming) {
                Box(Modifier.padding(start = 2.dp, bottom = 4.dp).width(8.dp).height(16.dp).alpha(blinkAlpha()).background(Ds.text2).semantics { contentDescription = "Writing…" })
            }
        }
        AnimatedVisibility(status == ReplyStatus.Stopped, enter = fadeIn(DsMotion.effects()), exit = fadeOut(DsMotion.fast())) {
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
                DsIcon(Lucide.Square, 12.dp, Ds.text3)
                Text("stopped by you · $elapsed", style = Ds.label, color = Ds.text3)
                if (onContinue != null) DsTextAction("Continue", onContinue)
            }
        }
        AnimatedVisibility(status == ReplyStatus.Interrupted, enter = fadeIn(DsMotion.effects()) + slideInVertically(DsMotion.spatial()) { -it / 2 }, exit = fadeOut(DsMotion.fast()) + slideOutVertically(DsMotion.fast()) { -it / 2 }) {
            FlowRow(
                Modifier.fillMaxWidth().clip(DsShapes.sm).background(Ds.surface).dsOutline(Ds.border).padding(horizontal = 12.dp, vertical = 8.dp),
                horizontalArrangement = Arrangement.spacedBy(10.dp),
                itemVerticalAlignment = Alignment.CenterVertically,
            ) {
                DsIcon(Lucide.Unplug, 14.dp, Ds.warning)
                Text(reason, style = Ds.caption, color = Ds.text, modifier = Modifier.weight(1f).widthIn(min = 160.dp))
                if (onRestart != null) DsButton("Start over", onRestart, variant = DsButtonVariant.Ghost, iconStart = Lucide.RotateCcw)
                if (onContinue != null) DsButton("Continue", onContinue, variant = DsButtonVariant.Secondary, iconEnd = Lucide.ArrowRight)
            }
        }
    }
}

enum class ChatAction { Rename, Pin, Move, Export, Share, Delete }

/** A chat's options: rename, pin, move to a project, export, share, delete. */
@Composable
fun ChatMenu(onPick: (ChatAction) -> Unit, modifier: Modifier = Modifier, pinned: Boolean = false, inline: Boolean = false, trigger: @Composable (open: () -> Unit, expanded: Boolean) -> Unit = { _, _ -> }) {
    DsMenu(
        listOf(
            MenuEntry.Item("Rename chat", Lucide.Pencil, shortcut = "F2", onSelect = { onPick(ChatAction.Rename) }),
            MenuEntry.Item(if (pinned) "Unpin from sidebar" else "Pin to sidebar", if (pinned) Lucide.PinOff else Lucide.Pin, onSelect = { onPick(ChatAction.Pin) }),
            MenuEntry.Item("Move to a project", Lucide.FolderInput, onSelect = { onPick(ChatAction.Move) }),
            MenuEntry.Item("Export", Lucide.Download, meta = "md · pdf", onSelect = { onPick(ChatAction.Export) }),
            MenuEntry.Item("Share whole chat", Lucide.Share, onSelect = { onPick(ChatAction.Share) }),
            MenuEntry.Divider,
            MenuEntry.Item("Delete chat", Lucide.Trash, danger = true, onSelect = { onPick(ChatAction.Delete) }),
        ),
        modifier, wide = true, inline = inline, trigger = trigger,
    )
}

/** A day in the thread: a rule, the date in mono capitals, a rule. */
@Composable
fun DateDivider(label: String, modifier: Modifier = Modifier) = Divider(label.uppercase(), Ds.border, Ds.text3, modifier)

/** Where the replies you missed start, in the accent. */
@Composable
fun NewDivider(count: Int, modifier: Modifier = Modifier) =
    Divider(if (count == 1) "NEW · 1 reply while you were away" else "NEW · $count replies while you were away", Ds.accent, Ds.accent, modifier)

@Composable
private fun Divider(label: String, line: Color, ink: Color, modifier: Modifier) {
    Row(modifier.fillMaxWidth().semantics(mergeDescendants = true) { contentDescription = label }, horizontalArrangement = Arrangement.spacedBy(10.dp), verticalAlignment = Alignment.CenterVertically) {
        Box(Modifier.weight(1f).height(Hairline).background(line))
        Text(label, style = Ds.labelCaps, color = ink)
        Box(Modifier.weight(1f).height(Hairline).background(line))
    }
}

/** Back to the bottom of the thread, saying how many replies came in meanwhile. Slides up when it appears. */
@Composable
fun JumpToLatest(count: Int, onClick: () -> Unit, modifier: Modifier = Modifier) {
    Row(
        modifier.heightIn(min = 36.dp).clip(DsShapes.full).background(Ds.surface).border(Hairline, Ds.borderStrong, DsShapes.full)
            .clickable(role = Role.Button, onClick = onClick).padding(horizontal = 14.dp),
        horizontalArrangement = Arrangement.spacedBy(6.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        DsIcon(Lucide.ArrowDown, 14.dp, Ds.text)
        AnimatedContent(count, label = "jump-count") {
            Text(
                when (it) { 0 -> "Jump to latest"; 1 -> "1 new reply"; else -> "$it new replies" },
                style = Ds.caption.copy(fontWeight = FontWeight.Medium), color = Ds.text,
            )
        }
    }
}

/** Older messages on their way in, at the top of the thread. */
@Composable
fun LoadingOlder(modifier: Modifier = Modifier) {
    Row(modifier.fillMaxWidth().padding(vertical = 8.dp).semantics { liveRegion = LiveRegionMode.Polite }, horizontalArrangement = Arrangement.spacedBy(8.dp, Alignment.CenterHorizontally), verticalAlignment = Alignment.CenterVertically) {
        DsSpinner(12.dp, Ds.text3)
        Text("Loading older messages…", style = Ds.label, color = Ds.text3)
    }
}

data class MoveProject(val id: String, val name: String, val workspace: String, val shared: String? = null)

/** Moving a chat into a project: search them, pick one (a check slides in), and what sharing it means. */
@Composable
fun MoveDialog(open: Boolean, projects: List<MoveProject>, onMove: (String) -> Unit, onDismiss: () -> Unit, inline: Boolean = false, initialChosen: String? = null) {
    var query by remember { mutableStateOf("") }
    var chosen by remember { mutableStateOf(initialChosen) }
    val shown = projects.filter { it.name.contains(query.trim(), ignoreCase = true) }
    val target = projects.firstOrNull { it.id == chosen }
    DsDialog(
        open, onDismiss, "Move chat to a project", wide = true, inline = inline,
        actions = {
            DsButton("Cancel", onDismiss, variant = DsButtonVariant.Secondary)
            DsButton("Move chat", { chosen?.let { onMove(it); onDismiss() } }, iconEnd = Lucide.FolderInput, enabled = chosen != null)
        },
    ) {
        DsSearch(query, { query = it }, "Search projects")
        Column(Modifier.selectableGroup(), verticalArrangement = Arrangement.spacedBy(2.dp)) {
            if (shown.isEmpty()) Text("No projects match", style = Ds.caption, color = Ds.text3, modifier = Modifier.padding(8.dp))
            shown.forEach { project ->
                val on = chosen == project.id
                val bg by animateColorAsState(if (on) Ds.surface2 else Color.Transparent, DsMotion.fast(), label = "move-bg")
                Row(
                    Modifier.fillMaxWidth().heightIn(min = MinTouch).clip(DsShapes.sm).background(bg).selectable(on, role = Role.RadioButton) { chosen = project.id }.padding(horizontal = 8.dp),
                    horizontalArrangement = Arrangement.spacedBy(10.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    DsIcon(Lucide.Folder, 14.dp, Ds.text2)
                    DsText(project.name, Ds.mono, Ds.text, Modifier.weight(1f), singleLine = true)
                    Text(project.workspace, style = Ds.label, color = Ds.text3)
                    AnimatedVisibility(on, enter = scaleIn(DsMotion.spatial()) + fadeIn(DsMotion.fast()), exit = scaleOut(DsMotion.fast()) + fadeOut(DsMotion.fast())) {
                        DsIcon(Lucide.Check, 14.dp, Ds.accent)
                    }
                }
            }
        }
        AnimatedVisibility(target?.shared != null) {
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                DsIcon(Lucide.Users, 12.dp, Ds.text3, Modifier.padding(top = 2.dp))
                Text("Members of ${target?.shared ?: ""} will see this chat. Your memory and money records stay personal.", style = Ds.caption, color = Ds.text3)
            }
        }
    }
}

/** Renaming a chat in place: the name selected, ↵ saves, esc cancels. */
@Composable
fun RenameField(value: String, onSave: (String) -> Unit, onCancel: () -> Unit, modifier: Modifier = Modifier) {
    var name by remember { mutableStateOf(androidx.compose.ui.text.input.TextFieldValue(value, androidx.compose.ui.text.TextRange(0, value.length))) }
    val focus = remember { FocusRequester() }
    LaunchedEffect(Unit) { focus.requestFocus() }
    Row(modifier, horizontalArrangement = Arrangement.spacedBy(10.dp), verticalAlignment = Alignment.CenterVertically) {
        androidx.compose.foundation.text.BasicTextField(
            name, { name = it },
            Modifier.weight(1f).focusRequester(focus).clip(DsShapes.sm).background(Ds.surface).border(Hairline, Ds.accent, DsShapes.sm).padding(horizontal = 8.dp, vertical = 6.dp)
                .onPreviewKeyEvent { event ->
                    when {
                        event.type != KeyEventType.KeyDown -> false
                        event.key == Key.Enter && name.text.isNotBlank() -> { onSave(name.text.trim()); true }
                        event.key == Key.Escape -> { onCancel(); true }
                        else -> false
                    }
                }
                .semantics { contentDescription = "Chat name" },
            singleLine = true,
            textStyle = Ds.ui.copy(color = Ds.text),
            cursorBrush = androidx.compose.ui.graphics.SolidColor(Ds.accent),
        )
        Text("↵ save · esc cancel", style = Ds.label, color = Ds.text3)
    }
}

/** The crew still working in the background: who, and when to expect it. */
@Composable
fun BackgroundNote(modifier: Modifier = Modifier, eta: String? = null) {
    NoteRow(modifier.semantics { liveRegion = LiveRegionMode.Polite }, Ds.surface) {
        DsSpinner(14.dp, Ds.text2)
        Text("You can leave this chat. I'll post the result here and in Inbox.", style = Ds.caption, color = Ds.text2, modifier = Modifier.weight(1f))
        if (eta != null) Text(eta, style = Ds.label, color = Ds.text3)
    }
}

data class Alternative(val agent: Crew, val title: String, val detail: String? = null, val external: Boolean = false, val onClick: () -> Unit = {})

/** Something the crew won't do, why, and who or what can help instead. */
@Composable
fun Boundary(title: String, reason: String, modifier: Modifier = Modifier, alternatives: List<Alternative> = emptyList()) {
    DsCard(
        modifier,
        header = {
            DsIcon(Lucide.Shield, 14.dp, Ds.text2)
            DsCardTitle(title)
        },
    ) {
        Column(Modifier.padding(14.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
            Text(reason, style = Ds.ui, color = Ds.text2)
            alternatives.forEach { alt ->
                Row(
                    Modifier.fillMaxWidth().clip(DsShapes.sm).border(Hairline, Ds.borderStrong, DsShapes.sm).clickable(role = Role.Button, onClick = alt.onClick)
                        .padding(horizontal = 9.dp, vertical = 7.dp),
                    horizontalArrangement = Arrangement.spacedBy(10.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Mascot(alt.agent, 20.dp)
                    Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(1.dp)) {
                        Text(alt.title, style = Ds.ui, color = Ds.text)
                        if (alt.detail != null) {
                            Row(horizontalArrangement = Arrangement.spacedBy(4.dp), verticalAlignment = Alignment.CenterVertically) {
                                Text(alt.detail, style = Ds.label, color = Ds.text3)
                                if (alt.external) DsIcon(Lucide.ArrowUpRight, 12.dp, Ds.text3)
                            }
                        }
                    }
                    DsIcon(Lucide.ChevronRight, 12.dp, Ds.text3)
                }
            }
        }
    }
}

/** A service the crew needs connected: who needs it and why, what it reads, that it's safe, and connect or later. */
@Composable
fun ConnectPrompt(agent: Crew, icon: NomiGlyph, service: String, why: String, reads: String, onConnect: () -> Unit, modifier: Modifier = Modifier, alternative: String? = null, onLater: (() -> Unit)? = null) {
    DsCard(
        modifier,
        header = {
            Box(Modifier.size(28.dp).clip(DsShapes.sm).background(agent.tint), contentAlignment = Alignment.Center) { DsIcon(icon, 14.dp, Ds.text) }
            Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(1.dp)) {
                Text("Connect $service", style = Ds.ui.copy(fontWeight = FontWeight.Medium), color = Ds.text)
                Text(why, style = Ds.label, color = Ds.text3)
            }
        },
        footer = {
            DsText(alternative ?: "", Ds.caption, Ds.text3, Modifier.footerNote(this))
            if (onLater != null) DsButton("Not now", onLater, variant = DsButtonVariant.Ghost)
            DsButton("Connect", onConnect, iconEnd = Lucide.ArrowUpRight)
        },
    ) {
        Column(Modifier.padding(14.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
            ConnectLine(Lucide.Eye, Ds.text2, reads)
            ConnectLine(Lucide.ShieldCheck, Ds.accent, "Nothing is created or changed without asking")
            ConnectLine(Lucide.Unplug, Ds.text2, "Disconnect any time in Settings › Connections")
        }
    }
}

@Composable
private fun ConnectLine(icon: NomiGlyph, tint: Color, text: String) {
    Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
        DsIcon(icon, 13.dp, tint)
        Text(text, style = Ds.caption, color = Ds.text2)
    }
}

/** Who on the crew is working on this chat now (their shapes moving), and stop all. */
@Composable
fun CrewWorking(crew: List<Crew>, modifier: Modifier = Modifier, onStop: (() -> Unit)? = null) {
    Row(
        modifier.fillMaxWidth().clip(DsShapes.sm).dsOutline(Ds.border).padding(start = 12.dp, end = 8.dp, top = 6.dp, bottom = 6.dp).semantics { liveRegion = LiveRegionMode.Polite },
        horizontalArrangement = Arrangement.spacedBy(10.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        MascotStack(crew, working = true)
        AnimatedContent(crew.size, Modifier.weight(1f), label = "crew-count") {
            Text(if (it == 1) "1 crew member working on this chat" else "$it crew working on this chat", style = Ds.caption, color = Ds.text)
        }
        if (onStop != null) DsButton("Stop all", onStop, variant = DsButtonVariant.GhostDanger, iconStart = Lucide.Square)
    }
}

/** One crew member handed the chat to another: their two shapes, why, and asking the first instead. */
@Composable
fun Handoff(from: StepOwner, to: StepOwner, modifier: Modifier = Modifier, reason: String? = null, onBack: (() -> Unit)? = null) {
    NoteRow(modifier, Color.Transparent, outline = true) {
        Mascot(from.agent, 20.dp)
        DsIcon(Lucide.ArrowRight, 12.dp, Ds.text3)
        Mascot(to.agent, 20.dp, working = true)
        Text("${from.name} handed this to ${to.name}", style = Ds.caption, color = Ds.text2)
        Text(reason ?: "", style = Ds.label, color = Ds.text3, modifier = Modifier.weight(1f))
        if (onBack != null) DsTextAction("Ask ${from.name} instead", onBack)
    }
}

/** A fact the crew saved to memory, with edit and undo. */
@Composable
fun MemorySaved(fact: String, modifier: Modifier = Modifier, onEdit: (() -> Unit)? = null, onUndo: (() -> Unit)? = null) {
    NoteRow(modifier.semantics { liveRegion = LiveRegionMode.Polite }, Ds.surface) {
        DsIcon(Lucide.BookmarkPlus, 14.dp, Ds.accent)
        Text("Saved to memory: $fact", style = Ds.caption, color = Ds.text2, modifier = Modifier.weight(1f))
        if (onEdit != null) DsTextAction("Edit", onEdit)
        if (onUndo != null) DsTextAction("Undo", onUndo)
    }
}

/** A fact from memory this reply used, and where to manage memory. */
@Composable
fun MemoryUsed(fact: String, modifier: Modifier = Modifier, onManage: (() -> Unit)? = null) {
    FlowRow(modifier, horizontalArrangement = Arrangement.spacedBy(6.dp), itemVerticalAlignment = Alignment.CenterVertically) {
        Row(horizontalArrangement = Arrangement.spacedBy(6.dp), verticalAlignment = Alignment.CenterVertically) {
            DsIcon(Lucide.Bookmark, 12.dp, Ds.text3)
            Text("From memory: $fact", style = Ds.caption, color = Ds.text3)
        }
        if (onManage != null) DsTextAction("Manage", onManage, underline = true)
    }
}

/** Signed out mid-chat: the message wasn't sent but is kept, and signing in again. */
@Composable
fun SessionExpired(onSignIn: () -> Unit, modifier: Modifier = Modifier, draft: String? = null) {
    DsCard(
        modifier,
        footer = {
            Box(Modifier.footerNote(this))
            DsButton("Sign in again", onSignIn, iconEnd = Lucide.ArrowRight)
        },
    ) {
        Column(Modifier.padding(20.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
            DsIcon(Lucide.LogIn, 18.dp, Ds.text2)
            Text("You've been signed out", style = Ds.body.copy(fontWeight = FontWeight.Medium), color = Ds.text)
            Text(
                "Your session ended, so this message wasn't sent. It's saved on this device and goes out after you sign in. The crew's running tasks keep going.",
                style = Ds.ui, color = Ds.text2,
            )
            if (draft != null) {
                Row(
                    Modifier.fillMaxWidth().clip(DsShapes.sm).background(Ds.surface).padding(horizontal = 10.dp, vertical = 8.dp),
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    DsIcon(Lucide.FilePen, 12.dp, Ds.text3)
                    DsText(draft, Ds.caption, Ds.text2, Modifier.weight(1f), singleLine = true)
                    Text("draft kept", style = Ds.label, color = Ds.text3)
                }
            }
        }
    }
}

enum class TaskState { Done, Stopped, Waiting }
data class StoppedTask(val agent: Crew, val title: String, val state: TaskState, val detail: String? = null)

/** The supervisor stopped the crew: each task and where it got to, discard or resume. */
@Composable
fun SupervisorStopped(tasks: List<StoppedTask>, modifier: Modifier = Modifier, meta: String? = null, onDiscard: (() -> Unit)? = null, onResume: (() -> Unit)? = null) {
    DsCard(
        modifier,
        header = {
            Mascot(Crew.Supervisor, 20.dp)
            DsCardTitle(if (tasks.size == 1) "Supervisor stopped 1 task" else "Supervisor stopped ${tasks.size} tasks")
            if (meta != null) Text(meta, style = Ds.label, color = Ds.text3)
        },
        footer = {
            Text("Nothing that left Nomi was undone.", style = Ds.caption, color = Ds.text3, modifier = Modifier.footerNote(this))
            if (onDiscard != null) DsButton("Discard", onDiscard, variant = DsButtonVariant.Ghost)
            if (onResume != null) DsButton("Resume", onResume, variant = DsButtonVariant.Secondary, iconEnd = Lucide.Play)
        },
    ) {
        Column(Modifier.padding(horizontal = 14.dp, vertical = 4.dp)) {
            tasks.forEachIndexed { index, task ->
                if (index > 0) DsRule()
                Row(Modifier.fillMaxWidth().padding(vertical = 8.dp), horizontalArrangement = Arrangement.spacedBy(10.dp), verticalAlignment = Alignment.CenterVertically) {
                    DsIcon(
                        when (task.state) { TaskState.Done -> Lucide.Check; TaskState.Stopped -> Lucide.Square; TaskState.Waiting -> Lucide.CircleDashed },
                        14.dp, if (task.state == TaskState.Done) Ds.accent else Ds.text3,
                    )
                    Mascot(task.agent, 20.dp)
                    Text(task.title, style = Ds.ui, color = Ds.text, modifier = Modifier.weight(1f))
                    if (task.detail != null) Text(task.detail, style = Ds.label, color = Ds.text3)
                }
            }
        }
    }
}
