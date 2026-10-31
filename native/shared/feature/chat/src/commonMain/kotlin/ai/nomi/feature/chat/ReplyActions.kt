package ai.nomi.feature.chat

import ai.nomi.designsystem.component.NomiButton
import ai.nomi.designsystem.component.NomiButtonSize
import ai.nomi.designsystem.component.NomiButtonVariant
import ai.nomi.designsystem.icon.NomiGlyph
import ai.nomi.designsystem.icon.NomiIcon
import ai.nomi.designsystem.icon.NomiIcons
import ai.nomi.designsystem.theme.NomiTheme
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Checkbox
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
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
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.selected
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.delay

const val RATING_UP = "up"
const val RATING_DOWN = "down"

/** What went wrong with a reply (`DislikeFeedback.svelte`), in the web's two rows. */
enum class DislikeReason(val wire: String, val label: String) {
    Inaccurate("inaccurate", "Not accurate"),
    Ignored("ignored", "Didn't do what I asked"),
    WrongCrew("wrong_crew", "Wrong crew member"),
    TooLong("too_long", "Too long"),
    Outdated("outdated", "Out of date"),
    Unsafe("unsafe", "Unsafe or harmful"),
    Other("other", "Other"),
}

/** How a reply is asked for again (`RegenerateMenu.svelte`). */
enum class RegenerateStyle(val wire: String, val label: String) {
    Again("again", "Try again"),
    Shorter("shorter", "Shorter"),
    Longer("longer", "Longer"),
    List("list", "As a list"),
}

/** A thumbs-down's details, as the form sends them. */
data class Dislike(val reasons: List<DislikeReason>, val note: String, val includeChat: Boolean)

/**
 * Where the feedback on one reply stands, beyond its stored rating: the "what went wrong" form open after a
 * thumbs-down, then the thanks ([sentWithChat]: whether the whole chat went with it).
 */
data class ReplyFeedbackState(val dislikeOpen: Boolean = false, val sentWithChat: Boolean? = null, val sending: Boolean = false)

/** What a reply's actions do; the defaults do nothing (screenshots, read-only threads). */
class ReplyHandlers(
    val rate: (messageId: String, rating: String?) -> Unit = { _, _ -> },
    val sendDislike: (messageId: String, Dislike) -> Unit = { _, _ -> },
    val closeDislike: (messageId: String) -> Unit = {},
    val delete: (messageId: String) -> Unit = {},
    val regenerate: (messageId: String, RegenerateStyle) -> Unit = { _, _ -> },
    /** Shares a link to the reply; calls back true once the link went to the share sheet or clipboard. */
    val share: (messageId: String, done: (Boolean) -> Unit) -> Unit = { _, _ -> },
)

private const val COPIED_RESET_MS = 2000L
private val ActionButton = 32.dp
private val ActionGlyph = 16.dp
private val ActionGap = 2.dp
private val SeparatorWidth = 1.dp
private val SeparatorHeight = 14.dp
private val SeparatorPadding = 4.dp

/**
 * `ResponseActions.svelte`: the quiet bar under every reply. Copy; like and dislike as pressed toggles (a like in the
 * primary colour); share; regenerate on the latest reply; delete set apart. When it was sent sits on the right.
 */
@Composable
fun ResponseActions(
    text: String,
    time: String?,
    feedback: String?,
    onRate: (String?) -> Unit,
    onShare: ((done: (Boolean) -> Unit) -> Unit)?,
    onRegenerate: (() -> Unit)?,
    onDelete: (() -> Unit)?,
    modifier: Modifier = Modifier,
) {
    val colors = MaterialTheme.colorScheme
    val clipboard = LocalClipboardManager.current
    var copied by remember { mutableStateOf(false) }
    var shared by remember { mutableStateOf(false) }
    LaunchedEffect(copied, shared) {
        if (copied || shared) {
            delay(COPIED_RESET_MS)
            copied = false
            shared = false
        }
    }
    Row(modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(ActionGap)) {
        ActionButton(if (copied) NomiIcons.Check else NomiIcons.Copy, if (copied) "Copied" else "Copy", active = copied) {
            clipboard.setText(AnnotatedString(text))
            copied = true
        }
        ActionButton(NomiIcons.ThumbsUp, "Good reply", pressed = feedback == RATING_UP, active = feedback == RATING_UP) {
            onRate(if (feedback == RATING_UP) null else RATING_UP)
        }
        ActionButton(NomiIcons.ThumbsDown, "Bad reply", pressed = feedback == RATING_DOWN) {
            onRate(if (feedback == RATING_DOWN) null else RATING_DOWN)
        }
        if (onShare != null) {
            ActionButton(if (shared) NomiIcons.Check else NomiIcons.Share, if (shared) "Link ready" else "Share", active = shared) {
                onShare { ok -> shared = ok }
            }
        }
        if (onRegenerate != null) ActionButton(NomiIcons.Refresh, "Regenerate", onClick = onRegenerate)
        if (onDelete != null) {
            Box(Modifier.padding(horizontal = SeparatorPadding).width(SeparatorWidth).height(SeparatorHeight).background(colors.outlineVariant))
            ActionButton(NomiIcons.Delete, "Delete reply", onClick = onDelete)
        }
        if (time != null) {
            Spacer(Modifier.weight(1f))
            Text(time, style = NomiTheme.metaStyle, color = colors.onSurfaceVariant)
        }
    }
}

