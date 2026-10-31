package ai.nomi.companion

import ai.nomi.designsystem.component.NomiButton
import ai.nomi.designsystem.component.NomiButtonSize
import ai.nomi.designsystem.component.NomiButtonVariant
import ai.nomi.designsystem.icon.NomiIcon
import ai.nomi.designsystem.icon.NomiIcons
import ai.nomi.designsystem.shape.AgentShape
import ai.nomi.designsystem.shape.NomiIdentity
import ai.nomi.designsystem.shape.agentIdentity
import ai.nomi.designsystem.theme.NomiTheme
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.setValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.isShiftPressed
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.onPreviewKeyEvent
import androidx.compose.ui.input.key.type
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp

const val COMPANION_PANEL_TAG = "companion-panel"
const val COMPANION_INPUT_TAG = "companion-input"

/** The colours a companion surface draws with: the notch's black, or the bubble's theme surface. */
data class CompanionPalette(
    val surface: Color,
    val content: Color,
    val muted: Color,
    val field: Color,
    val chip: Color,
    val accent: Color,
    val onAccent: Color,
    val error: Color,
)

private const val NOTCH_BLACK_ARGB = 0xFF000000L
private const val NOTCH_FG_ARGB = 0xFFF4F7F5L
private const val NOTCH_ERROR_ARGB = 0xFFFFB4ABL
private const val NOTCH_MUTED_ALPHA = 0.66f
private const val NOTCH_FIELD_ALPHA = 0.12f
private const val NOTCH_CHIP_ALPHA = 0.08f

/** The notch's palette: pure black so it reads as part of the hardware notch. */
@Composable
fun notchPalette(): CompanionPalette {
    val foreground = Color(NOTCH_FG_ARGB)
    val colors = MaterialTheme.colorScheme
    return CompanionPalette(
        surface = Color(NOTCH_BLACK_ARGB),
        content = foreground,
        muted = foreground.copy(alpha = NOTCH_MUTED_ALPHA),
        field = foreground.copy(alpha = NOTCH_FIELD_ALPHA),
        chip = foreground.copy(alpha = NOTCH_CHIP_ALPHA),
        accent = colors.inversePrimary,
        onAccent = colors.onPrimaryContainer,
        error = Color(NOTCH_ERROR_ARGB),
    )
}

/** The bubble's palette: the theme's card surface. */
@Composable
fun surfacePalette(): CompanionPalette {
    val colors = MaterialTheme.colorScheme
    return CompanionPalette(
        surface = colors.surfaceContainerLowest,
        content = colors.onSurface,
        muted = colors.onSurfaceVariant,
        field = colors.surfaceContainerHigh,
        chip = colors.surfaceContainerLow,
        accent = colors.primary,
        onAccent = colors.onPrimary,
        error = colors.error,
    )
}

private val HeaderAvatar = 32.dp
private val CrewAvatar = 22.dp
private val SectionGap = 14.dp
private val RowGap = 8.dp
private val SmallGap = 4.dp
private val ChipGap = 6.dp
private val ChipIcon = 16.dp
private val RowContentGap = 10.dp
private val RowPaddingH = 10.dp
private val RowPaddingV = 8.dp
private val ChipPaddingH = 12.dp
private val ChipPaddingV = 8.dp
private val InputMinHeight = 48.dp
private val InputPaddingH = 16.dp
private val InputPaddingV = 12.dp
private val SendSize = 44.dp
private val AnswerMaxHeight = 180.dp
private val BubbleCorner = 16.dp
private val QuestionMaxWidth = 300.dp
private const val FILL = 1f
private const val TITLE_WEIGHT = 700
private const val BODY_MAX_LINES = 2
private const val INPUT_MAX_LINES = 4

/**
 * The open companion, shared by the Mac notch and the phone bubble: what brought it up ([card]), then either the
 * conversation the person is having from here or what's going on (crew at work, recent chats, projects), and a
 * roomy box to ask Nomi anything. Answers stream in place; "Open" moves to the full app.
 */
