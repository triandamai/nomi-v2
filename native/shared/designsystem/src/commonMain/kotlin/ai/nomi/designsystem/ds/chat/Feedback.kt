package ai.nomi.designsystem.ds.chat

import ai.nomi.designsystem.ds.ChipKind
import ai.nomi.designsystem.ds.CheckMark
import ai.nomi.designsystem.ds.CopyGlyph
import ai.nomi.designsystem.ds.Crew
import ai.nomi.designsystem.ds.Ds
import ai.nomi.designsystem.ds.DsBareField
import ai.nomi.designsystem.ds.DsBareIcon
import ai.nomi.designsystem.ds.DsButton
import ai.nomi.designsystem.ds.DsButtonVariant
import ai.nomi.designsystem.ds.DsChip
import ai.nomi.designsystem.ds.DsDialog
import ai.nomi.designsystem.ds.DsIcon
import ai.nomi.designsystem.ds.DsMotion
import ai.nomi.designsystem.ds.DsSegmented
import ai.nomi.designsystem.ds.DsShapes
import ai.nomi.designsystem.ds.DsText
import ai.nomi.designsystem.ds.DsTextAction
import ai.nomi.designsystem.ds.DsToggle
import ai.nomi.designsystem.ds.DsVerticalRule
import ai.nomi.designsystem.ds.Hairline
import ai.nomi.designsystem.ds.Mascot
import ai.nomi.designsystem.ds.MinTouch
import ai.nomi.designsystem.ds.SegmentOption
import ai.nomi.designsystem.ds.SegmentedSurface
import ai.nomi.designsystem.ds.dsOutline
import ai.nomi.designsystem.ds.rememberCopier
import ai.nomi.designsystem.icon.Lucide
import ai.nomi.designsystem.icon.NomiGlyph
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.scaleIn
import androidx.compose.animation.scaleOut
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.selection.toggleable
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.scale
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.selected
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp

private const val PRESSED_POP = 1.12f

enum class ReplyFeedback { Liked, Disliked }

@Composable
private fun ToolButton(icon: NomiGlyph, label: String, onClick: () -> Unit, pressed: Boolean = false, content: (@Composable () -> Unit)? = null) {
    val bg by animateColorAsState(if (pressed) Ds.surface2 else Color.Transparent, DsMotion.fast(), label = "tool-bg")
    val pop by animateFloatAsState(if (pressed) PRESSED_POP else 1f, DsMotion.spatial(), label = "tool-pop")
    Box(
        Modifier.size(MinTouch).clip(DsShapes.sm).background(bg).clickable(role = Role.Button, onClick = onClick)
            .semantics { contentDescription = label; selected = pressed },
        contentAlignment = Alignment.Center,
    ) {
        if (content != null) content() else DsIcon(icon, 14.dp, if (pressed) Ds.text else Ds.text3, Modifier.scale(pop))
    }
}

/**
 * What you can do with a reply: copy (turning to a check), like and dislike (a little pop when pressed), share,
 * regenerate and delete, with what made it at the end ("deepseek-flash · 1.4s · 812 tokens").
 */
@Composable
fun ResponseActions(
    text: String,
    modifier: Modifier = Modifier,
    meta: String? = null,
    feedback: ReplyFeedback? = null,
    onLike: (() -> Unit)? = null,
    onDislike: (() -> Unit)? = null,
    onShare: (() -> Unit)? = null,
    onRegenerate: (() -> Unit)? = null,
    onDelete: (() -> Unit)? = null,
) {
    val copier = rememberCopier()
    Row(modifier.fillMaxWidth().semantics { contentDescription = "Reply actions" }, verticalAlignment = Alignment.CenterVertically) {
        ToolButton(Lucide.Copy, if (copier.copied) "Copied" else "Copy", { copier.copy(text) }) { CopyGlyph(copier.copied, 14.dp, Ds.text3) }
        if (onLike != null) ToolButton(Lucide.ThumbsUp, "Good reply", onLike, feedback == ReplyFeedback.Liked)
        if (onDislike != null) ToolButton(Lucide.ThumbsDown, "Bad reply", onDislike, feedback == ReplyFeedback.Disliked)
        if (onShare != null) ToolButton(Lucide.Share, "Share", onShare)
        if (onRegenerate != null) ToolButton(Lucide.Refresh, "Regenerate", onRegenerate)
        if (onDelete != null) {
            DsVerticalRule(Modifier.padding(horizontal = 2.dp))
            ToolButton(Lucide.Trash, "Delete reply", onDelete)
        }
        if (meta != null) DsText(meta, Ds.label, Ds.text3, Modifier.weight(1f).padding(start = 12.dp), singleLine = true)
    }
}

