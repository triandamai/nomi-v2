package ai.nomi.designsystem.ds.mobile

import ai.nomi.designsystem.ds.CopyGlyph
import ai.nomi.designsystem.ds.Crew
import ai.nomi.designsystem.ds.Ds
import ai.nomi.designsystem.ds.DsBareField
import ai.nomi.designsystem.ds.DsBareIcon
import ai.nomi.designsystem.ds.DsIcon
import ai.nomi.designsystem.ds.DsMotion
import ai.nomi.designsystem.ds.DsShapes
import ai.nomi.designsystem.ds.DsSpinner
import ai.nomi.designsystem.ds.DsStatus
import ai.nomi.designsystem.ds.DsTag
import ai.nomi.designsystem.ds.DsText
import ai.nomi.designsystem.ds.DsTextAction
import ai.nomi.designsystem.ds.Hairline
import ai.nomi.designsystem.ds.Mascot
import ai.nomi.designsystem.ds.MinTouch
import ai.nomi.designsystem.ds.RadioMark
import ai.nomi.designsystem.ds.StatusKind
import ai.nomi.designsystem.ds.TagKind
import ai.nomi.designsystem.ds.chat.Cell
import ai.nomi.designsystem.ds.chat.CellTone
import ai.nomi.designsystem.ds.chat.ReplyFeedback
import ai.nomi.designsystem.ds.dsDisabled
import ai.nomi.designsystem.ds.dsOutline
import ai.nomi.designsystem.ds.rememberCopier
import ai.nomi.designsystem.icon.Lucide
import ai.nomi.designsystem.icon.NomiGlyph
import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.scaleIn
import androidx.compose.animation.scaleOut
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.selection.selectable
import androidx.compose.foundation.selection.selectableGroup
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.scale
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.semantics.LiveRegionMode
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.liveRegion
import androidx.compose.ui.semantics.selected
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

private const val PRESSED_POP = 1.12f

/** The phone's own type sizes (ds-twins `--text-phone*`). */
object PhoneType {
    val title: TextStyle @Composable get() = Ds.body.copy(fontSize = 17.sp, lineHeight = 22.sp)
    val body: TextStyle @Composable get() = Ds.body.copy(fontSize = 16.sp, lineHeight = 21.sp)
    val small: TextStyle @Composable get() = Ds.body.copy(fontSize = 14.sp, lineHeight = 18.sp)
    val heading: TextStyle @Composable get() = Ds.body.copy(fontSize = 18.sp, lineHeight = 23.sp)
}

/** A teammate in a shared build: their initials in their colour (peer 1 or 2), or neutral. */
data class Person(val name: String, val initials: String, val peer: Int? = null) {
    val first: String get() = name.split(" ").first()
}

/** A teammate's initials in a circle, in their colour (web PeerAvatar). [ring] sets it off from its neighbours. */
@Composable
fun PeerAvatar(initials: String, modifier: Modifier = Modifier, peer: Int? = null, small: Boolean = false, ring: Boolean = false) {
    val (bg, ink) = when (peer) {
        1 -> Ds.peer1Bg to Ds.peer1
        2 -> Ds.peer2Bg to Ds.peer2
        else -> Ds.surface2 to Ds.text2
    }
    val edge = when {
        !ring -> Color.Transparent
        peer == 1 -> Ds.peer1
        peer == 2 -> Ds.peer2
        else -> Ds.bg
    }
    Box(
        modifier.size(if (small) 20.dp else 28.dp).clip(DsShapes.full).background(bg).border(if (ring) 2.dp else 0.dp, edge, DsShapes.full),
        contentAlignment = Alignment.Center,
    ) { Text(initials, style = if (small) Ds.micro.copy(fontSize = 9.sp) else Ds.label, color = ink) }
}

enum class PhoneButtonVariant { Primary, Secondary, Danger }

/** A phone's full-width button (46dp): primary in the accent, secondary outlined, danger in the error container. */
@Composable
fun PhoneButton(label: String, onClick: () -> Unit, modifier: Modifier = Modifier, variant: PhoneButtonVariant = PhoneButtonVariant.Primary, icon: NomiGlyph? = null, enabled: Boolean = true) {
    val shape = androidx.compose.material3.MaterialTheme.shapes.large
    val (bg, ink, edge) = when (variant) {
        PhoneButtonVariant.Primary -> Triple(Ds.accent, Ds.accentInk, Ds.accent)
        PhoneButtonVariant.Secondary -> Triple(Color.Transparent, Ds.text, Ds.borderStrong)
        PhoneButtonVariant.Danger -> Triple(Ds.dangerBg, Ds.danger, Ds.dangerBorder)
    }
    Row(
        modifier.fillMaxWidth().height(46.dp).clip(shape).background(bg).border(Hairline, edge, shape)
            .clickable(enabled = enabled, role = Role.Button, onClick = onClick).dsDisabled(!enabled),
        horizontalArrangement = Arrangement.spacedBy(8.dp, Alignment.CenterHorizontally),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(label, style = PhoneType.body.copy(fontWeight = FontWeight.Medium), color = ink)
        if (icon != null) DsIcon(icon, 20.dp, ink)
    }
}

@Composable
private fun RoundTool(icon: NomiGlyph, label: String, onClick: () -> Unit, pressed: Boolean = false, content: (@Composable () -> Unit)? = null) {
    val bg by animateColorAsState(if (pressed) Ds.surface2 else Color.Transparent, DsMotion.fast(), label = "round-tool")
    val pop by animateFloatAsState(if (pressed) PRESSED_POP else 1f, DsMotion.spatial(), label = "round-pop")
    Box(
        Modifier.size(MinTouch).clip(DsShapes.full).background(bg).clickable(role = Role.Button, onClick = onClick).semantics { contentDescription = label; selected = pressed },
        contentAlignment = Alignment.Center,
    ) { if (content != null) content() else DsIcon(icon, 18.dp, if (pressed) Ds.text else Ds.text2, Modifier.scale(pop)) }
}

