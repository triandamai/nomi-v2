package ai.nomi.feature.home

import ai.nomi.core.timeAgo
import ai.nomi.data.Session
import ai.nomi.designsystem.component.Composer
import ai.nomi.designsystem.component.ComposerPlatform
import ai.nomi.designsystem.component.LocalComposerPlatform
import ai.nomi.designsystem.component.NomiButton
import ai.nomi.designsystem.component.NomiButtonSize
import ai.nomi.designsystem.component.NomiButtonVariant
import ai.nomi.designsystem.component.NomiListPanel
import ai.nomi.designsystem.component.NomiRow
import ai.nomi.designsystem.component.NomiSectionHeader
import ai.nomi.designsystem.component.NomiSuggestionChip
import ai.nomi.designsystem.shape.AgentIdentity
import ai.nomi.designsystem.shape.AgentShape
import ai.nomi.designsystem.shape.MorphingMark
import ai.nomi.designsystem.shape.NomiIdentity
import ai.nomi.designsystem.shape.ShapeMotion
import ai.nomi.designsystem.shape.agentIdentity
import ai.nomi.designsystem.theme.NomiTheme
import ai.nomi.designsystem.theme.toFittedBrush
import ai.nomi.network.CrewMemberDto
import androidx.compose.foundation.background
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawingPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.em
import androidx.compose.ui.unit.sp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import kotlin.time.Clock

const val HOME_SCREEN_TAG = "home-screen"

/** `.home__row--narrow`: the stage layout's centred column. */
private val NarrowMaxWidth = 720.dp
private val PagePadding = 20.dp
private val SectionGap = 36.dp
private val AskGap = 20.dp
private val ChipGap = 8.dp
private val HeroMarkSize = 76.dp
private val HeroGap = 14.dp
private val HeroTopPadding = 32.dp
private const val HERO_STEP_MS = 1600L
private val HeroSequence = listOf("cookie9", "sunny12", "flower5", "clover4")
private const val HERO_WIDTH_SHARE = 0.07f
private const val HERO_MIN_SP = 40f
private const val HERO_MAX_SP = 64f
private const val HERO_LINE_HEIGHT = 1.02f
private const val HERO_TRACKING_EM = -0.035f
private const val SUB_SCALE = 0.4f
private const val SUB_WEIGHT = 650
private val RecentAvatar = 32.dp

private const val PLACEHOLDER = "Ask Nomi, or hand something to the crew…"
private const val START_LABEL = "Start chat"
private const val DESKTOP_HINT = "Enter to send · Shift+Enter for a new line"
private val Suggestions = listOf("Summarize this week’s spending", "Plan my Saturday", "Start a small website project", "Remind me to stretch at 4pm")

@Composable
fun HomeScreen(viewModel: HomeViewModel, onOpenChat: (String) -> Unit, onOpenChats: () -> Unit, modifier: Modifier = Modifier) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    HomeContent(
        state,
        onDraftChange = viewModel::onDraftChange,
        onSend = { viewModel.startChat(onOpenChat) },
        onSuggestion = viewModel::useSuggestion,
        onOpenChat = onOpenChat,
        onOpenChats = onOpenChats,
        modifier = modifier,
    )
}

/** Home in the web's default "stage" layout: hero greeting and composer, the crew strip, then recent chats. */
@OptIn(ExperimentalLayoutApi::class)
@Composable
fun HomeContent(
    state: HomeUiState,
    onDraftChange: (String) -> Unit,
    onSend: () -> Unit,
    onSuggestion: (String) -> Unit,
    onOpenChat: (String) -> Unit,
    onOpenChats: () -> Unit,
    modifier: Modifier = Modifier,
    nowMs: Long = Clock.System.now().toEpochMilliseconds(),
) {
    BoxWithConstraints(modifier.fillMaxSize().background(NomiTheme.extendedColors.ground).testTag(HOME_SCREEN_TAG)) {
        val heroSp = (maxWidth.value * HERO_WIDTH_SHARE).coerceIn(HERO_MIN_SP, HERO_MAX_SP)
        Column(
            Modifier.fillMaxSize().imePadding().verticalScroll(rememberScrollState()).safeDrawingPadding().padding(PagePadding),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            Column(Modifier.widthIn(max = NarrowMaxWidth).fillMaxWidth(), verticalArrangement = Arrangement.spacedBy(SectionGap)) {
                Hero(state, heroSp)
                Column(verticalArrangement = Arrangement.spacedBy(AskGap)) {
                    Composer(
                        value = state.draft,
                        onValueChange = onDraftChange,
                        onSend = onSend,
                        placeholder = PLACEHOLDER,
                        sendLabel = START_LABEL,
                        sending = state.sending,
                        hint = if (LocalComposerPlatform.current == ComposerPlatform.Desktop) DESKTOP_HINT else "",
                    )
                    state.error?.let { Text(it, style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.error) }
                    FlowRow(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(ChipGap, Alignment.CenterHorizontally), verticalArrangement = Arrangement.spacedBy(ChipGap)) {
                        Suggestions.forEach { NomiSuggestionChip(it, onClick = { onSuggestion(it) }) }
                    }
                }
                if (state.crew.isNotEmpty()) CrewStrip(state)
                Recent(state.recent, nowMs, onOpenChat, onOpenChats)
            }
        }
    }
}