enum class DislikeReason(val label: String) {
    Inaccurate("Not accurate"), Ignored("Ignored my request"), WrongCrew("Wrong crew member"),
    TooLong("Too long"), Outdated("Out of date"), Unsafe("Unsafe or harmful"), Other("Something else"),
}

/** What went wrong with a reply: pick reasons (two rows), add a note, optionally include the whole chat, then send. */
@Composable
fun DislikeFeedback(
    onSend: (reasons: List<DislikeReason>, note: String, includeChat: Boolean) -> Unit,
    onCancel: () -> Unit,
    modifier: Modifier = Modifier,
    initialReasons: List<DislikeReason> = emptyList(),
    initialNote: String = "",
) {
    val reasons = remember { mutableStateListOf<DislikeReason>().apply { addAll(initialReasons) } }
    var note by remember { mutableStateOf(initialNote) }
    var includeChat by remember { mutableStateOf(false) }
    Column(modifier.fillMaxWidth().clip(DsShapes.md).background(Ds.surface).dsOutline(Ds.borderStrong, DsShapes.md)) {
        Row(Modifier.fillMaxWidth().padding(start = 14.dp, end = 4.dp, top = 4.dp), verticalAlignment = Alignment.CenterVertically) {
            Text("What went wrong?", style = Ds.ui.copy(fontWeight = FontWeight.Medium), color = Ds.text, modifier = Modifier.weight(1f))
            DsBareIcon(Lucide.X, "Close", onCancel)
        }
        Column(Modifier.padding(horizontal = 14.dp, vertical = 8.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
            listOf(DislikeReason.entries.take(3), DislikeReason.entries.drop(3)).forEach { row ->
                FlowRow(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    row.forEach { reason ->
                        val on = reason in reasons
                        val bg by animateColorAsState(if (on) Ds.text else Color.Transparent, DsMotion.fast(), label = "reason-bg")
                        val ink by animateColorAsState(if (on) Ds.bg else Ds.text2, DsMotion.fast(), label = "reason-ink")
                        Row(
                            Modifier.heightIn(min = 32.dp).clip(DsShapes.full).background(bg).border(Hairline, if (on) Ds.text else Ds.borderStrong, DsShapes.full)
                                .toggleable(on, role = Role.Checkbox) { if (it) reasons += reason else reasons -= reason }.padding(horizontal = 11.dp),
                            horizontalArrangement = Arrangement.spacedBy(6.dp),
                            verticalAlignment = Alignment.CenterVertically,
                        ) {
                            AnimatedVisibility(on, enter = scaleIn(DsMotion.spatial()) + fadeIn(DsMotion.fast()), exit = scaleOut(DsMotion.fast()) + fadeOut(DsMotion.fast())) {
                                DsIcon(Lucide.Check, 12.dp, ink)
                            }
                            Text(reason.label, style = Ds.ui, color = ink)
                        }
                    }
                }
            }
        }
        DsBareField(
            note, { note = it }, "", Ds.caption,
            Modifier.padding(start = 14.dp, end = 14.dp, top = 4.dp, bottom = 10.dp).fillMaxWidth().height(72.dp).clip(DsShapes.sm).background(Ds.bg)
                .border(Hairline, Ds.borderStrong, DsShapes.sm).padding(horizontal = 9.dp, vertical = 7.dp),
            singleLine = false,
        )
        Row(
            Modifier.padding(start = 14.dp, end = 14.dp, bottom = 12.dp).toggleable(includeChat, role = Role.Checkbox) { includeChat = it },
            horizontalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            CheckMark(includeChat, null, Modifier.padding(top = 1.dp))
            Column(verticalArrangement = Arrangement.spacedBy(2.dp)) {
                Text("Include the whole chat", style = Ds.caption, color = Ds.text)
                Text("Off by default. Only this reply and your note are sent; amounts and names stay masked.", style = Ds.caption, color = Ds.text3)
            }
        }
        HorizontalDivider(thickness = Hairline, color = Ds.border)
        Row(Modifier.fillMaxWidth().padding(horizontal = 14.dp, vertical = 9.dp), horizontalArrangement = Arrangement.spacedBy(8.dp, Alignment.End)) {
            DsButton("Cancel", onCancel, variant = DsButtonVariant.Ghost)
            DsButton("Send feedback", { onSend(reasons.toList(), note.trim(), includeChat) }, enabled = reasons.isNotEmpty() || note.isNotBlank())
        }
    }
}

/** Thanks, said quietly, with what was sent; a retry when sending failed. */
@Composable
fun FeedbackSent(modifier: Modifier = Modifier, withChat: Boolean = false, retryLabel: String? = null, onRetry: (() -> Unit)? = null) {
    NoteRow(modifier, Ds.surface) {
        DsIcon(Lucide.Check, 14.dp, Ds.accent)
        Text(
            if (withChat) "Thanks. Feedback sent with the whole chat." else "Thanks. Feedback sent with this reply only.",
            style = Ds.caption, color = Ds.text2, modifier = Modifier.weight(1f),
        )
        if (retryLabel != null && onRetry != null) DsTextAction(retryLabel, onRetry, color = Ds.text)
    }
}

/** Where a reply was: deleted, with undo and regenerate. */
@Composable
fun DeletedReply(name: String, modifier: Modifier = Modifier, onUndo: (() -> Unit)? = null, onRegenerate: (() -> Unit)? = null) {
    NoteRow(modifier, Color.Transparent, outline = true) {
        DsIcon(Lucide.Trash, 14.dp, Ds.text3)
        Text("You deleted $name's reply", style = Ds.caption.copy(fontStyle = FontStyle.Italic), color = Ds.text3, modifier = Modifier.weight(1f))
        if (onUndo != null) DsTextAction("Undo", onUndo)
        if (onRegenerate != null) DsTextAction("Regenerate", onRegenerate)
    }
}

/** Asks before a reply and its cards leave the chat. */
@Composable
fun DeleteReplyDialog(open: Boolean, name: String, onDelete: () -> Unit, onDismiss: () -> Unit, inline: Boolean = false) {
    DsDialog(
        open, onDismiss, "Delete this reply?",
        description = "$name's reply and its cards leave this chat. Your message stays, so you can ask again. Things the crew already did, like a sent email, aren't undone.",
        inline = inline,
        actions = {
            DsButton("Cancel", onDismiss, variant = DsButtonVariant.Secondary)
            DsButton("Delete reply", { onDelete(); onDismiss() }, variant = DsButtonVariant.Danger, iconStart = Lucide.Trash)
        },
    )
}

/** A quiet one-line note in the thread: an icon, words, and small actions on the right. */
@Composable
internal fun NoteRow(modifier: Modifier, background: Color, outline: Boolean = false, content: @Composable androidx.compose.foundation.layout.FlowRowScope.() -> Unit) {
    FlowRow(
        modifier.fillMaxWidth().clip(DsShapes.sm).background(background).then(if (outline) Modifier.dsOutline(Ds.border) else Modifier)
            .padding(horizontal = 12.dp, vertical = 8.dp),
        horizontalArrangement = Arrangement.spacedBy(8.dp),
        verticalArrangement = Arrangement.spacedBy(4.dp),
        itemVerticalAlignment = Alignment.CenterVertically,
        content = content,
    )
}

enum class ShareScope { Reply, Chat }
data class ShareSettings(val scope: ShareScope = ShareScope.Reply, val mask: Boolean = true, val anyone: Boolean = true)
data class SharePreview(val agent: Crew, val name: String, val time: String, val excerpt: String)

/** What to share and how: this reply or the whole chat, a preview, masking, who can open it, expiry, and the link. */
@Composable
fun SharePanel(
    preview: SharePreview,
    chatMessages: Int,
    url: String,
    expiry: String,
    onClose: () -> Unit,
    modifier: Modifier = Modifier,
    settings: ShareSettings = ShareSettings(),
    onChange: ((ShareSettings) -> Unit)? = null,
    onExpiry: (() -> Unit)? = null,
    onCopy: (() -> Unit)? = null,
) {
    var current by remember { mutableStateOf(settings) }
    val copier = rememberCopier()
    fun update(next: ShareSettings) {
        current = next
        onChange?.invoke(next)
    }
    Column(modifier.fillMaxWidth()) {
        Column(Modifier.padding(20.dp), verticalArrangement = Arrangement.spacedBy(16.dp)) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Text("Share", style = Ds.body.copy(fontWeight = FontWeight.Medium), color = Ds.text, modifier = Modifier.weight(1f))
                DsBareIcon(Lucide.X, "Close", onClose)
            }
            DsSegmented(
                "What to share",
                listOf(SegmentOption("reply", "This reply"), SegmentOption("chat", "Whole chat", "$chatMessages msgs")),
                if (current.scope == ShareScope.Reply) "reply" else "chat",
                { update(current.copy(scope = if (it == "reply") ShareScope.Reply else ShareScope.Chat)) },
                surface = SegmentedSurface.Bg,
            )
            Column(
                Modifier.fillMaxWidth().clip(DsShapes.sm).background(Ds.bg).dsOutline(Ds.border).padding(12.dp),
                verticalArrangement = Arrangement.spacedBy(6.dp),
            ) {
                Row(horizontalArrangement = Arrangement.spacedBy(6.dp), verticalAlignment = Alignment.CenterVertically) {
                    Mascot(preview.agent, 20.dp)
                    Text(preview.name, style = Ds.caption.copy(fontWeight = FontWeight.Medium), color = Ds.text)
                    Text(preview.time, style = Ds.label, color = Ds.text3)
                }
                Text(preview.excerpt, style = Ds.caption, color = Ds.text2)
            }
            Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                ToggleLine("Hide amounts and names", "Shows Rp ••• and initials. Your records never leave Nomi.", current.mask) { update(current.copy(mask = it)) }
                ToggleLine("Anyone with the link can view", "Read-only. No sign-in, no replies.", current.anyone) { update(current.copy(anyone = it)) }
                Row(verticalAlignment = Alignment.CenterVertically) {
                    Text("Link expires", style = Ds.ui, color = Ds.text, modifier = Modifier.weight(1f))
                    DsChip(ChipKind.Option, expiry, icon = Lucide.Hourglass, onClick = onExpiry)
                }
            }
            FlowRow(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalArrangement = Arrangement.spacedBy(8.dp), itemVerticalAlignment = Alignment.CenterVertically) {
                Row(
                    Modifier.weight(1f).height(40.dp).clip(DsShapes.sm).background(Ds.bg).dsOutline(Ds.borderStrong).padding(horizontal = 10.dp),
                    horizontalArrangement = Arrangement.spacedBy(6.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    DsIcon(Lucide.Link, 12.dp, Ds.text3)
                    DsText(url.removePrefix("https://").removePrefix("http://"), Ds.mono, Ds.text, singleLine = true)
                }
                DsButton("Copy link", { copier.copy(url); onCopy?.invoke() }, iconEnd = if (copier.copied) Lucide.Check else Lucide.Copy)
            }
        }
        HorizontalDivider(thickness = Hairline, color = Ds.border)
        Text("You can revoke this link any time in Settings › Data", style = Ds.caption, color = Ds.text3, modifier = Modifier.padding(horizontal = 20.dp, vertical = 12.dp))
    }
}