/** A reply's actions on the phone: copy, like, dislike, share, more, as round 38dp tools. */
@Composable
fun ActionBar(text: String, modifier: Modifier = Modifier, feedback: ReplyFeedback? = null, onLike: (() -> Unit)? = null, onDislike: (() -> Unit)? = null, onShare: (() -> Unit)? = null, onMore: (() -> Unit)? = null) {
    val copier = rememberCopier()
    Row(modifier.semantics { contentDescription = "Reply actions" }, verticalAlignment = Alignment.CenterVertically) {
        RoundTool(Lucide.Copy, if (copier.copied) "Copied" else "Copy", { copier.copy(text) }) { CopyGlyph(copier.copied, 18.dp, Ds.text2) }
        if (onLike != null) RoundTool(Lucide.ThumbsUp, "Good reply", onLike, feedback == ReplyFeedback.Liked)
        if (onDislike != null) RoundTool(Lucide.ThumbsDown, "Bad reply", onDislike, feedback == ReplyFeedback.Disliked)
        if (onShare != null) RoundTool(Lucide.Share, "Share", onShare)
        if (onMore != null) RoundTool(Lucide.Ellipsis, "More", onMore)
    }
}

/** The phone's approval card: who and what, to/subject/excerpt, and decline / approve side by side. */
@Composable
fun MobileApprovalCard(
    agent: Crew,
    title: String,
    approveLabel: String,
    rejectLabel: String,
    onApprove: () -> Unit,
    onReject: () -> Unit,
    modifier: Modifier = Modifier,
    to: String? = null,
    subject: String? = null,
    excerpt: String? = null,
    attachment: String? = null,
    approveIcon: NomiGlyph? = null,
    onOpen: (() -> Unit)? = null,
) {
    PhoneCard(modifier) {
        Row(
            Modifier.fillMaxWidth().clickable(enabled = onOpen != null, role = Role.Button) { onOpen?.invoke() }.padding(horizontal = 14.dp, vertical = 12.dp),
            horizontalArrangement = Arrangement.spacedBy(8.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Mascot(agent, 20.dp)
            Text(title, style = Ds.body.copy(fontWeight = FontWeight.SemiBold), color = Ds.text, modifier = Modifier.weight(1f))
            if (onOpen != null) DsIcon(Lucide.ChevronRight, 20.dp, Ds.text3)
        }
        HorizontalDivider(thickness = Hairline, color = Ds.border)
        Column(Modifier.padding(14.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
            if (to != null) Text("to $to", style = Ds.label, color = Ds.text3)
            if (subject != null) Text(subject, style = Ds.body.copy(fontWeight = FontWeight.Medium), color = Ds.text)
            if (excerpt != null) Text(excerpt, style = PhoneType.small, color = Ds.text2, maxLines = 3)
            if (attachment != null) {
                Row(horizontalArrangement = Arrangement.spacedBy(6.dp), verticalAlignment = Alignment.CenterVertically) {
                    DsIcon(Lucide.Paperclip, 12.dp, Ds.text3)
                    Text(attachment, style = Ds.label, color = Ds.text3)
                }
            }
        }
        Row(Modifier.padding(start = 14.dp, end = 14.dp, bottom = 14.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            PhoneButton(rejectLabel, onReject, Modifier.weight(1f), PhoneButtonVariant.Secondary)
            PhoneButton(approveLabel, onApprove, Modifier.weight(1f), icon = approveIcon)
        }
    }
}

@Composable
internal fun PhoneCard(modifier: Modifier = Modifier, content: @Composable androidx.compose.foundation.layout.ColumnScope.() -> Unit) {
    val shape = androidx.compose.material3.MaterialTheme.shapes.extraLarge
    Column(modifier.fillMaxWidth().clip(shape).background(Ds.surface).dsOutline(Ds.border, shape), content = content)
}

/** "Ask Koda about this screen", over a build's preview, with voice. */
@Composable
fun AskBar(onAsk: () -> Unit, onVoice: () -> Unit, modifier: Modifier = Modifier, agent: Crew = Crew.Koda) {
    Row(
        modifier.fillMaxWidth().height(52.dp).clip(DsShapes.full).background(Ds.surface).dsOutline(Ds.borderStrong, DsShapes.full).padding(start = 6.dp, end = 4.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Row(
            Modifier.weight(1f).height(MinTouch).clip(DsShapes.full).clickable(role = Role.Button, onClick = onAsk).padding(horizontal = 10.dp),
            horizontalArrangement = Arrangement.spacedBy(8.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Mascot(agent, 20.dp)
            DsText("Ask ${agent.displayName} about this screen", PhoneType.body, Ds.text3, singleLine = true)
        }
        RoundTool(Lucide.Mic, "Record a voice note", onVoice)
    }
}

enum class AskState { Running, Next, Queued }
data class Ask(val text: String, val who: Person, val state: AskState, val meta: String? = null)

/** The asks waiting for Koda, one at a time, and letting someone's go first. */
@Composable
fun AskQueue(asks: List<Ask>, modifier: Modifier = Modifier, onFirst: ((Int) -> Unit)? = null) {
    val next = asks.indexOfFirst { it.state == AskState.Next }
    PhoneCard(modifier) {
        Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
                Mascot(Crew.Koda, 20.dp)
                Text(if (asks.size == 1) "1 ask in line" else "${asks.size} asks in line", style = Ds.body.copy(fontWeight = FontWeight.Medium), color = Ds.text)
            }
            asks.forEach { ask ->
                Row(horizontalArrangement = Arrangement.spacedBy(10.dp), verticalAlignment = Alignment.CenterVertically) {
                    PeerAvatar(ask.who.initials, peer = ask.who.peer)
                    Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
                        Text(ask.text, style = Ds.body, color = Ds.text)
                        if (ask.meta != null) Text(ask.meta, style = Ds.label, color = Ds.text3)
                    }
                    if (ask.state == AskState.Running) DsSpinner(18.dp, Ds.text2)
                }
            }
            Text("One change at a time, so edits never collide.", style = Ds.ui, color = Ds.text3)
            if (onFirst != null && next > 0) PhoneButton("Let ${asks[next].who.first} go first", { onFirst(next) }, variant = PhoneButtonVariant.Secondary, icon = Lucide.ChevronsUpDown)
        }
    }
}

/** Koda's build plan on the phone: what, the steps, the outcome, build it, its cost, and the other two choices. */
@Composable
fun BuildPlanCard(project: String, summary: String, steps: List<String>, outcome: String, cost: String, onBuild: () -> Unit, onAskFirst: () -> Unit, onKeep: () -> Unit, modifier: Modifier = Modifier) {
    PhoneCard(modifier) {
        Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
                Mascot(Crew.Koda, 20.dp)
                Text("Koda", style = PhoneType.small.copy(fontWeight = FontWeight.Medium), color = Ds.text)
                Text("plan", style = Ds.label, color = Ds.text3)
            }
            Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
                Text(project, style = Ds.label, color = Ds.text3)
                Text(summary, style = PhoneType.heading.copy(fontWeight = FontWeight.SemiBold), color = Ds.text, modifier = Modifier.semantics { heading() })
            }
            Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                steps.forEachIndexed { index, step ->
                    Row(horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                        Text("${index + 1}", style = Ds.mono, color = Ds.text3, modifier = Modifier.width(14.dp))
                        Text(step, style = Ds.body, color = Ds.text)
                    }
                }
            }
            Row(
                Modifier.fillMaxWidth().clip(DsShapes.sm).background(Ds.accentBg).padding(horizontal = 12.dp, vertical = 10.dp),
                horizontalArrangement = Arrangement.spacedBy(8.dp),
            ) {
                DsIcon(Lucide.Sparkles, 14.dp, Ds.accent, Modifier.padding(top = 2.dp))
                Text(outcome, style = PhoneType.small, color = Ds.text)
            }
            PhoneButton("Build it", onBuild, icon = Lucide.ArrowRight)
            Text(cost, style = Ds.caption, color = Ds.text3, textAlign = TextAlign.Center, modifier = Modifier.fillMaxWidth())
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                PhoneButton("Ask before it goes live", onAskFirst, Modifier.weight(1f), PhoneButtonVariant.Secondary)
                PhoneButton("Keep planning", onKeep, Modifier.weight(1f), PhoneButtonVariant.Secondary)
            }
        }
    }
}