@Composable
fun CompanionPanel(
    overview: CompanionOverview,
    card: CompanionCard?,
    working: ai.nomi.data.CompanionSignal.Working?,
    palette: CompanionPalette,
    onAsk: (text: String, sessionId: String?) -> Unit,
    onOpenLink: (String) -> Unit,
    onOpenApp: () -> Unit,
    onNewConversation: () -> Unit,
    onOpenConversation: () -> Unit,
    modifier: Modifier = Modifier,
    lookAt: Offset? = null,
    /** In a fixed-height surface (the notch) the box to ask in sits at the bottom; elsewhere the panel hugs its content. */
    fillHeight: Boolean = false,
    trailing: @Composable () -> Unit = {},
) {
    val conversation = overview.conversation
    Column(modifier.testTag(COMPANION_PANEL_TAG), verticalArrangement = Arrangement.spacedBy(SectionGap)) {
        PanelHeader(card, working, conversation, palette, lookAt, onOpenLink, onOpenApp, onOpenConversation, trailing)
        Column(
            Modifier.weight(FILL, fill = fillHeight).verticalScroll(rememberScrollState()),
            verticalArrangement = Arrangement.spacedBy(SectionGap),
        ) {
            if (conversation != null) {
                ConversationView(conversation, palette, onNewConversation)
            } else {
                card?.let { CardBody(it, palette) }
                if (overview.crew.isNotEmpty()) CrewSection(overview.crew, palette, onOpenLink)
                if (overview.chats.isNotEmpty()) ChatsSection(overview.chats, palette, onOpenLink)
                if (overview.projects.isNotEmpty()) ProjectsSection(overview.projects, palette, onOpenLink)
            }
        }
        AskField(
            placeholder = when {
                conversation != null -> "Ask a follow-up…"
                card?.sessionId != null -> "Reply…"
                else -> "Ask Nomi anything…"
            },
            palette = palette,
            onSend = { text -> onAsk(text, if (conversation == null) card?.sessionId else null) },
        )
    }
}

@Composable
private fun PanelHeader(
    card: CompanionCard?,
    working: ai.nomi.data.CompanionSignal.Working?,
    conversation: CompanionConversation?,
    palette: CompanionPalette,
    lookAt: Offset?,
    onOpenLink: (String) -> Unit,
    onOpenApp: () -> Unit,
    onOpenConversation: () -> Unit,
    trailing: @Composable () -> Unit,
) {
    val agentKey = working?.agentType ?: card?.agentType
    val title = when {
        conversation != null -> "Asking Nomi"
        card != null -> card.title
        else -> "Nomi"
    }
    val status = when {
        working != null -> formatAgentPhase(working.phase)
        conversation?.answering == true -> "Answering…"
        else -> "Ready when you are"
    }
    Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(RowGap)) {
        AgentShape(
            identity = agentKey?.let(::agentIdentity) ?: NomiIdentity,
            size = HeaderAvatar,
            working = working != null || conversation?.answering == true,
            face = true,
            lookAt = lookAt,
        )
        Column(Modifier.weight(FILL)) {
            Text(
                title,
                style = MaterialTheme.typography.titleSmall.copy(fontWeight = FontWeight(TITLE_WEIGHT)),
                color = palette.content,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            Text(status, style = NomiTheme.metaStyle, color = palette.muted, maxLines = 1, overflow = TextOverflow.Ellipsis)
        }
        NomiButton(
            onClick = {
                when {
                    conversation != null -> onOpenConversation()
                    card != null -> onOpenLink(card.link)
                    else -> onOpenApp()
                }
            },
            variant = NomiButtonVariant.Tonal,
            size = NomiButtonSize.Small,
        ) {
            Text(if (conversation != null || card != null) "Open" else "Open Nomi")
        }
        trailing()
    }
}

@Composable
private fun CardBody(card: CompanionCard, palette: CompanionPalette) {
    val isError = card is CompanionCard.Activity && card.isError
    Text(
        card.body,
        style = MaterialTheme.typography.bodyMedium,
        color = if (isError) palette.error else palette.content,
        maxLines = BODY_MAX_LINES,
        overflow = TextOverflow.Ellipsis,
    )
}

