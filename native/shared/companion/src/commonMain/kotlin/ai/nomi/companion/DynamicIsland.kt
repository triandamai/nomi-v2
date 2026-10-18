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
import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.animateContentSize
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
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
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

const val DYNAMIC_ISLAND_TAG = "dynamic-island"

val IslandCollapsedWidth = 172.dp
val IslandWorkingWidth = 248.dp
val IslandExpandedWidth = 360.dp
val IslandCollapsedHeight = 36.dp
val IslandExpandedMinHeight = 132.dp

private val CollapsedCorner = 20.dp
private val ExpandedCorner = 26.dp
private val PillShadow = 12.dp
private val BorderWidth = 1.dp
private const val BORDER_ALPHA = 0.14f
private const val MUTED_TEXT_ALPHA = 0.74f
private const val INPUT_BG_ALPHA = 0.10f
private val CompactAvatarSize = 22.dp
private val ExpandedAvatarSize = 32.dp
private val CompactHorizontalPadding = 12.dp
private val CompactVerticalPadding = 6.dp
private val ExpandedPadding = 16.dp
private val ItemGap = 10.dp
private val SmallGap = 6.dp
private val BadgeHorizontalPadding = 7.dp
private val BadgeVerticalPadding = 2.dp
private val QuickInputHeight = 38.dp
private val QuickInputPadding = 12.dp
private const val FILL = 1f
private const val BRAND_WEIGHT = 800
private const val TITLE_WEIGHT = 700
private const val BODY_MAX_LINES = 3

/**
 * The Dynamic-Island-style companion pill: dark `stage` surface with Nomi's morphing/glancing shape.
 * Expands with M3 Expressive spatial springs on hover, peek, or tap.
 */
@Composable
fun DynamicIslandPill(
    state: CompanionUiState,
    onOpenLink: (String) -> Unit,
    onOpenApp: () -> Unit,
    modifier: Modifier = Modifier,
    lookAt: Offset? = null,
    collapsedWidth: Dp = IslandCollapsedWidth,
    onQuickReply: ((sessionId: String, text: String) -> Unit)? = null,
    onClickCollapsed: (() -> Unit)? = null,
) {
    if (state is CompanionUiState.Snoozed) return

    val extended = NomiTheme.extendedColors
    val isExpanded = state is CompanionUiState.Expanded || state is CompanionUiState.Peeking
    val targetWidth = when (state) {
        is CompanionUiState.Idle -> collapsedWidth
        is CompanionUiState.Working -> maxOf(collapsedWidth, IslandWorkingWidth)
        is CompanionUiState.Peeking, is CompanionUiState.Expanded -> IslandExpandedWidth
        CompanionUiState.Snoozed -> collapsedWidth
    }
    val targetCorner = if (isExpanded) ExpandedCorner else CollapsedCorner
    val width by animateDpAsState(targetWidth, MaterialTheme.motionScheme.defaultSpatialSpec(), label = "island-width")
    val corner by animateDpAsState(targetCorner, MaterialTheme.motionScheme.defaultSpatialSpec(), label = "island-corner")
    val shape = RoundedCornerShape(corner)

    Box(
        modifier
            .testTag(DYNAMIC_ISLAND_TAG)
            .width(width)
            .shadow(PillShadow, shape)
            .clip(shape)
            .background(extended.stage)
            .border(BorderWidth, extended.onStage.copy(alpha = BORDER_ALPHA), shape)
            .animateContentSize(MaterialTheme.motionScheme.defaultSpatialSpec()),
    ) {
        val effectsSpec = MaterialTheme.motionScheme.fastEffectsSpec<Float>()
        AnimatedContent(
            targetState = state,
            transitionSpec = {
                fadeIn(effectsSpec) togetherWith fadeOut(effectsSpec)
            },
            label = "island-content",
        ) { current ->
            when (current) {
                is CompanionUiState.Idle -> CollapsedIdleRow(
                    unreadCount = current.unreadCount,
                    lookAt = lookAt,
                    onClick = onClickCollapsed ?: onOpenApp,
                )
                is CompanionUiState.Working -> CollapsedWorkingRow(
                    agentType = current.agentType,
                    phase = current.phase,
                    unreadCount = current.unreadCount,
                    onClick = onClickCollapsed ?: onOpenApp,
                )
                is CompanionUiState.Peeking -> ExpandedIslandCard(
                    card = current.card,
                    working = null,
                    lookAt = lookAt,
                    onOpenLink = onOpenLink,
                    onOpenApp = onOpenApp,
                    onQuickReply = onQuickReply,
                )
                is CompanionUiState.Expanded -> ExpandedIslandCard(
                    card = current.card,
                    working = current.working,
                    lookAt = lookAt,
                    onOpenLink = onOpenLink,
                    onOpenApp = onOpenApp,
                    onQuickReply = onQuickReply,
                )
                CompanionUiState.Snoozed -> Spacer(Modifier.height(IslandCollapsedHeight))
            }
        }
    }
}

@Composable
private fun CollapsedIdleRow(
    unreadCount: Int,
    lookAt: Offset?,
    onClick: () -> Unit,
) {
    val extended = NomiTheme.extendedColors
    val interaction = remember { MutableInteractionSource() }
    Row(
        Modifier
            .fillMaxWidth()
            .heightIn(min = IslandCollapsedHeight)
            .clickable(interaction, indication = null, onClick = onClick)
            .padding(horizontal = CompactHorizontalPadding, vertical = CompactVerticalPadding),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(ItemGap),
    ) {
        AgentShape(NomiIdentity, CompactAvatarSize, face = true, lookAt = lookAt)
        Text(
            "nomi",
            style = MaterialTheme.typography.titleSmall.copy(
                fontFamily = MaterialTheme.typography.displaySmall.fontFamily,
                fontWeight = FontWeight(BRAND_WEIGHT),
            ),
            color = extended.onStage,
            modifier = Modifier.weight(FILL),
        )
        if (unreadCount > 0) {
            UnreadCountPill(unreadCount)
        }
    }
}