@Composable
private fun ActionButton(glyph: NomiGlyph, label: String, pressed: Boolean = false, active: Boolean = false, onClick: () -> Unit) {
    val colors = MaterialTheme.colorScheme
    IconButton(
        onClick = onClick,
        modifier = Modifier
            .size(ActionButton)
            .clip(MaterialTheme.shapes.small)
            .background(if (pressed) colors.surfaceContainerHigh else colors.surface.copy(alpha = 0f))
            .semantics { if (pressed) selected = true },
    ) {
        NomiIcon(
            glyph,
            contentDescription = label,
            size = ActionGlyph,
            tint = when {
                active -> colors.primary
                pressed -> colors.onSurface
                else -> colors.onSurfaceVariant
            },
        )
    }
}

private val CardPadding = 14.dp
private val CardGap = 10.dp
private val ChipGap = 8.dp
private val ChipHeight = 32.dp
private val ChipPaddingH = 12.dp
private val ChipCheck = 12.dp
private val NoteHeight = 72.dp
private val NotePadding = 10.dp
private val Hairline = 1.dp
private const val TITLE_WEIGHT = 600
private const val FIRST_ROW = 3

/**
 * `DislikeFeedback.svelte`: what went wrong first (any number of reasons), then an optional note, and whether the
 * whole chat goes with it (off by default).
 */
@OptIn(ExperimentalLayoutApi::class)
@Composable
fun DislikeForm(sending: Boolean, onSend: (Dislike) -> Unit, onCancel: () -> Unit, modifier: Modifier = Modifier) {
    val colors = MaterialTheme.colorScheme
    var picked by rememberSaveable { mutableStateOf(emptyList<DislikeReason>()) }
    var note by rememberSaveable { mutableStateOf("") }
    var includeChat by rememberSaveable { mutableStateOf(false) }
    Column(
        modifier
            .fillMaxWidth()
            .clip(MaterialTheme.shapes.medium)
            .background(colors.surfaceContainerLowest)
            .border(Hairline, colors.outlineVariant, MaterialTheme.shapes.medium),
    ) {
        Row(Modifier.padding(start = CardPadding, end = ActionGap), verticalAlignment = Alignment.CenterVertically) {
            Text(
                "What went wrong?",
                style = MaterialTheme.typography.titleSmall.copy(fontWeight = FontWeight(TITLE_WEIGHT)),
                color = colors.onSurface,
                modifier = Modifier.weight(1f),
            )
            IconButton(onClick = onCancel) { NomiIcon(NomiIcons.Close, "Close", size = ActionGlyph, tint = colors.onSurfaceVariant) }
        }
        Column(Modifier.padding(horizontal = CardPadding), verticalArrangement = Arrangement.spacedBy(ChipGap)) {
            listOf(DislikeReason.entries.take(FIRST_ROW), DislikeReason.entries.drop(FIRST_ROW)).forEach { row ->
                FlowRow(horizontalArrangement = Arrangement.spacedBy(ChipGap), verticalArrangement = Arrangement.spacedBy(ChipGap)) {
                    row.forEach { reason ->
                        val on = reason in picked
                        ReasonChip(reason.label, on) {
                            picked = if (on) picked - reason else DislikeReason.entries.filter { it == reason || it in picked }
                        }
                    }
                }
            }
        }
        BasicTextField(
            value = note,
            onValueChange = { note = it },
            textStyle = MaterialTheme.typography.bodyMedium.copy(color = colors.onSurface),
            cursorBrush = SolidColor(colors.primary),
            modifier = Modifier
                .padding(start = CardPadding, end = CardPadding, top = CardGap)
                .fillMaxWidth()
                .height(NoteHeight)
                .clip(MaterialTheme.shapes.small)
                .background(colors.surface)
                .border(Hairline, colors.outlineVariant, MaterialTheme.shapes.small)
                .padding(NotePadding)
                .semantics { this.contentDescription = "What went wrong?" },
            decorationBox = { field ->
                if (note.isEmpty()) Text("Add a note (optional)", style = MaterialTheme.typography.bodyMedium, color = colors.onSurfaceVariant)
                field()
            },
        )
        Row(
            Modifier
                .padding(horizontal = CardPadding / 2, vertical = ChipGap)
                .clickable(role = Role.Checkbox) { includeChat = !includeChat },
            verticalAlignment = Alignment.Top,
        ) {
            Checkbox(checked = includeChat, onCheckedChange = { includeChat = it })
            Column(Modifier.padding(top = CardGap)) {
                Text("Include the whole chat", style = MaterialTheme.typography.bodyMedium, color = colors.onSurface)
                Text(
                    "Off by default. Only this reply and your note are sent; amounts and names stay masked.",
                    style = MaterialTheme.typography.bodySmall,
                    color = colors.onSurfaceVariant,
                )
            }
        }
        HorizontalDivider(color = colors.outlineVariant)
        Row(
            Modifier.fillMaxWidth().padding(horizontal = CardPadding, vertical = CardGap),
            horizontalArrangement = Arrangement.spacedBy(ChipGap, Alignment.End),
        ) {
            NomiButton(onClick = onCancel, variant = NomiButtonVariant.Text) { Text("Cancel") }
            NomiButton(
                onClick = { onSend(Dislike(picked, note.trim(), includeChat)) },
                enabled = !sending && (picked.isNotEmpty() || note.isNotBlank()),
            ) { Text("Send feedback") }
        }
    }
}