/** Steps as segments: done in the accent, the one on now in text, the rest in the border. Each fills as it goes. */
@Composable
fun StepSegments(current: Int, total: Int, modifier: Modifier = Modifier, doneColor: Color = Ds.accent, nowColor: Color = Ds.text) {
    Row(modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(4.dp)) {
        repeat(total) { index ->
            val color by animateColorAsState(
                when {
                    index < current - 1 -> doneColor
                    index == current - 1 -> nowColor
                    else -> Ds.borderStrong
                },
                DsMotion.effects(), label = "step-segment",
            )
            Box(Modifier.weight(1f).height(4.dp).clip(DsShapes.full).background(color))
        }
    }
}

/** A build on the phone: what it's doing now, step x of y, that you can close the app, and stop. */
@Composable
fun BuildProgress(project: String, elapsed: String, step: String, current: Int, total: Int, label: String, eta: String, onStop: () -> Unit, modifier: Modifier = Modifier) {
    PhoneCard(modifier) {
        Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
                Mascot(Crew.Koda, 20.dp, working = true)
                Text("Building $project", style = Ds.body.copy(fontWeight = FontWeight.Medium), color = Ds.text, modifier = Modifier.weight(1f))
                Text(elapsed, style = Ds.label, color = Ds.text2)
            }
            StepSegments(current, total)
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
                DsSpinner(14.dp, Ds.text2)
                AnimatedContent(step, Modifier.weight(1f), label = "build-step") { Text(it, style = Ds.mono, color = Ds.text) }
            }
            Text("Step $current of $total · $label · $eta", style = Ds.ui, color = Ds.text2, modifier = Modifier.semantics { liveRegion = LiveRegionMode.Polite })
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                DsIcon(Lucide.Bell, 14.dp, Ds.text3, Modifier.padding(top = 2.dp))
                Text("You can close the app. We'll notify you when it's live.", style = Ds.caption, color = Ds.text3)
            }
            Row(
                Modifier.align(Alignment.CenterHorizontally).heightIn(min = MinTouch).clip(DsShapes.full).clickable(role = Role.Button, onClick = onStop).padding(horizontal = 16.dp),
                horizontalArrangement = Arrangement.spacedBy(6.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                DsIcon(Lucide.Square, 14.dp, Ds.danger)
                Text("Stop", style = PhoneType.small, color = Ds.danger)
            }
        }
    }
}

