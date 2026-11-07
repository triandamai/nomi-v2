package ai.nomi.designsystem.ds

import ai.nomi.designsystem.icon.Lucide
import ai.nomi.designsystem.icon.NomiGlyph
import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.MutableTransitionState
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.semantics.LiveRegionMode
import androidx.compose.ui.semantics.liveRegion
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.delay

private const val COPIED_MS = 1500L
private const val UNSENT_ALPHA = 0.85f

/** Copies text and says "Copied" for a moment (web createCopier). */
class Copier internal constructor(private val copyText: (String) -> Unit, private val flag: androidx.compose.runtime.MutableState<Boolean>) {
    val copied: Boolean get() = flag.value
    fun copy(text: String) {
        copyText(text)
        flag.value = true
    }
}

@Suppress("DEPRECATION")
@Composable
fun rememberCopier(): Copier {
    val clipboard = LocalClipboardManager.current
    val flag = remember { androidx.compose.runtime.mutableStateOf(false) }
    if (flag.value) {
        androidx.compose.runtime.LaunchedEffect(Unit) {
            delay(COPIED_MS)
            flag.value = false
        }
    }
    return remember(clipboard) { Copier({ clipboard.setText(AnnotatedString(it)) }, flag) }
}

/** The copy icon that turns into a check once copied (cross-fading). */
@Composable
fun CopyGlyph(copied: Boolean, size: androidx.compose.ui.unit.Dp, tint: androidx.compose.ui.graphics.Color) {
    AnimatedContent(copied, transitionSpec = { fadeIn(DsMotionSpecs.fast) togetherWith fadeOut(DsMotionSpecs.fast) }, label = "copy-glyph") { done ->
        DsIcon(if (done) Lucide.Check else Lucide.Copy, size, tint)
    }
}

internal object DsMotionSpecs {
    val fast = androidx.compose.animation.core.tween<Float>(ai.nomi.designsystem.generated.NomiMotionTokens.effectsFast.durationMs)
}

/** Fades and rises into place the first time it's composed: new messages and cards arriving in a thread. */
@Composable
fun DsArrive(modifier: Modifier = Modifier, content: @Composable () -> Unit) {
    val state = remember { MutableTransitionState(false) }.apply { targetState = true }
    AnimatedVisibility(state, modifier, enter = fadeIn(DsMotion.effects()) + slideInVertically(DsMotion.spatial()) { it / ARRIVE_FRACTION }) { content() }
}

private const val ARRIVE_FRACTION = 4

data class ActivityFile(val path: String, val change: String? = null)

enum class ActivityState { Done, Working, Failed }

/** What an agent did, summed up ("Wrote 3 files · +424"), with the real files under it. */
@Composable
fun ActivityBlock(title: String, files: List<ActivityFile>, modifier: Modifier = Modifier, total: String? = null, state: ActivityState = ActivityState.Done) {
    DsCard(
        modifier,
        strong = false,
        background = Ds.bg,
        header = {
            when (state) {
                ActivityState.Done -> DsIcon(Lucide.Check, 14.dp, Ds.accent)
                ActivityState.Failed -> DsIcon(Lucide.Alert, 14.dp, Ds.danger)
                ActivityState.Working -> DsSpinner(14.dp, Ds.text2)
            }
            DsText(title, Ds.caption, Ds.text, Modifier.weight(1f), singleLine = true)
            if (total != null) Text(total, style = Ds.label, color = Ds.text2)
        },
    ) {
        Column(Modifier.padding(vertical = 2.dp)) { files.forEach { FileRow(it.path, change = it.change) } }
    }
}

data class ErrorDetail(val key: String, val value: String, val strong: Boolean = false)
data class ErrorAction(val label: String, val icon: NomiGlyph? = null, val onClick: () -> Unit)

/**
 * A failure, named plainly, with the provider's own words quoted in mono (code, request id, reason) and the next
 * move one press away: the first action is the main one (inverse), any other is secondary.
 */
