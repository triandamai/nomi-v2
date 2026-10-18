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
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.scaleIn
import androidx.compose.animation.scaleOut
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.detectDragGestures
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp

const val MESSENGER_BUBBLE_TAG = "messenger-bubble"
const val BUBBLE_SPEECH_CARD_TAG = "bubble-speech-card"
const val BUBBLE_QUICK_CHAT_TAG = "bubble-quick-chat"

val BubbleSizeDp = 60.dp
val DismissTargetSizeDp = 56.dp

private val SpeechCardMaxWidth = 264.dp
private val QuickChatWidth = 312.dp
private val BubbleShadow = 10.dp
private val CardShadow = 14.dp
private val CardCorner = 22.dp
private val CardPadding = 14.dp
private val SpeechPadding = 12.dp
private val RowGap = 10.dp
private val SmallGap = 6.dp
private val BorderWidth = 1.dp
private val QuickInputHeight = 40.dp
private val QuickInputPadding = 12.dp
private const val FILL = 1f
private const val TITLE_WEIGHT = 700
private const val PREVIEW_MAX_LINES = 2
private const val EXPANDED_MAX_LINES = 4
private const val CAPTURED_SCALE = 1.18f
private const val NORMAL_SCALE = 1f

/**
 * Messenger-style floating chat-head bubble (`AgentShape` 60dp with Nomi's eyes, unread badge, beside-the-head
 * speech peek card, and expandable quick-chat overlay).
 *
 * Used both by Android's `WindowManager` overlay (`BubbleOverlayController`) and as an in-app floating companion.
 */
@Composable
fun MessengerBubble(
    state: CompanionUiState,
    onTapHead: () -> Unit,
    onOpenLink: (String) -> Unit,
    onOpenApp: () -> Unit,
    modifier: Modifier = Modifier,
    dockedRight: Boolean = true,
    lookAt: Offset? = null,
    onDragStart: () -> Unit = {},
    onDragBy: (Offset) -> Unit = {},
    onDragEnd: () -> Unit = {},
    onQuickReply: ((sessionId: String, text: String) -> Unit)? = null,
    onDismiss: (() -> Unit)? = null,
) {
    if (state is CompanionUiState.Snoozed) return

    val workingSignal = when (state) {
        is CompanionUiState.Working -> state
        is CompanionUiState.Expanded -> state.working?.let { CompanionUiState.Working(it.agentType, it.phase) }
        else -> null
    }
    val card = when (state) {
        is CompanionUiState.Peeking -> state.card
        is CompanionUiState.Expanded -> state.card
        else -> null
    }
    val unreadCount = when (state) {
        is CompanionUiState.Idle -> state.unreadCount
        is CompanionUiState.Working -> state.unreadCount
        is CompanionUiState.Peeking -> state.unreadCount
        is CompanionUiState.Expanded -> state.unreadCount
        CompanionUiState.Snoozed -> 0
    }
    val agentKey = workingSignal?.agentType ?: card?.agentType
    val identity = agentKey?.let(::agentIdentity) ?: NomiIdentity

    Column(
        modifier = modifier.testTag(MESSENGER_BUBBLE_TAG),
        horizontalAlignment = if (dockedRight) Alignment.End else Alignment.Start,
        verticalArrangement = Arrangement.spacedBy(RowGap),
    ) {
        Row(
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(RowGap),
        ) {
            if (dockedRight) {
                AnimatedVisibility(
                    visible = state is CompanionUiState.Peeking && card != null,
                    enter = fadeIn(MaterialTheme.motionScheme.fastEffectsSpec()) + scaleIn(MaterialTheme.motionScheme.defaultSpatialSpec()),
                    exit = fadeOut(MaterialTheme.motionScheme.fastEffectsSpec()) + scaleOut(MaterialTheme.motionScheme.defaultSpatialSpec()),
                ) {
                    card?.let {
                        BubbleSpeechCard(
                            card = it,
                            onClick = onTapHead,
                        )
                    }
                }
            }

            BubbleHead(
                identity = identity,
                working = workingSignal != null,
                unreadCount = unreadCount,
                lookAt = lookAt,
                onClick = onTapHead,
                onDragStart = onDragStart,
                onDragBy = onDragBy,
                onDragEnd = onDragEnd,
            )

            if (!dockedRight) {
                AnimatedVisibility(
                    visible = state is CompanionUiState.Peeking && card != null,
                    enter = fadeIn(MaterialTheme.motionScheme.fastEffectsSpec()) + scaleIn(MaterialTheme.motionScheme.defaultSpatialSpec()),
                    exit = fadeOut(MaterialTheme.motionScheme.fastEffectsSpec()) + scaleOut(MaterialTheme.motionScheme.defaultSpatialSpec()),
                ) {
                    card?.let {
                        BubbleSpeechCard(
                            card = it,
                            onClick = onTapHead,
                        )
                    }
                }
            }
        }

        AnimatedVisibility(
            visible = state is CompanionUiState.Expanded,
            enter = fadeIn(MaterialTheme.motionScheme.fastEffectsSpec()) + scaleIn(MaterialTheme.motionScheme.defaultSpatialSpec()),
            exit = fadeOut(MaterialTheme.motionScheme.fastEffectsSpec()) + scaleOut(MaterialTheme.motionScheme.defaultSpatialSpec()),
        ) {
            BubbleQuickChatCard(
                card = card,
                working = workingSignal,
                onOpenLink = onOpenLink,
                onOpenApp = onOpenApp,
                onClose = onTapHead,
                onDismiss = onDismiss,
                onQuickReply = onQuickReply,
            )
        }
    }
}