/** Who spoke on the phone: their mascot, name, and when. */
@Composable
fun Byline(agent: Crew, name: String, modifier: Modifier = Modifier, meta: String? = null) {
    Row(modifier, horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
        Mascot(agent, 20.dp)
        Text(name, style = PhoneType.small.copy(fontWeight = FontWeight.SemiBold), color = Ds.text)
        if (meta != null) Text(meta, style = Ds.label, color = Ds.text3)
    }
}

/** A teammate's change is ready: publish it, or review it in the preview. */
@Composable
fun ChangeReady(who: Person, meta: String, onPublish: () -> Unit, onReview: () -> Unit, modifier: Modifier = Modifier) {
    PhoneCard(modifier) {
        Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
            Row(horizontalArrangement = Arrangement.spacedBy(10.dp), verticalAlignment = Alignment.CenterVertically) {
                PeerAvatar(who.initials, peer = who.peer)
                Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
                    Text("${who.first}'s change is ready", style = Ds.body.copy(fontWeight = FontWeight.Medium), color = Ds.text)
                    Text(meta, style = Ds.label, color = Ds.text3)
                }
            }
            PhoneButton("Publish", onPublish, icon = Lucide.ArrowUp)
            PhoneButton("Review in preview", onReview, variant = PhoneButtonVariant.Secondary, icon = Lucide.Eye)
        }
    }
}

/** A chat in the phone's list: the crew member on their tint, the title and time, the last line, and unread. */
@Composable
fun ChatRow(agent: Crew, title: String, onClick: () -> Unit, modifier: Modifier = Modifier, time: String? = null, preview: String? = null, unread: Boolean = false) {
    Row(
        modifier.fillMaxWidth().clip(DsShapes.md).clickable(role = Role.Button, onClick = onClick).padding(horizontal = 16.dp, vertical = 10.dp),
        horizontalArrangement = Arrangement.spacedBy(12.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Box(Modifier.size(48.dp).clip(DsShapes.full).background(agent.tint), contentAlignment = Alignment.Center) { Mascot(agent, 32.dp) }
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(3.dp)) {
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
                DsText(title, PhoneType.body.copy(fontWeight = if (unread) FontWeight.SemiBold else FontWeight.Medium), Ds.text, Modifier.weight(1f), singleLine = true)
                if (time != null) Text(time, style = Ds.label, color = if (unread) Ds.accent else Ds.text3)
            }
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
                DsText(preview ?: "", PhoneType.small, Ds.text2, Modifier.weight(1f), singleLine = true)
                androidx.compose.animation.AnimatedVisibility(unread, enter = scaleIn(DsMotion.spatial()), exit = scaleOut(DsMotion.fast())) {
                    Box(Modifier.size(8.dp).clip(DsShapes.full).background(Ds.accent).semantics { contentDescription = "Unread" })
                }
            }
        }
    }
}

/**
 * The phone's composer: attachments above, a voice-transcribed notice, attach, the field with the mic inside, and
 * send (which pops in once there's something to send) or stop while a reply streams.
 */