@Composable
private fun Hero(state: HomeUiState, sizeSp: Float) {
    Column(
        Modifier.fillMaxWidth().padding(top = HeroTopPadding),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(HeroGap),
    ) {
        MorphingMark(HeroMarkSize, HeroSequence, HERO_STEP_MS, face = true)
        Text(
            state.greeting,
            style = MaterialTheme.typography.displayLarge.copy(
                fontSize = sizeSp.sp,
                lineHeight = (sizeSp * HERO_LINE_HEIGHT).sp,
                letterSpacing = HERO_TRACKING_EM.em,
                textAlign = TextAlign.Center,
            ),
            color = MaterialTheme.colorScheme.onSurface,
            modifier = Modifier.semantics { heading() },
        )
        Text(
            state.subtitle,
            style = MaterialTheme.typography.displayLarge.copy(
                fontSize = (sizeSp * SUB_SCALE).sp,
                lineHeight = (sizeSp * SUB_SCALE * HERO_LINE_HEIGHT).sp,
                fontWeight = FontWeight(SUB_WEIGHT),
                textAlign = TextAlign.Center,
            ),
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}

/** `.crew--strip .crew__item`. */
private val CrewCardWidth = 196.dp
private val CrewCardBadgedWidth = 248.dp
private val CrewAvatar = 40.dp
private val CrewGap = 10.dp
private val CrewCardGap = 12.dp
private const val CREW_PANEL_ALPHA = 0.72f
private const val CREW_ROLE_ALPHA = 0.8f
private const val CREW_IDLE_ALPHA = 0.6f
private const val CREW_NAME_WEIGHT = 650
private const val BADGE_WEIGHT = 700

private fun CrewMemberDto.identity(): AgentIdentity {
    val shape = shape
    val tone = tone
    return if (isDynamic && shape != null && tone != null) AgentIdentity(shape, tone, ShapeMotion.fromKey(motion)) else agentIdentity(agentType)
}

@Composable
private fun CrewStrip(state: HomeUiState) {
    Column(verticalArrangement = Arrangement.spacedBy(CrewCardGap)) {
        NomiSectionHeader("Your crew") {
            if (state.workingCount > 0) {
                Text("${state.workingCount} working", style = NomiTheme.metaStyle, color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
        }
        Row(Modifier.horizontalScroll(rememberScrollState()), horizontalArrangement = Arrangement.spacedBy(CrewGap)) {
            state.crew.forEach { CrewCard(it) }
        }
    }
}

@Composable
private fun CrewCard(member: CrewMemberDto) {
    val colors = MaterialTheme.colorScheme
    val badge = when (member.state) {
        "waiting" -> "Needs you"
        "done" -> "Done"
        else -> null
    }
    Row(
        Modifier
            .width(if (badge != null) CrewCardBadgedWidth else CrewCardWidth)
            .clip(MaterialTheme.shapes.extraLarge)
            .background(colors.surfaceContainerLowest.copy(alpha = CREW_PANEL_ALPHA))
            .padding(start = CrewGap, top = CrewGap, bottom = CrewGap, end = CrewCardGap),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(CrewCardGap),
    ) {
        val identity = member.identity()
        AgentShape(identity, CrewAvatar, working = member.state == "working", face = identity == NomiIdentity)
        Column(Modifier.weight(FILL)) {
            Text(member.name, style = MaterialTheme.typography.bodyLarge.copy(fontWeight = FontWeight(CREW_NAME_WEIGHT)), color = colors.onSurface, maxLines = 1)
            Text(
                if (member.state == "idle") member.role else member.status,
                style = MaterialTheme.typography.bodyMedium,
                color = colors.onSurface.copy(alpha = if (member.state == "idle") CREW_IDLE_ALPHA else CREW_ROLE_ALPHA),
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }
        badge?.let { CrewBadge(it, ember = member.state == "waiting") }
    }
}

private val BadgePaddingH = 10.dp
private const val DONE_BADGE_ALPHA = 0.14f
private const val FILL = 1f
private val BadgePaddingV = 4.dp

@Composable
private fun CrewBadge(text: String, ember: Boolean) {
    val gradient = NomiTheme.gradients.ember
    val shape = MaterialTheme.shapes.extraLarge
    val ink = if (ember) Color(gradient.onColor) else MaterialTheme.colorScheme.onSurface
    Box(
        Modifier
            .clip(shape)
            .then(if (ember) Modifier.background(gradient.toFittedBrush()) else Modifier.background(MaterialTheme.colorScheme.onSurface.copy(alpha = DONE_BADGE_ALPHA)))
            .padding(horizontal = BadgePaddingH, vertical = BadgePaddingV),
    ) {
        Text(text, style = MaterialTheme.typography.labelSmall.copy(fontWeight = FontWeight(BADGE_WEIGHT)), color = ink)
    }
}

@Composable
private fun Recent(sessions: List<Session>, nowMs: Long, onOpenChat: (String) -> Unit, onOpenChats: () -> Unit) {
    Column(verticalArrangement = Arrangement.spacedBy(CrewCardGap)) {
        NomiSectionHeader("Recent") {
            NomiButton(onClick = onOpenChats, variant = NomiButtonVariant.Text, size = NomiButtonSize.Small) { Text("All chats") }
        }
        if (sessions.isEmpty()) {
            Text("Nothing yet — your chats will show up here.", style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
        } else {
            NomiListPanel {
                sessions.forEach { session ->
                    NomiRow(
                        title = session.title.ifBlank { "New chat" },
                        line = session.lastMessagePreview.ifBlank { "No messages yet" },
                        meta = timeAgo(session.updatedAt, nowMs),
                        emphasized = session.unread,
                        onClick = { onOpenChat(session.id) },
                        leading = { AgentShape(NomiIdentity, RecentAvatar) },
                    )
                }
            }
        }
    }
}