@Composable
private fun ToggleLine(title: String, help: String, checked: Boolean, onChange: (Boolean) -> Unit) {
    Row(horizontalArrangement = Arrangement.spacedBy(12.dp), verticalAlignment = Alignment.CenterVertically) {
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
            Text(title, style = Ds.ui, color = Ds.text)
            Text(help, style = Ds.caption, color = Ds.text3)
        }
        DsToggle(title, checked, onChange)
    }
}

/** The share panel in a dialog (or in place, [inline], as the design sheet shows it). */
@Composable
fun ShareDialog(
    open: Boolean,
    onDismiss: () -> Unit,
    preview: SharePreview,
    chatMessages: Int,
    url: String,
    expiry: String,
    inline: Boolean = false,
) {
    val panel: @Composable () -> Unit = {
        Box(Modifier.clip(DsShapes.md).background(Ds.surface).dsOutline(Ds.borderStrong, DsShapes.md)) {
            SharePanel(preview, chatMessages, url, expiry, onDismiss)
        }
    }
    if (inline) {
        panel()
        return
    }
    if (!open) return
    androidx.compose.ui.window.Dialog(onDismissRequest = onDismiss) { DsArrivePanel { panel() } }
}

@Composable
private fun DsArrivePanel(content: @Composable () -> Unit) = ai.nomi.designsystem.ds.DsArrive { content() }