@Composable
private fun ReasonChip(label: String, on: Boolean, onClick: () -> Unit) {
    val colors = MaterialTheme.colorScheme
    val shape = RoundedCornerShape(percent = 50)
    Row(
        Modifier
            .heightIn(min = ChipHeight)
            .clip(shape)
            .background(if (on) colors.onSurface else colors.surface.copy(alpha = 0f))
            .border(BorderStroke(Hairline, if (on) colors.onSurface else colors.outline), shape)
            .clickable(role = Role.Checkbox, onClick = onClick)
            .semantics { selected = on }
            .padding(horizontal = ChipPaddingH),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(ChipGap / 2),
    ) {
        if (on) NomiIcon(NomiIcons.Check, null, size = ChipCheck, tint = colors.surface)
        Text(label, style = MaterialTheme.typography.labelLarge, color = if (on) colors.surface else colors.onSurfaceVariant)
    }
}

/** `FeedbackSent.svelte`: thanks, and exactly what went with it. */
@Composable
fun FeedbackSent(withChat: Boolean, modifier: Modifier = Modifier) {
    val colors = MaterialTheme.colorScheme
    Row(
        modifier.fillMaxWidth().clip(MaterialTheme.shapes.small).background(colors.surfaceContainer).padding(horizontal = CardPadding, vertical = CardGap),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(ChipGap),
    ) {
        NomiIcon(NomiIcons.Check, null, size = ActionGlyph, tint = colors.primary)
        Text(
            if (withChat) "Thanks. Feedback sent with the whole chat." else "Thanks. Feedback sent with this reply only.",
            style = MaterialTheme.typography.bodySmall,
            color = colors.onSurfaceVariant,
        )
    }
}

/** `DeleteReplyDialog.svelte`: what leaves (the reply and its cards), what stays, and what can't be undone. */
@Composable
fun DeleteReplyDialog(name: String, onDelete: () -> Unit, onDismiss: () -> Unit) {
    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text("Delete this reply?") },
        text = {
            Text(
                "$name's reply and its cards leave this chat. Your message stays, so you can ask again. " +
                    "Things the crew already did, like a sent email, are not undone.",
            )
        },
        confirmButton = {
            NomiButton(
                onClick = onDelete,
                colors = androidx.compose.material3.ButtonDefaults.buttonColors(
                    containerColor = MaterialTheme.colorScheme.error,
                    contentColor = MaterialTheme.colorScheme.onError,
                ),
            ) {
                NomiIcon(NomiIcons.Delete, null, size = ActionGlyph)
                Text("Delete reply")
            }
        },
        dismissButton = { NomiButton(onClick = onDismiss, variant = NomiButtonVariant.Text) { Text("Cancel") } },
    )
}

/** `RegenerateMenu.svelte`: the same again, or shorter, longer, or as a list. */
@OptIn(ExperimentalLayoutApi::class)
@Composable
fun RegenerateChoices(onPick: (RegenerateStyle) -> Unit, modifier: Modifier = Modifier) {
    FlowRow(modifier, horizontalArrangement = Arrangement.spacedBy(ChipGap), verticalArrangement = Arrangement.spacedBy(ChipGap)) {
        RegenerateStyle.entries.forEach { style ->
            NomiButton(onClick = { onPick(style) }, variant = NomiButtonVariant.Outlined, size = NomiButtonSize.ExtraSmall) { Text(style.label) }
        }
    }
}