@Composable
private fun CollapsedWorkingRow(
    agentType: String,
    phase: String,
    unreadCount: Int,
    onClick: () -> Unit,
) {
    val extended = NomiTheme.extendedColors
    val identity = agentIdentity(agentType)
    val interaction = remember { MutableInteractionSource() }
    Row(
        Modifier
            .fillMaxWidth()
            .heightIn(min = IslandCollapsedHeight)
            .clickable(interaction, indication = null, onClick = onClick)
            .padding(horizontal = CompactHorizontalPadding, vertical = CompactVerticalPadding),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(ItemGap),
    ) {
        AgentShape(identity, CompactAvatarSize, working = true, face = identity == NomiIdentity)
        Text(
            formatAgentPhase(phase),
            style = MaterialTheme.typography.labelLarge,
            color = extended.onStage,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            modifier = Modifier.weight(FILL),
        )
        if (unreadCount > 0) {
            UnreadCountPill(unreadCount)
        }
    }
}

@Composable
private fun ExpandedIslandCard(
    card: CompanionCard?,
    working: ai.nomi.data.CompanionSignal.Working?,
    lookAt: Offset?,
    onOpenLink: (String) -> Unit,
    onOpenApp: () -> Unit,
    onQuickReply: ((sessionId: String, text: String) -> Unit)?,
) {
    val extended = NomiTheme.extendedColors
    val agentKey = working?.agentType ?: card?.agentType
    val identity = agentKey?.let(::agentIdentity) ?: NomiIdentity
    var replyDraft by remember(card?.sessionId) { mutableStateOf("") }

    Column(
        Modifier
            .fillMaxWidth()
            .heightIn(min = IslandExpandedMinHeight)
            .padding(ExpandedPadding),
        verticalArrangement = Arrangement.spacedBy(ItemGap),
    ) {
        Row(
            Modifier.fillMaxWidth(),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(ItemGap),
        ) {
            AgentShape(
                identity = identity,
                size = ExpandedAvatarSize,
                working = working != null,
                face = identity == NomiIdentity,
                lookAt = lookAt,
            )
            Column(Modifier.weight(FILL)) {
                Text(
                    text = card?.title ?: if (working != null) formatAgentPhase(working.phase) else "Nomi is here",
                    style = MaterialTheme.typography.titleSmall.copy(fontWeight = FontWeight(TITLE_WEIGHT)),
                    color = extended.onStage,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                if (working != null && card != null) {
                    Text(
                        text = formatAgentPhase(working.phase),
                        style = NomiTheme.metaStyle,
                        color = extended.onStage.copy(alpha = MUTED_TEXT_ALPHA),
                        maxLines = 1,
                    )
                }
            }
            NomiButton(
                onClick = { if (card != null) onOpenLink(card.link) else onOpenApp() },
                variant = NomiButtonVariant.Tonal,
                size = NomiButtonSize.Small,
            ) {
                Text(if (card != null) "Open" else "Open Nomi")
            }
        }

        Text(
            text = card?.body ?: "Hover to peek at your crew, or open Nomi to start a conversation.",
            style = MaterialTheme.typography.bodyMedium,
            color = extended.onStage.copy(alpha = MUTED_TEXT_ALPHA),
            maxLines = BODY_MAX_LINES,
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
                    textStyle = MaterialTheme.typography.bodyMedium.copy(color = extended.onStage),
                    cursorBrush = SolidColor(MaterialTheme.colorScheme.primary),
                    keyboardOptions = KeyboardOptions(imeAction = ImeAction.Send),
                    keyboardActions = KeyboardActions(onSend = { sendReply() }),
                    modifier = Modifier.weight(FILL),
                    decorationBox = { inner ->
                        Box(
                            Modifier
                                .fillMaxWidth()
                                .heightIn(min = QuickInputHeight)
                                .clip(CircleShape)
                                .background(extended.onStage.copy(alpha = INPUT_BG_ALPHA))
                                .padding(horizontal = QuickInputPadding),
                            contentAlignment = Alignment.CenterStart,
                        ) {
                            if (replyDraft.isEmpty()) {
                                Text(
                                    "Quick reply…",
                                    style = MaterialTheme.typography.bodyMedium,
                                    color = extended.onStage.copy(alpha = MUTED_TEXT_ALPHA),
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
                    NomiIcon(NomiIcons.ArrowForward, contentDescription = "Send quick reply", tint = extended.onStage)
                }
            }
        }
    }
}

@Composable
internal fun UnreadCountPill(count: Int) {
    val colors = MaterialTheme.colorScheme
    Box(
        Modifier
            .clip(CircleShape)
            .background(colors.primary)
            .padding(horizontal = BadgeHorizontalPadding, vertical = BadgeVerticalPadding),
        contentAlignment = Alignment.Center,
    ) {
        Text(
            text = count.toString(),
            style = NomiTheme.metaStyle.copy(fontWeight = FontWeight(TITLE_WEIGHT)),
            color = colors.onPrimary,
        )
    }
}