@Composable
fun MobileComposer(
    value: String,
    onValueChange: (String) -> Unit,
    onSend: (String) -> Unit,
    modifier: Modifier = Modifier,
    streaming: Boolean = false,
    notice: String? = null,
    placeholder: String = "Message Nomi",
    onAttach: (() -> Unit)? = null,
    onRecord: (() -> Unit)? = null,
    onStop: (() -> Unit)? = null,
    attachments: (@Composable () -> Unit)? = null,
    toolbar: (@Composable () -> Unit)? = null,
) {
    val ready = value.isNotBlank()
    Column(modifier.fillMaxWidth().background(Ds.bg).padding(horizontal = 12.dp, vertical = 8.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
        if (attachments != null) Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) { attachments() }
        if (notice != null) {
            Row(horizontalArrangement = Arrangement.spacedBy(6.dp), verticalAlignment = Alignment.CenterVertically) {
                DsIcon(Lucide.Mic, 12.dp, Ds.text3)
                Text(notice, style = Ds.label, color = Ds.text3)
            }
        }
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.Bottom) {
            if (onAttach != null) {
                Box(Modifier.size(MinTouch).clip(DsShapes.full).background(Ds.surface2).clickable(role = Role.Button, onClick = onAttach).semantics { contentDescription = "Attach a file" }, contentAlignment = Alignment.Center) {
                    DsIcon(Lucide.Plus, 20.dp, Ds.text2)
                }
            }
            Row(
                Modifier.weight(1f).heightIn(min = MinTouch).clip(DsShapes.full).background(Ds.surface).dsOutline(Ds.borderStrong, DsShapes.full).padding(start = 16.dp, end = 4.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                DsBareField(value, onValueChange, placeholder, PhoneType.body, Modifier.weight(1f).padding(vertical = 10.dp), singleLine = false)
                if (onRecord != null) DsBareIcon(Lucide.Mic, "Record a voice note", onRecord, size = 20.dp, tint = Ds.text2)
            }
            val mode = when {
                streaming && onStop != null -> 2
                ready -> 1
                else -> 0
            }
            val pop = DsMotion.spatial<Float>()
            val quick = DsMotion.fast<Float>()
            AnimatedContent(mode, transitionSpec = { scaleIn(pop) + fadeIn(quick) togetherWith scaleOut(quick) + fadeOut(quick) }, label = "phone-send") { m ->
                when (m) {
                    2 -> Box(Modifier.size(MinTouch).clip(DsShapes.full).background(Ds.text).clickable(role = Role.Button) { onStop?.invoke() }.semantics { contentDescription = "Stop reply" }, contentAlignment = Alignment.Center) {
                        DsIcon(Lucide.Square, 14.dp, Ds.bg)
                    }
                    1 -> Box(Modifier.size(MinTouch).clip(DsShapes.full).background(Ds.accent).clickable(role = Role.Button) { onSend(value.trim()) }.semantics { contentDescription = "Send" }, contentAlignment = Alignment.Center) {
                        DsIcon(Lucide.ArrowUp, 20.dp, Ds.accentInk)
                    }
                    else -> Spacer(Modifier.size(0.dp))
                }
            }
        }
        if (toolbar != null) Row(Modifier.padding(horizontal = 2.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) { toolbar() }
    }
}

enum class DnsState { Waiting, Verified, Failed }

/** A DNS record to add at the registrar: its type and name, whether it's found yet, and its value to copy. */
@Composable
fun DnsRecord(type: String, name: String, value: String, state: DnsState, modifier: Modifier = Modifier) {
    val copier = rememberCopier()
    val (label, tint) = when (state) {
        DnsState.Waiting -> "waiting" to Ds.warning
        DnsState.Verified -> "verified" to Ds.accent
        DnsState.Failed -> "not found" to Ds.danger
    }
    Column(modifier.fillMaxWidth().clip(DsShapes.md).background(Ds.surface).dsOutline(Ds.border, DsShapes.md).padding(12.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
            Text(type, style = Ds.label.copy(fontWeight = FontWeight.Medium), color = Ds.text, modifier = Modifier.clip(DsShapes.xs).background(Ds.surface2).padding(horizontal = 6.dp, vertical = 2.dp))
            Text("Name: $name", style = PhoneType.small, color = Ds.text, modifier = Modifier.weight(1f))
            AnimatedContent(state, label = "dns-state") {
                Row(horizontalArrangement = Arrangement.spacedBy(5.dp), verticalAlignment = Alignment.CenterVertically) {
                    Box(Modifier.size(6.dp).clip(DsShapes.full).background(tint))
                    Text(label, style = Ds.caption, color = tint)
                }
            }
        }
        Row(Modifier.fillMaxWidth().clip(DsShapes.sm).background(Ds.bg).padding(start = 10.dp), verticalAlignment = Alignment.CenterVertically) {
            DsText(value, Ds.mono, Ds.text, Modifier.weight(1f), singleLine = true)
            DsBareIcon(if (copier.copied) Lucide.Check else Lucide.Copy, if (copier.copied) "Copied" else "Copy $value", { copier.copy(value) }, size = 18.dp, tint = Ds.text2)
        }
    }
}

/** A failure on the phone: what failed and its code, the provider's own words quoted, then the next moves. */
@Composable
fun ErrorCard(
    title: String,
    provider: String,
    reason: String,
    modifier: Modifier = Modifier,
    code: String? = null,
    meta: String? = null,
    primaryLabel: String? = null,
    onPrimary: (() -> Unit)? = null,
    secondaryLabel: String? = null,
    onSecondary: (() -> Unit)? = null,
) {
    PhoneCard(modifier.semantics { liveRegion = LiveRegionMode.Assertive }) {
        Row(Modifier.fillMaxWidth().background(Ds.dangerBg).padding(horizontal = 14.dp, vertical = 12.dp), horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
            DsIcon(Lucide.Alert, 20.dp, Ds.danger)
            Text(title, style = Ds.body.copy(fontWeight = FontWeight.SemiBold), color = Ds.danger, modifier = Modifier.weight(1f))
            if (code != null) Text(code, style = Ds.label, color = Ds.text3)
        }
        Column(Modifier.padding(14.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
            Column(Modifier.fillMaxWidth().clip(DsShapes.sm).background(Ds.bg).padding(12.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                Text("$provider said", style = Ds.caption, color = Ds.text3)
                Text(reason, style = PhoneType.small.copy(fontFamily = Ds.label.fontFamily), color = Ds.text)
            }
            if (meta != null) Text(meta, style = Ds.label, color = Ds.text3)
            if (primaryLabel != null && onPrimary != null) PhoneButton(primaryLabel, onPrimary)
            if (secondaryLabel != null && onSecondary != null) PhoneButton(secondaryLabel, onSecondary, variant = PhoneButtonVariant.Secondary)
        }
    }
}

/** A build that's live: its address to copy, open and share, and what changed. */
@Composable
fun LiveCard(meta: String, url: String, summary: String, onOpen: () -> Unit, onShare: () -> Unit, modifier: Modifier = Modifier) {
    val copier = rememberCopier()
    PhoneCard(modifier) {
        Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
                Mascot(Crew.Koda, 20.dp)
                Text("Koda", style = PhoneType.small.copy(fontWeight = FontWeight.Medium), color = Ds.text)
                DsText(meta, Ds.label, Ds.text3, Modifier.weight(1f), singleLine = true)
                DsStatus(StatusKind.Live, "Live")
            }
            Row(Modifier.fillMaxWidth().clip(DsShapes.sm).background(Ds.bg).dsOutline(Ds.border).padding(start = 12.dp), horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
                DsIcon(Lucide.Globe, 14.dp, Ds.text2)
                DsText(url.removePrefix("https://").removePrefix("http://"), Ds.mono, Ds.text, Modifier.weight(1f), singleLine = true)
                DsBareIcon(if (copier.copied) Lucide.Check else Lucide.Copy, if (copier.copied) "Copied" else "Copy link", { copier.copy(url) }, size = 18.dp, tint = Ds.text2)
            }
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                PhoneButton("Open", onOpen, Modifier.weight(1f), icon = Lucide.ExternalLink)
                PhoneButton("Share", onShare, Modifier.weight(1f), PhoneButtonVariant.Secondary, Lucide.Share)
            }
            Text(summary, style = Ds.caption, color = Ds.text3)
        }
    }
}