@Composable
private fun BubbleHead(
    identity: ai.nomi.designsystem.shape.AgentIdentity,
    working: Boolean,
    unreadCount: Int,
    lookAt: Offset?,
    onClick: () -> Unit,
    onDragStart: () -> Unit,
    onDragBy: (Offset) -> Unit,
    onDragEnd: () -> Unit,
) {
    val interaction = remember { MutableInteractionSource() }
    Box(
        Modifier
            .size(BubbleSizeDp)
            .pointerInput(Unit) {
                detectDragGestures(
                    onDragStart = { onDragStart() },
                    onDragEnd = { onDragEnd() },
                    onDragCancel = { onDragEnd() },
                    onDrag = { change, dragAmount ->
                        change.consume()
                        onDragBy(dragAmount)
                    },
                )
            }
            .clickable(interaction, indication = null, onClick = onClick),
        contentAlignment = Alignment.Center,
    ) {
        Box(Modifier.shadow(BubbleShadow, CircleShape)) {
            AgentShape(
                identity = identity,
                size = BubbleSizeDp,
                working = working,
                face = true,
                lookAt = lookAt,
            )
        }
        if (unreadCount > 0) {
            Box(Modifier.align(Alignment.TopEnd)) {
                UnreadCountPill(unreadCount)
            }
        }
    }
}