@Composable
private fun ConversationView(conversation: CompanionConversation, palette: CompanionPalette, onNewConversation: () -> Unit) {
    Column(verticalArrangement = Arrangement.spacedBy(RowGap)) {
        conversation.question?.let { question ->
            Box(Modifier.fillMaxWidth(), contentAlignment = Alignment.CenterEnd) {
                Text(
                    question,
                    style = MaterialTheme.typography.bodyMedium,
                    color = palette.onAccent,
                    modifier = Modifier
                        .widthIn(max = QuestionMaxWidth)
                        .background(palette.accent, RoundedCornerShape(BubbleCorner, BubbleCorner, SmallGap, BubbleCorner))
                        .padding(horizontal = RowPaddingH + SmallGap, vertical = RowPaddingV),
                )
            }
        }
        val answer = conversation.answer
        when {
            answer != null -> Text(
                answer,
                style = MaterialTheme.typography.bodyMedium,
                color = palette.content,
                modifier = Modifier.heightIn(max = AnswerMaxHeight).verticalScroll(rememberScrollState()),
            )
            conversation.error != null -> Unit
            conversation.answering -> Text(
                conversation.phase?.let(::formatAgentPhase) ?: "Thinking…",
                style = MaterialTheme.typography.bodyMedium,
                color = palette.muted,
            )
        }
        conversation.error?.let { Text(it, style = MaterialTheme.typography.bodySmall, color = palette.error) }
        if (!conversation.answering) {
            Text(
                "New question",
                style = MaterialTheme.typography.labelLarge.copy(fontWeight = FontWeight.SemiBold),
                color = palette.accent,
                modifier = Modifier
                    .clip(CircleShape)
                    .clickable(role = Role.Button, onClick = onNewConversation)
                    .padding(vertical = SmallGap),
            )
        }
    }
}

@Composable
private fun Section(title: String, palette: CompanionPalette, content: @Composable ColumnScope.() -> Unit) {
    Column(verticalArrangement = Arrangement.spacedBy(SmallGap)) {
        Text(title, style = MaterialTheme.typography.labelMedium.copy(fontWeight = FontWeight.SemiBold), color = palette.muted)
        content()
    }
}