/** The phone's top bar in a chat: back, the title with who's on it, and the chat's options. */
@Composable
fun NavBar(title: String, onBack: () -> Unit, modifier: Modifier = Modifier, who: Pair<Crew, String>? = null, onMenu: (() -> Unit)? = null) {
    Row(modifier.fillMaxWidth().height(56.dp).background(Ds.bg).padding(horizontal = 4.dp), verticalAlignment = Alignment.CenterVertically) {
        DsBareIcon(Lucide.ChevronLeft, "Back", onBack, size = 26.dp, tint = Ds.accent)
        Column(Modifier.weight(1f), horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(2.dp)) {
            DsText(title, PhoneType.title.copy(fontWeight = FontWeight.SemiBold), Ds.text, Modifier.semantics { heading() }, singleLine = true)
            if (who != null) {
                Row(horizontalArrangement = Arrangement.spacedBy(4.dp), verticalAlignment = Alignment.CenterVertically) {
                    Mascot(who.first, 16.dp)
                    Text(who.second, style = Ds.caption, color = Ds.text3)
                }
            }
        }
        if (onMenu != null) DsBareIcon(Lucide.Ellipsis, "Chat options", onMenu, size = 22.dp, tint = Ds.text2) else Spacer(Modifier.size(MinTouch))
    }
}

/** One answer to a question on the phone: its key, title, detail, recommendation, and a radio; chosen fills in. */
@Composable
fun PhoneOption(title: String, checked: Boolean, onSelect: () -> Unit, modifier: Modifier = Modifier, key: Int? = null, detail: String? = null, recommended: Boolean = false) {
    val edge by animateColorAsState(if (checked) Ds.accent else Ds.borderStrong, DsMotion.fast(), label = "phone-option-edge")
    val fill by animateColorAsState(if (checked) Ds.accentBg else Color.Transparent, DsMotion.fast(), label = "phone-option-fill")
    val shape = androidx.compose.material3.MaterialTheme.shapes.large
    Row(
        modifier.fillMaxWidth().clip(shape).background(fill).border(Hairline, edge, shape).selectable(checked, role = Role.RadioButton, onClick = onSelect).padding(13.dp),
        horizontalArrangement = Arrangement.spacedBy(12.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        if (key != null) {
            Box(Modifier.size(24.dp).clip(DsShapes.xs).background(Ds.surface2), contentAlignment = Alignment.Center) { Text(key.toString(), style = Ds.label, color = Ds.text2) }
        }
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(3.dp)) {
            Text(title, style = PhoneType.body.copy(fontWeight = FontWeight.Medium), color = Ds.text)
            if (detail != null) Text(detail, style = Ds.ui, color = Ds.text3)
            if (recommended) DsTag(TagKind.Go, "Recommended")
        }
        RadioMark(checked, Modifier.size(22.dp))
    }
}

enum class PhoneToastKind { Success, Error, Info }

/** A toast on the phone: its mark, title, detail and action, on the surface. */
@Composable
fun PhoneToast(kind: PhoneToastKind, title: String, modifier: Modifier = Modifier, detail: String? = null, actionLabel: String? = null, onAction: (() -> Unit)? = null) {
    val (icon, tint) = when (kind) {
        PhoneToastKind.Success -> Lucide.Check to Ds.accent
        PhoneToastKind.Error -> Lucide.Alert to Ds.danger
        PhoneToastKind.Info -> Lucide.Info to Ds.text2
    }
    val shape = androidx.compose.material3.MaterialTheme.shapes.large
    Row(
        modifier.fillMaxWidth().clip(shape).background(Ds.surface).dsOutline(Ds.borderStrong, shape).padding(14.dp)
            .semantics { liveRegion = if (kind == PhoneToastKind.Error) LiveRegionMode.Assertive else LiveRegionMode.Polite },
        horizontalArrangement = Arrangement.spacedBy(12.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        DsIcon(icon, 18.dp, tint)
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
            Text(title, style = Ds.body.copy(fontWeight = FontWeight.Medium), color = Ds.text)
            if (detail != null) Text(detail, style = Ds.ui, color = Ds.text3)
        }
        if (actionLabel != null && onAction != null) DsTextAction(actionLabel, onAction, color = Ds.accent)
    }
}

/** Which question of how many, as segments (web mobile Progress). */
@Composable
fun MobileProgress(current: Int, total: Int, modifier: Modifier = Modifier) {
    Box(modifier.semantics { contentDescription = "Question $current of $total" }) { StepSegments(current + 1, total, doneColor = Ds.text, nowColor = Ds.borderStrong) }
}

enum class PublishReady { Idle, Ready }