@Composable
fun ErrorCallout(
    title: String,
    modifier: Modifier = Modifier,
    code: String? = null,
    requestId: String? = null,
    details: List<ErrorDetail> = emptyList(),
    actions: List<ErrorAction> = emptyList(),
) {
    Column(
        modifier.fillMaxWidth().clip(DsShapes.md).dsOutline(Ds.borderStrong, DsShapes.md).semantics { liveRegion = LiveRegionMode.Assertive },
    ) {
        FlowRow(
            Modifier.fillMaxWidth().background(Ds.dangerBg).padding(start = 14.dp, end = 14.dp, top = 10.dp, bottom = 9.dp),
            horizontalArrangement = Arrangement.spacedBy(8.dp),
            verticalArrangement = Arrangement.spacedBy(2.dp),
            itemVerticalAlignment = Alignment.CenterVertically,
        ) {
            DsIcon(Lucide.Alert, 14.dp, Ds.danger)
            DsText(title, Ds.ui, Ds.danger, weight = FontWeight.Medium)
            if (code != null) DsText(code, Ds.label, Ds.text3, singleLine = true)
            if (requestId != null) Text(requestId, style = Ds.label, color = Ds.text3)
        }
        HorizontalDivider(thickness = Hairline, color = Ds.border)
        if (details.isNotEmpty()) {
            Column(Modifier.fillMaxWidth().background(Ds.surface).padding(14.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                details.forEach { detail ->
                    Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                        Text(detail.key, style = Ds.label, color = Ds.text3, modifier = Modifier.width(96.dp))
                        Text(detail.value, style = Ds.label, color = if (detail.strong) Ds.text else Ds.text2, modifier = Modifier.weight(1f))
                    }
                }
            }
        }
        if (actions.isNotEmpty()) {
            HorizontalDivider(thickness = Hairline, color = Ds.border)
            FlowRow(
                Modifier.fillMaxWidth().background(Ds.surface).padding(CardFooterPadding),
                horizontalArrangement = Arrangement.spacedBy(8.dp),
                verticalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                actions.forEachIndexed { index, action ->
                    DsButton(action.label, action.onClick, variant = if (index == 0) DsButtonVariant.Inverse else DsButtonVariant.Secondary, iconEnd = action.icon)
                }
            }
        }
    }
}

enum class EventKind(val glyph: NomiGlyph) { Deploy(Lucide.Rocket), Rollback(Lucide.RotateCcw), Domain(Lucide.Globe), Database(Lucide.Database) }