/** Speech-bubble card that pops out beside the head during [CompanionUiState.Peeking]. */
@Composable
fun BubbleSpeechCard(
    card: CompanionCard,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val colors = MaterialTheme.colorScheme
    val shape = RoundedCornerShape(CardCorner)
    Column(
        modifier
            .testTag(BUBBLE_SPEECH_CARD_TAG)
            .widthIn(max = SpeechCardMaxWidth)
            .shadow(CardShadow, shape)
            .clip(shape)
            .background(colors.surfaceContainerLowest)
            .border(BorderWidth, colors.outlineVariant, shape)
            .clickable(onClick = onClick)
            .padding(SpeechPadding),
        verticalArrangement = Arrangement.spacedBy(SmallGap),
    ) {
        Text(
            text = card.title,
            style = MaterialTheme.typography.labelLarge.copy(fontWeight = FontWeight(TITLE_WEIGHT)),
            color = colors.onSurface,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
        Text(
            text = card.body,
            style = MaterialTheme.typography.bodySmall,
            color = colors.onSurfaceVariant,
            maxLines = PREVIEW_MAX_LINES,
            overflow = TextOverflow.Ellipsis,
        )
    }
}

/** Compact chat & status panel shown when the Messenger bubble is tapped open. */
@Composable
fun BubbleQuickChatCard(
    card: CompanionCard?,
    working: CompanionUiState.Working?,
    onOpenLink: (String) -> Unit,
    onOpenApp: () -> Unit,
    onClose: () -> Unit,
    modifier: Modifier = Modifier,
    onDismiss: (() -> Unit)? = null,
    onQuickReply: ((sessionId: String, text: String) -> Unit)? = null,
) {
    val colors = MaterialTheme.colorScheme
    val shape = RoundedCornerShape(CardCorner)
    var replyDraft by remember(card?.sessionId) { mutableStateOf("") }

    Column(
        modifier
            .testTag(BUBBLE_QUICK_CHAT_TAG)
            .widthIn(max = QuickChatWidth)
            .shadow(CardShadow, shape)
            .clip(shape)
            .background(colors.surfaceContainerLowest)
            .border(BorderWidth, colors.outlineVariant, shape)
            .padding(CardPadding),
        verticalArrangement = Arrangement.spacedBy(RowGap),
    ) {
        Row(
            Modifier.fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(SmallGap),
        ) {
            Column(Modifier.weight(FILL)) {
                Text(
                    text = card?.title ?: if (working != null) formatAgentPhase(working.phase) else "Nomi",
                    style = MaterialTheme.typography.titleSmall.copy(fontWeight = FontWeight(TITLE_WEIGHT)),
                    color = colors.onSurface,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                if (working != null && card != null) {
                    Text(
                        text = formatAgentPhase(working.phase),
                        style = NomiTheme.metaStyle,
                        color = colors.onSurfaceVariant,
                    )
                }
            }
            if (onDismiss != null) {
                IconButton(onClick = onDismiss) {
                    NomiIcon(NomiIcons.Close, contentDescription = "Hide bubble", tint = colors.onSurfaceVariant)
                }
            } else {
                IconButton(onClick = onClose) {
                    NomiIcon(NomiIcons.Close, contentDescription = "Collapse", tint = colors.onSurfaceVariant)
                }
            }
        }

        Text(
            text = card?.body ?: "Tap Open in Nomi to jump into your conversation.",
            style = MaterialTheme.typography.bodyMedium,
            color = colors.onSurfaceVariant,
            maxLines = EXPANDED_MAX_LINES,
            overflow = TextOverflow.Ellipsis,
        )

        val sessionId = card?.sessionId
        if (sessionId != null && onQuickReply != null) {
            val sendReply = {
                val trimmed = replyDraft.trim()
                if (trimmed.isNotEmpty()) {
                    onQuickReply(sessionId, trimmed)
                    replyDraft = ""
                }
            }
            Row(
                Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(SmallGap),
            ) {
                BasicTextField(
                    value = replyDraft,
                    onValueChange = { replyDraft = it },
                    singleLine = true,
                    textStyle = MaterialTheme.typography.bodyMedium.copy(color = colors.onSurface),
                    cursorBrush = SolidColor(colors.primary),
                    keyboardOptions = KeyboardOptions(imeAction = ImeAction.Send),
                    keyboardActions = KeyboardActions(onSend = { sendReply() }),
                    modifier = Modifier.weight(FILL),
                    decorationBox = { inner ->
                        Box(
                            Modifier
                                .fillMaxWidth()
                                .heightIn(min = QuickInputHeight)
                                .clip(CircleShape)
                                .background(colors.surfaceContainerHigh)
                                .padding(horizontal = QuickInputPadding),
                            contentAlignment = Alignment.CenterStart,
                        ) {
                            if (replyDraft.isEmpty()) {
                                Text(
                                    "Reply…",
                                    style = MaterialTheme.typography.bodyMedium,
                                    color = colors.onSurfaceVariant,
                                )
                            }
                            inner()
                        }
                    },
                )
                IconButton(
                    onClick = sendReply,
                    enabled = replyDraft.isNotBlank(),
                ) {
                    NomiIcon(NomiIcons.ArrowForward, contentDescription = "Send reply", tint = colors.primary)
                }
            }
        }

        Row(
            Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.End,
        ) {
            NomiButton(
                onClick = { if (card != null) onOpenLink(card.link) else onOpenApp() },
                variant = NomiButtonVariant.Tonal,
                size = NomiButtonSize.Small,
            ) {
                Text("Open in Nomi")
            }
        }
    }
}

/** Bottom-center dismiss target shown while dragging the Messenger bubble on Android. */
@Composable
fun BubbleDismissTarget(
    captured: Boolean,
    modifier: Modifier = Modifier,
) {
    val colors = MaterialTheme.colorScheme
    val scale by animateFloatAsState(
        if (captured) CAPTURED_SCALE else NORMAL_SCALE,
        MaterialTheme.motionScheme.defaultSpatialSpec(),
        label = "dismiss-scale",
    )
    Box(
        modifier
            .size(DismissTargetSizeDp)
            .graphicsLayer {
                scaleX = scale
                scaleY = scale
            }
            .shadow(BubbleShadow, CircleShape)
            .clip(CircleShape)
            .background(if (captured) colors.errorContainer else colors.inverseSurface),
        contentAlignment = Alignment.Center,
    ) {
        NomiIcon(
            NomiIcons.Close,
            contentDescription = "Dismiss bubble",
            tint = if (captured) colors.onErrorContainer else colors.inverseOnSurface,
        )
    }
}