/** A project's top bar on the phone: back, its name and live status, publish (or up to date), options. */
@Composable
fun ProjectNav(project: String, status: String, publish: PublishReady, onBack: () -> Unit, onMenu: () -> Unit, modifier: Modifier = Modifier, live: Boolean = true, count: Int = 0, onPublish: (() -> Unit)? = null) {
    Row(modifier.fillMaxWidth().height(52.dp).background(Ds.bg).padding(horizontal = 8.dp), horizontalArrangement = Arrangement.spacedBy(4.dp), verticalAlignment = Alignment.CenterVertically) {
        DsBareIcon(Lucide.ChevronLeft, "Back", onBack, size = 22.dp, tint = Ds.text)
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
            DsText(project, PhoneType.body.copy(fontWeight = FontWeight.SemiBold), Ds.text, singleLine = true)
            Row(horizontalArrangement = Arrangement.spacedBy(6.dp), verticalAlignment = Alignment.CenterVertically) {
                Box(Modifier.size(6.dp).clip(DsShapes.full).background(if (live) Ds.accent else Ds.text3))
                DsText(status, Ds.caption, Ds.text3, singleLine = true)
            }
        }
        val ready = publish == PublishReady.Ready
        val bg by animateColorAsState(if (ready) Ds.accent else Color.Transparent, DsMotion.effects(), label = "publish-bg")
        Row(
            Modifier.height(36.dp).clip(androidx.compose.foundation.shape.RoundedCornerShape(16.dp)).background(bg)
                .border(Hairline, if (ready) Ds.accent else Ds.borderStrong, androidx.compose.foundation.shape.RoundedCornerShape(16.dp))
                .clickable(enabled = ready, role = Role.Button) { onPublish?.invoke() }.padding(horizontal = 13.dp),
            horizontalArrangement = Arrangement.spacedBy(6.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            AnimatedContent(ready, label = "project-publish") { r ->
                Row(horizontalArrangement = Arrangement.spacedBy(6.dp), verticalAlignment = Alignment.CenterVertically) {
                    if (r) {
                        DsIcon(Lucide.Rocket, 14.dp, Ds.accentInk)
                        Text(if (count > 0) "Publish $count changes" else "Publish", style = Ds.ui.copy(fontWeight = FontWeight.Medium), color = Ds.accentInk)
                    } else {
                        DsIcon(Lucide.Check, 14.dp, Ds.text3)
                        Text("Up to date", style = Ds.ui, color = Ds.text2)
                    }
                }
            }
        }
        DsBareIcon(Lucide.Ellipsis, "Project options", onMenu, size = 22.dp, tint = Ds.text2)
    }
}

/** A bottom sheet's top: the grabber, title and subtitle, and close. */
@Composable
fun SheetHeader(title: String, onClose: () -> Unit, modifier: Modifier = Modifier, subtitle: String? = null) {
    Column(modifier.fillMaxWidth()) {
        Box(Modifier.fillMaxWidth().height(20.dp), contentAlignment = Alignment.Center) {
            Box(Modifier.width(36.dp).height(5.dp).clip(DsShapes.full).background(Ds.borderStrong))
        }
        Row(Modifier.padding(start = 16.dp, end = 4.dp, bottom = 12.dp), verticalAlignment = Alignment.CenterVertically) {
            Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
                Text(title, style = PhoneType.title.copy(fontWeight = FontWeight.SemiBold), color = Ds.text, modifier = Modifier.semantics { heading() })
                if (subtitle != null) Text(subtitle, style = Ds.ui, color = Ds.text3)
            }
            DsBareIcon(Lucide.X, "Close", onClose, size = 20.dp, tint = Ds.text2)
        }
    }
}

/** A row in a bottom sheet: icon, label, meta, a chevron when it goes further; danger in the error colour. */
@Composable
fun SheetRow(icon: NomiGlyph, label: String, onClick: () -> Unit, modifier: Modifier = Modifier, meta: String? = null, more: Boolean = false, danger: Boolean = false) {
    Row(
        modifier.fillMaxWidth().heightIn(min = 52.dp).clickable(role = Role.Button, onClick = onClick).padding(horizontal = 16.dp),
        horizontalArrangement = Arrangement.spacedBy(14.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        DsIcon(icon, 20.dp, if (danger) Ds.danger else Ds.text2)
        DsText(label, PhoneType.body, if (danger) Ds.danger else Ds.text, Modifier.weight(1f), singleLine = true)
        if (meta != null) Text(meta, style = Ds.label, color = Ds.text3)
        if (more) DsIcon(Lucide.ChevronRight, 20.dp, Ds.text3)
    }
}

data class StatRow(val label: String, val value: String, val delta: Cell? = null, val detail: String? = null)

/** Numbers on the phone as a list: each label, value and change, and the total. */
@Composable
fun StatList(title: String, rows: List<StatRow>, modifier: Modifier = Modifier, sub: String? = null, total: StatRow? = null) {
    PhoneCard(modifier) {
        Row(Modifier.fillMaxWidth().padding(horizontal = 14.dp, vertical = 12.dp), verticalAlignment = Alignment.CenterVertically) {
            Text(title, style = Ds.body.copy(fontWeight = FontWeight.SemiBold), color = Ds.text, modifier = Modifier.weight(1f))
            if (sub != null) Text(sub, style = Ds.label, color = Ds.text3)
        }
        HorizontalDivider(thickness = Hairline, color = Ds.border)
        rows.forEachIndexed { index, row ->
            if (index > 0) HorizontalDivider(thickness = Hairline, color = Ds.border)
            Column(Modifier.padding(horizontal = 14.dp, vertical = 10.dp), verticalArrangement = Arrangement.spacedBy(3.dp)) {
                StatLine(row, false)
                if (row.detail != null) Text(row.detail, style = Ds.caption, color = Ds.text3)
            }
        }
        if (total != null) {
            HorizontalDivider(thickness = Hairline, color = Ds.border)
            Box(Modifier.background(Ds.bg).padding(horizontal = 14.dp, vertical = 10.dp)) { StatLine(total, true) }
        }
    }
}

@Composable
private fun StatLine(row: StatRow, strong: Boolean) {
    Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
        Text(row.label, style = Ds.body.copy(fontWeight = if (strong) FontWeight.SemiBold else FontWeight.Normal), color = Ds.text, modifier = Modifier.weight(1f))
        Text(row.value, style = PhoneType.small.copy(fontFamily = Ds.label.fontFamily, fontWeight = if (strong) FontWeight.Medium else FontWeight.Normal), color = Ds.text)
        Text(
            row.delta?.text ?: "", style = Ds.mono, textAlign = TextAlign.End, modifier = Modifier.width(40.dp),
            color = when (row.delta?.tone) { CellTone.Bad -> Ds.danger; CellTone.Good -> Ds.accent; null -> Ds.text3 },
        )
    }
}

