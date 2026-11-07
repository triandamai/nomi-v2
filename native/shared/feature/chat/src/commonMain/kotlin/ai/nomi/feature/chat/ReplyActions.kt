package ai.nomi.feature.chat

import ai.nomi.designsystem.ds.DsButton
import ai.nomi.designsystem.ds.DsButtonVariant
import ai.nomi.designsystem.ds.chat.ReplyFeedback
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import ai.nomi.designsystem.ds.chat.DeleteReplyDialog as DsDeleteReplyDialog
import ai.nomi.designsystem.ds.chat.DislikeFeedback as DsDislikeFeedback
import ai.nomi.designsystem.ds.chat.FeedbackSent as DsFeedbackSent
import ai.nomi.designsystem.ds.chat.ResponseActions as DsResponseActions
import ai.nomi.designsystem.icon.Lucide

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

private val ChoiceGap = 8.dp

/**
 * `ResponseActions.svelte`: the quiet bar under every reply, drawn by the design system's ResponseActions. Copy; like
 * and dislike as pressed toggles; share; regenerate on the latest reply; delete set apart; when it was sent on the
 * right.
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
    DsResponseActions(
        text = text,
        modifier = modifier,
        meta = time,
        feedback = when (feedback) {
            RATING_UP -> ReplyFeedback.Liked
            RATING_DOWN -> ReplyFeedback.Disliked
            else -> null
        },
        onLike = { onRate(if (feedback == RATING_UP) null else RATING_UP) },
        onDislike = { onRate(if (feedback == RATING_DOWN) null else RATING_DOWN) },
        onShare = onShare?.let { share -> { share {} } },
        onRegenerate = onRegenerate,
        onDelete = onDelete,
    )
}

/**
 * `DislikeFeedback.svelte`: what went wrong first (any number of reasons), then an optional note, and whether the
 * whole chat goes with it (off by default).
 */
@Composable
fun DislikeForm(sending: Boolean, onSend: (Dislike) -> Unit, onCancel: () -> Unit, modifier: Modifier = Modifier) {
    DsDislikeFeedback(
        onSend = { reasons, note, includeChat -> onSend(Dislike(reasons.map { DislikeReason.entries[it.ordinal] }, note, includeChat)) },
        onCancel = onCancel,
        modifier = modifier,
        sending = sending,
    )
}

/** `FeedbackSent.svelte`: thanks, and exactly what went with it. */
@Composable
fun FeedbackSent(withChat: Boolean, modifier: Modifier = Modifier) = DsFeedbackSent(modifier, withChat = withChat)

/** `DeleteReplyDialog.svelte`: what leaves (the reply and its cards), what stays, and what can't be undone. */
@Composable
fun DeleteReplyDialog(name: String, onDelete: () -> Unit, onDismiss: () -> Unit) = DsDeleteReplyDialog(true, name, onDelete, onDismiss)

/** `RegenerateMenu.svelte`: the same again, or shorter, longer, or as a list. */
@Composable
fun RegenerateChoices(onPick: (RegenerateStyle) -> Unit, modifier: Modifier = Modifier) {
    FlowRow(modifier, horizontalArrangement = Arrangement.spacedBy(ChoiceGap), verticalArrangement = Arrangement.spacedBy(ChoiceGap)) {
        RegenerateStyle.entries.forEach { style ->
            val icon = when (style) {
                RegenerateStyle.Again -> Lucide.Refresh
                RegenerateStyle.Shorter -> Lucide.Minimize
                RegenerateStyle.Longer -> Lucide.Maximize
                RegenerateStyle.List -> Lucide.List
            }
            DsButton(style.label, { onPick(style) }, variant = DsButtonVariant.Secondary, iconStart = icon)
        }
    }
}