/** Something that happened to a project, logged in mono with its time ("deploy #14 · main@a3f9c21 · 15:38"). */
@Composable
fun EventRow(kind: EventKind, text: String, time: String, modifier: Modifier = Modifier) {
    Row(
        modifier.fillMaxWidth().clip(DsShapes.sm).background(Ds.surface).padding(horizontal = 12.dp, vertical = 8.dp),
        horizontalArrangement = Arrangement.spacedBy(8.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        DsIcon(kind.glyph, 14.dp, Ds.text2)
        DsText(text, Ds.label, Ds.text2, Modifier.weight(1f), singleLine = true)
        Text(time, style = Ds.label, color = Ds.text3)
    }
}

enum class FileState { Done, Writing, Queued }

/**
 * A file an agent touched, with its change ("+214"), in mono. While a build writes files it says where each one
 * is: written (a check), being written (turning, "writing"), or still to come (faded).
 */
@Composable
fun FileRow(path: String, modifier: Modifier = Modifier, change: String? = null, state: FileState? = null) {
    Row(
        modifier.fillMaxWidth().padding(horizontal = 12.dp, vertical = 6.dp),
        horizontalArrangement = Arrangement.spacedBy(8.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        AnimatedContent(state, transitionSpec = { fadeIn(DsMotionSpecs.fast) togetherWith fadeOut(DsMotionSpecs.fast) }, label = "file-state") { now ->
            when (now) {
                FileState.Done -> DsIcon(Lucide.Check, 12.dp, Ds.accent)
                FileState.Writing -> DsSpinner(12.dp, Ds.text)
                FileState.Queued -> DsIcon(Lucide.Circle, 12.dp, Ds.text3)
                null -> DsIcon(Lucide.FileCode, 12.dp, Ds.text3)
            }
        }
        DsText(path, Ds.label, if (state == FileState.Writing) Ds.text else Ds.text2, Modifier.weight(1f), singleLine = true)
        when {
            state == FileState.Writing -> Text("writing", style = Ds.label, color = Ds.text2)
            change != null -> Text(change, style = Ds.label, color = Ds.text3)
        }
    }
}

/** A plain key with a machine value: "Code   23 files · main@a3f9c21". */
@Composable
fun KVRow(label: String, value: String, modifier: Modifier = Modifier, icon: NomiGlyph? = null) {
    Row(modifier.fillMaxWidth().padding(vertical = 7.dp), horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
        if (icon != null) DsIcon(icon, 14.dp, Ds.text3)
        Text(label, style = Ds.ui, color = Ds.text2, modifier = Modifier.width(72.dp))
        DsText(value, Ds.label, Ds.text, Modifier.weight(1f), singleLine = true)
    }
}

/** The step an agent is on right now, as it really is, with the time so far: no progress bar. */
@Composable
fun LiveStep(step: String, modifier: Modifier = Modifier, elapsed: String? = null, stopLabel: String? = null, onStop: (() -> Unit)? = null) {
    Row(
        modifier.fillMaxWidth().clip(DsShapes.sm).dsOutline(Ds.border).padding(horizontal = 12.dp, vertical = 8.dp),
        horizontalArrangement = Arrangement.spacedBy(8.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        DsSpinner(14.dp, Ds.text2)
        AnimatedContent(step, Modifier.weight(1f).semantics { liveRegion = LiveRegionMode.Polite }, label = "live-step") {
            DsText(it, Ds.label, Ds.text, singleLine = true)
        }
        if (elapsed != null) Text(elapsed, style = Ds.label, color = Ds.text3)
        if (stopLabel != null && onStop != null) DsButton(stopLabel, onStop, variant = DsButtonVariant.Ghost, iconStart = Lucide.Square)
    }
}

/** An answer: who gave it and when, then prose first. Rich blocks come as [content], only when it has that shape. */
@Composable
fun MessageAgent(agent: Crew, name: String, modifier: Modifier = Modifier, meta: String? = null, text: String? = null, working: Boolean = false, content: (@Composable ColumnScope.() -> Unit)? = null) {
    Column(modifier.fillMaxWidth(), verticalArrangement = Arrangement.spacedBy(12.dp)) {
        AgentByline(agent, name, meta = meta, working = working)
        if (text != null) Text(text, style = Ds.body, color = Ds.text2)
        content?.invoke(this)
    }
}

enum class SendStatus { Sent, Sending, Queued, Failed }

data class MessageVersion(val current: Int, val total: Int, val onPrev: () -> Unit, val onNext: () -> Unit)
data class AttachedFile(val name: String, val detail: String? = null)

/**
 * What the person said: on the right in a surface-2 bubble, with any files above it. Until it's sent it's a little
 * faded and says where it is (sending, queued while offline, or not sent with retry and edit). Once sent: copy, edit
 * and the time; an edited message says so and steps between its versions.
 */
@Composable
fun MessageUser(
    text: String,
    modifier: Modifier = Modifier,
    status: SendStatus = SendStatus.Sent,
    time: String? = null,
    edited: Boolean = false,
    version: MessageVersion? = null,
    files: List<AttachedFile> = emptyList(),
    onCancel: (() -> Unit)? = null,
    onRetry: (() -> Unit)? = null,
    onEdit: (() -> Unit)? = null,
    onDiscard: (() -> Unit)? = null,
    onDelete: (() -> Unit)? = null,
    above: (@Composable () -> Unit)? = null,
) {
    val copier = rememberCopier()
    val fade by animateFloatAsState(if (status == SendStatus.Sent) 1f else UNSENT_ALPHA, DsMotion.effects(), label = "unsent-fade")
    Column(modifier.fillMaxWidth(), horizontalAlignment = Alignment.End, verticalArrangement = Arrangement.spacedBy(6.dp)) {
        above?.invoke()
        if (files.isNotEmpty()) {
            FlowRow(horizontalArrangement = Arrangement.spacedBy(8.dp, Alignment.End), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                files.forEach { file ->
                    Row(
                        Modifier.width(200.dp).clip(DsShapes.sm).background(Ds.surface).dsOutline(Ds.border).padding(horizontal = 10.dp, vertical = 8.dp),
                        horizontalArrangement = Arrangement.spacedBy(10.dp),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Box(Modifier.size(28.dp).clip(DsShapes.xs).background(Ds.surface2), contentAlignment = Alignment.Center) { DsIcon(Lucide.FileText, 14.dp, Ds.text2) }
                        Column(Modifier.weight(1f)) {
                            DsText(file.name, Ds.mono, Ds.text, singleLine = true)
                            if (file.detail != null) DsText(file.detail, Ds.label, Ds.text3, singleLine = true)
                        }
                    }
                }
            }
        }
        if (text.isNotEmpty()) Text(
            text,
            style = Ds.body,
            color = Ds.text,
            modifier = Modifier
                .widthIn(max = 576.dp)
                .alpha(fade)
                .clip(DsShapes.md)
                .background(Ds.surface2)
                .then(if (status == SendStatus.Failed) Modifier.dsOutline(Ds.dangerBorder, DsShapes.md) else Modifier)
                .padding(horizontal = 14.dp, vertical = 8.dp),
        )
        AnimatedContent(status, label = "send-status") { now ->
            Row(horizontalArrangement = Arrangement.spacedBy(6.dp), verticalAlignment = Alignment.CenterVertically) {
                when (now) {
                    SendStatus.Sending -> {
                        DsSpinner(12.dp, Ds.text3)
                        Text("sending…", style = Ds.label, color = Ds.text3)
                    }
                    SendStatus.Queued -> {
                        DsIcon(Lucide.Hourglass, 12.dp, Ds.text3)
                        Text("queued · sends when you're back online", style = Ds.label, color = Ds.text3)
                        if (onCancel != null) DsTextAction("Cancel", onCancel)
                    }
                    SendStatus.Failed -> {
                        DsIcon(Lucide.Alert, 12.dp, Ds.danger)
                        Text("not sent", style = Ds.label, color = Ds.danger)
                        if (onRetry != null) DsTextAction("Retry", onRetry)
                        if (onEdit != null) DsTextAction("Edit", onEdit)
                        if (onDiscard != null) DsTextAction("Discard", onDiscard)
                    }
                    SendStatus.Sent -> when {
                        edited || version != null -> {
                            if (edited && time != null) Text("edited · $time", style = Ds.label, color = Ds.text3)
                            if (version != null) VersionSwitcher(version.current, version.total, version.onPrev, version.onNext)
                        }
                        else -> {
                            DsBareIcon(Lucide.Copy, if (copier.copied) "Copied" else "Copy message", { copier.copy(text) }, size = 13.dp)
                            if (onEdit != null) DsBareIcon(Lucide.Pencil, "Edit message", onEdit, size = 13.dp)
                            if (onDelete != null) DsBareIcon(Lucide.Trash, "Delete message", onDelete, size = 13.dp)
                            if (time != null) Text(time, style = Ds.label, color = Ds.text3)
                        }
                    }
                }
            }
        }
    }
}

/** Steps between versions of an edited message or a regenerated reply: ‹ 1 / 2 ›. */
@Composable
fun VersionSwitcher(current: Int, total: Int, onPrev: () -> Unit, onNext: () -> Unit, modifier: Modifier = Modifier) {
    Row(modifier, verticalAlignment = Alignment.CenterVertically) {
        DsBareIcon(Lucide.ChevronLeft, "Previous version", onPrev, size = 12.dp, tint = Ds.text2, enabled = current > 1)
        AnimatedContent(current, label = "version") { Text("$it / $total", style = Ds.label, color = Ds.text2) }
        DsBareIcon(Lucide.ChevronRight, "Next version", onNext, size = 12.dp, tint = Ds.text2, enabled = current < total)
    }
}