@Composable
private fun CrewSection(crew: List<CompanionCrewMember>, palette: CompanionPalette, onOpenLink: (String) -> Unit) {
    Section("Crew at work", palette) {
        crew.forEach { member ->
            PanelRow(palette, onClick = { onOpenLink("/chat/${member.sessionId}") }) {
                AgentShape(agentIdentity(member.agentType), CrewAvatar, working = true, face = true)
                Column(Modifier.weight(FILL)) {
                    Text(
                        "${member.name} · ${formatAgentPhase(member.phase)}",
                        style = MaterialTheme.typography.bodyMedium.copy(fontWeight = FontWeight.SemiBold),
                        color = palette.content,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                    Text(member.chatTitle, style = NomiTheme.metaStyle, color = palette.muted, maxLines = 1, overflow = TextOverflow.Ellipsis)
                }
            }
        }
    }
}

@Composable
private fun ChatsSection(chats: List<CompanionChat>, palette: CompanionPalette, onOpenLink: (String) -> Unit) {
    Section("Recent chats", palette) {
        chats.forEach { chat ->
            PanelRow(palette, onClick = { onOpenLink("/chat/${chat.id}") }) {
                NomiIcon(NomiIcons.Chat, contentDescription = null, tint = palette.muted, size = CrewAvatar)
                Column(Modifier.weight(FILL)) {
                    Text(
                        chat.title,
                        style = MaterialTheme.typography.bodyMedium.copy(fontWeight = if (chat.unread) FontWeight.Bold else FontWeight.Normal),
                        color = palette.content,
                        maxLines = 1,
                        overflow = TextOverflow.Ellipsis,
                    )
                    val meta = when {
                        chat.working -> "Working…"
                        chat.preview.isNotBlank() -> chat.preview
                        else -> null
                    }
                    meta?.let { Text(it, style = NomiTheme.metaStyle, color = palette.muted, maxLines = 1, overflow = TextOverflow.Ellipsis) }
                }
                if (chat.unread) Text("New", style = NomiTheme.metaStyle.copy(fontWeight = FontWeight.Bold), color = palette.accent)
            }
        }
    }
}

@Composable
private fun ProjectsSection(projects: List<CompanionProject>, palette: CompanionPalette, onOpenLink: (String) -> Unit) {
    Section("Projects", palette) {
        Row(Modifier.horizontalScroll(rememberScrollState()), horizontalArrangement = Arrangement.spacedBy(RowGap)) {
            projects.forEach { project ->
                Row(
                    Modifier
                        .clip(CircleShape)
                        .background(palette.chip)
                        .clickable(role = Role.Button, onClickLabel = "Open ${project.name}") { onOpenLink("/chat/${project.sessionId}") }
                        .padding(horizontal = ChipPaddingH, vertical = ChipPaddingV),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(ChipGap),
                ) {
                    NomiIcon(NomiIcons.Folder, contentDescription = null, tint = palette.muted, size = ChipIcon)
                    Text(project.name, style = MaterialTheme.typography.labelLarge, color = palette.content, maxLines = 1)
                }
            }
        }
    }
}

@Composable
private fun PanelRow(palette: CompanionPalette, onClick: () -> Unit, content: @Composable () -> Unit) {
    Row(
        Modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(BubbleCorner - SmallGap))
            .clickable(role = Role.Button, onClick = onClick)
            .padding(horizontal = RowPaddingH - SmallGap, vertical = RowPaddingV - SmallGap),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(RowContentGap),
    ) { content() }
}

/** A roomy box: a few lines tall as the question grows; Enter sends, Shift+Enter breaks the line. */
@Composable
private fun AskField(placeholder: String, palette: CompanionPalette, onSend: (String) -> Unit) {
    var draft by remember { mutableStateOf("") }
    val send = {
        val text = draft.trim()
        if (text.isNotEmpty()) {
            onSend(text)
            draft = ""
        }
    }
    Row(verticalAlignment = Alignment.Bottom, horizontalArrangement = Arrangement.spacedBy(RowGap)) {
        BasicTextField(
            value = draft,
            onValueChange = { draft = it },
            maxLines = INPUT_MAX_LINES,
            textStyle = MaterialTheme.typography.bodyLarge.copy(color = palette.content),
            cursorBrush = SolidColor(palette.accent),
            keyboardOptions = KeyboardOptions(imeAction = ImeAction.Send),
            keyboardActions = KeyboardActions(onSend = { send() }),
            modifier = Modifier
                .weight(FILL)
                .testTag(COMPANION_INPUT_TAG)
                .onPreviewKeyEvent { event ->
                    if (event.type == KeyEventType.KeyDown && event.key == Key.Enter && !event.isShiftPressed) {
                        send()
                        true
                    } else {
                        false
                    }
                },
            decorationBox = { inner ->
                Box(
                    Modifier
                        .fillMaxWidth()
                        .heightIn(min = InputMinHeight)
                        .clip(RoundedCornerShape(InputMinHeight / 2))
                        .background(palette.field)
                        .padding(horizontal = InputPaddingH, vertical = InputPaddingV),
                    contentAlignment = Alignment.CenterStart,
                ) {
                    if (draft.isEmpty()) Text(placeholder, style = MaterialTheme.typography.bodyLarge, color = palette.muted)
                    inner()
                }
            },
        )
        IconButton(
            onClick = send,
            enabled = draft.isNotBlank(),
            modifier = Modifier.size(SendSize).clip(CircleShape).background(if (draft.isNotBlank()) palette.accent else palette.field),
        ) {
            NomiIcon(NomiIcons.ArrowForward, contentDescription = "Send", tint = if (draft.isNotBlank()) palette.onAccent else palette.muted)
        }
    }
}