/** A teammate's message in a shared build, in their colour, with when and to whom. */
@Composable
fun TeammateBubble(who: Person, text: String, modifier: Modifier = Modifier, time: String? = null, to: String? = null, phone: Boolean = true) {
    val meta = listOfNotNull(time, to?.let { "to $it" }).joinToString(" · ")
    val (bg, ink) = when (who.peer) {
        1 -> Ds.peer1Bg to Ds.peer1
        2 -> Ds.peer2Bg to Ds.peer2
        else -> Ds.surface2 to Ds.text
    }
    Row(modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(10.dp)) {
        PeerAvatar(who.initials, peer = who.peer)
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(4.dp)) {
            Row(horizontalArrangement = Arrangement.spacedBy(6.dp), verticalAlignment = Alignment.CenterVertically) {
                Text(who.name, style = Ds.ui.copy(fontWeight = FontWeight.Medium), color = ink)
                if (meta.isNotEmpty()) Text(meta, style = Ds.label, color = Ds.text3)
            }
            Text(
                text, style = if (phone) PhoneType.body else Ds.body, color = Ds.text,
                modifier = Modifier.clip(if (phone) androidx.compose.foundation.shape.RoundedCornerShape(24.dp) else DsShapes.md).background(bg).padding(horizontal = 14.dp, vertical = if (phone) 10.dp else 8.dp),
            )
        }
    }
}

/** What the person said on the phone: the accent bubble on the right; not sent shows with retry. */
@Composable
fun UserMessage(text: String, modifier: Modifier = Modifier, failed: Boolean = false, onRetry: (() -> Unit)? = null) {
    Column(modifier.fillMaxWidth(), horizontalAlignment = Alignment.End, verticalArrangement = Arrangement.spacedBy(4.dp)) {
        Text(
            text, style = PhoneType.body, color = Ds.text,
            modifier = Modifier.widthIn(max = 300.dp).clip(androidx.compose.foundation.shape.RoundedCornerShape(22.dp)).background(Ds.surface2).padding(horizontal = 14.dp, vertical = 10.dp),
        )
        if (failed) {
            Row(Modifier.padding(horizontal = 4.dp), horizontalArrangement = Arrangement.spacedBy(6.dp), verticalAlignment = Alignment.CenterVertically) {
                DsIcon(Lucide.Alert, 12.dp, Ds.danger)
                Text("not sent", style = Ds.label, color = Ds.danger)
                if (onRetry != null) DsTextAction("Retry", onRetry)
            }
        }
    }
}

enum class WorkspaceView { Chat, Preview }

/** Chat or preview, on the phone: a two-way switch whose chosen side lifts onto the ground. */
@Composable
fun WorkspaceSwitch(value: WorkspaceView, onChange: (WorkspaceView) -> Unit, modifier: Modifier = Modifier, previewCount: Int? = null) {
    Box(modifier.fillMaxWidth().background(Ds.bg).padding(start = 16.dp, end = 16.dp, top = 6.dp, bottom = 10.dp)) {
        Row(Modifier.fillMaxWidth().height(MinTouch).clip(DsShapes.md).background(Ds.surface2).padding(2.dp).selectableGroup(), horizontalArrangement = Arrangement.spacedBy(2.dp)) {
            listOf(WorkspaceView.Chat to Lucide.Message, WorkspaceView.Preview to Lucide.Eye).forEach { (view, icon) ->
                val checked = view == value
                val bg by animateColorAsState(if (checked) Ds.bg else Color.Transparent, DsMotion.effects(), label = "ws-switch")
                Row(
                    Modifier.weight(1f).fillMaxWidth().height(40.dp).clip(DsShapes.sm).background(bg).selectable(checked, role = Role.RadioButton) { onChange(view) },
                    horizontalArrangement = Arrangement.spacedBy(6.dp, Alignment.CenterHorizontally),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    DsIcon(icon, 15.dp, if (checked) Ds.text else Ds.text2)
                    Text(if (view == WorkspaceView.Chat) "Chat" else "Preview", style = PhoneType.small.copy(fontWeight = if (checked) FontWeight.Medium else FontWeight.Normal), color = if (checked) Ds.text else Ds.text2)
                    if (view == WorkspaceView.Preview && previewCount != null && previewCount > 0) {
                        Text(previewCount.toString(), style = Ds.label, color = Ds.accentInk, modifier = Modifier.clip(DsShapes.full).background(Ds.accent).padding(horizontal = 6.dp))
                    }
                }
            }
        }
    }
}

/** A phone's status bar (9:41, signal, wifi, battery), for phone mockups. */
@Composable
fun PhoneStatusBar(modifier: Modifier = Modifier, ink: Color = Ds.text) {
    Row(modifier.fillMaxWidth().height(44.dp).padding(horizontal = 24.dp), verticalAlignment = Alignment.CenterVertically) {
        Text("9:41", style = PhoneType.title.copy(fontWeight = FontWeight.SemiBold), color = ink, modifier = Modifier.weight(1f))
        Row(horizontalArrangement = Arrangement.spacedBy(6.dp), verticalAlignment = Alignment.CenterVertically) {
            DsIcon(Lucide.Signal, 17.dp, ink)
            DsIcon(Lucide.Wifi, 17.dp, ink)
            DsIcon(Lucide.Battery, 25.dp, ink)
        }
    }
}
