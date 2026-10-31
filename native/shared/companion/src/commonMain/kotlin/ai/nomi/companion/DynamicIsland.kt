package ai.nomi.companion

import ai.nomi.designsystem.component.NomiButton
import ai.nomi.designsystem.component.NomiButtonSize
import ai.nomi.designsystem.component.NomiButtonVariant
import ai.nomi.designsystem.shape.AgentShape
import ai.nomi.designsystem.shape.NomiIdentity
import ai.nomi.designsystem.shape.agentIdentity
import ai.nomi.designsystem.theme.NomiTheme
import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.wrapContentSize
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.setValue
import androidx.compose.runtime.SideEffect
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.TransformOrigin
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.input.pointer.PointerEventType
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

const val DYNAMIC_ISLAND_TAG = "dynamic-island"

val IslandCollapsedWidth = 188.dp
val IslandWorkingWidth = 248.dp
/** A peek is wide enough for a reply's first lines; the open panel for the crew, chats and an answer. */
val IslandPeekWidth = 420.dp
val IslandExpandedWidth = 460.dp
val IslandCollapsedHeight = 32.dp
val IslandWorkingHeight = 36.dp
val IslandPeekHeight = 96.dp
val IslandExpandedHeight = 440.dp
/** A notched Mac's camera housing, when the screen doesn't say how tall its menu bar is. */
val DefaultNotchHeight = 38.dp

private const val NOTCH_BLACK_ARGB = 0xFF000000L
private const val NOTCH_FG_ARGB = 0xFFF4F7F5L
private val NotchBlack = Color(NOTCH_BLACK_ARGB)
private val NotchForeground = Color(NOTCH_FG_ARGB)

private const val NOTCH_ZOOM_DURATION_MS = 260
private const val ORIGIN_CENTER_X = 0.5f
private const val ORIGIN_TOP_Y = 0f
private const val MIN_ZOOM_SCALE = 0.52f
private const val FULL_ZOOM_SCALE = 1f
private const val HIDDEN_ALPHA = 0f
private const val VISIBLE_ALPHA = 1f
private const val ACTIVE_VISIBILITY_THRESHOLD = 0.02f

private val CollapsedCorner = 14.dp
private val ExpandedCorner = 22.dp
private val ZeroCorner = 0.dp
private val PillShadow = 12.dp
private val BorderWidth = 1.dp
private const val BORDER_ALPHA = 0.14f
private const val MUTED_TEXT_ALPHA = 0.74f
private const val INPUT_BG_ALPHA = 0.14f
private val CompactAvatarSize = 20.dp
private val ExpandedAvatarSize = 28.dp
private val CompactHorizontalPadding = 12.dp
private val CompactVerticalPadding = 4.dp
private val ExpandedPadding = 12.dp
private val BelowNotchGap = 8.dp
private val ItemGap = 8.dp
private val SmallGap = 6.dp
private val BadgeHorizontalPadding = 7.dp
private val BadgeVerticalPadding = 2.dp
private const val FILL = 1f
private const val BRAND_WEIGHT = 800
private const val TITLE_WEIGHT = 700
private const val PEEK_BODY_MAX_LINES = 2

/**
 * The Dynamic-Island-style companion pill: pure black notch surface that stays smaller than the physical Mac notch
 * when idle, and zooms both width and height simultaneously with a synchronized tween when peeking, hovering, or hiding.
 */
@Composable
fun DynamicIslandPill(
    state: CompanionUiState,
    onOpenLink: (String) -> Unit,
    onOpenApp: () -> Unit,
    modifier: Modifier = Modifier,
    lookAt: Offset? = null,
    collapsedWidth: Dp = IslandCollapsedWidth,
    notchMode: Boolean = false,
    onHoverChange: ((Boolean) -> Unit)? = null,
    onClickCollapsed: (() -> Unit)? = null,
    overview: CompanionOverview = CompanionOverview(),
    onAsk: ((text: String, sessionId: String?) -> Unit)? = null,
    onNewConversation: () -> Unit = {},
    onOpenConversation: () -> Unit = onOpenApp,
    /** How tall the hardware notch is: in [notchMode] content starts below it, never under the camera. */
    notchHeight: Dp = DefaultNotchHeight,
) {
    if (state is CompanionUiState.Snoozed) return

    // Remember the last active (non-Idle) state so when hiding back to Idle, the card stays mounted and zooms down
    // simultaneously with the notch width and height instead of disappearing or holding the height open.
    var lastActiveState by remember { mutableStateOf<CompanionUiState?>(null) }
    SideEffect {
        if (state !is CompanionUiState.Idle) {
            lastActiveState = state
        }
    }

    val isExpanded = state is CompanionUiState.Expanded || state is CompanionUiState.Peeking
    val isActiveOrHovered = isExpanded || state is CompanionUiState.Working

    val clearance = if (notchMode) notchHeight else 0.dp
    val peekFullHeight = IslandPeekHeight + clearance
    val expandedFullHeight = IslandExpandedHeight + clearance
    val workingFullHeight = if (notchMode) maxOf(notchHeight, IslandWorkingHeight) else IslandWorkingHeight

    val targetWidth = when (state) {
        is CompanionUiState.Idle -> collapsedWidth
        is CompanionUiState.Working -> maxOf(collapsedWidth, IslandWorkingWidth)
        is CompanionUiState.Peeking -> maxOf(collapsedWidth, IslandPeekWidth)
        is CompanionUiState.Expanded -> maxOf(collapsedWidth, IslandExpandedWidth)
        CompanionUiState.Snoozed -> collapsedWidth
    }
    val targetHeight = when (state) {
        is CompanionUiState.Idle -> IslandCollapsedHeight
        is CompanionUiState.Working -> workingFullHeight
        is CompanionUiState.Peeking -> peekFullHeight
        is CompanionUiState.Expanded -> expandedFullHeight
        CompanionUiState.Snoozed -> IslandCollapsedHeight
    }
    val targetCorner = if (isExpanded) ExpandedCorner else CollapsedCorner

    val zoomDpSpec = tween<Dp>(durationMillis = NOTCH_ZOOM_DURATION_MS, easing = FastOutSlowInEasing)
    val zoomFloatSpec = tween<Float>(durationMillis = NOTCH_ZOOM_DURATION_MS, easing = FastOutSlowInEasing)

    val width by animateDpAsState(targetValue = targetWidth, animationSpec = zoomDpSpec, label = "island-width")
    val height by animateDpAsState(targetValue = targetHeight, animationSpec = zoomDpSpec, label = "island-height")
    val corner by animateDpAsState(targetValue = targetCorner, animationSpec = zoomDpSpec, label = "island-corner")
    val activeAlpha by animateFloatAsState(
        targetValue = if (isActiveOrHovered) VISIBLE_ALPHA else HIDDEN_ALPHA,
        animationSpec = zoomFloatSpec,
        label = "island-active-alpha",
    )

    val shape = if (notchMode) {
        RoundedCornerShape(topStart = ZeroCorner, topEnd = ZeroCorner, bottomStart = corner, bottomEnd = corner)
    } else {
        RoundedCornerShape(corner)
    }

    val hoverModifier = if (onHoverChange != null) {
        Modifier.pointerInput(onHoverChange) {
            awaitPointerEventScope {
                while (true) {
                    val event = awaitPointerEvent()
                    when (event.type) {
                        PointerEventType.Enter, PointerEventType.Move -> onHoverChange(true)
                        PointerEventType.Exit -> onHoverChange(false)
                    }
                }
            }
        }
    } else {
        Modifier
    }

    val stillZoomingOut = activeAlpha > ACTIVE_VISIBILITY_THRESHOLD
    val showSurfaceChrome = !notchMode || isActiveOrHovered || stillZoomingOut
    val surfaceModifier = if (showSurfaceChrome) {
        Modifier
            .shadow(PillShadow, shape)
            .clip(shape)
            .background(NotchBlack)
            .let { base ->
                if (notchMode) base else base.border(BorderWidth, NotchForeground.copy(alpha = BORDER_ALPHA), shape)
            }
    } else {
        Modifier
            .clip(shape)
            .background(NotchBlack)
    }

    val displayState = if (isActiveOrHovered) {
        state
    } else if (stillZoomingOut && lastActiveState != null) {
        lastActiveState!!
    } else {
        state
    }

    val contentTargetWidth = when (displayState) {
        is CompanionUiState.Working -> maxOf(collapsedWidth, IslandWorkingWidth)
        is CompanionUiState.Peeking -> maxOf(collapsedWidth, IslandPeekWidth)
        is CompanionUiState.Expanded -> maxOf(collapsedWidth, IslandExpandedWidth)
        else -> collapsedWidth
    }
    val contentTargetHeight = when (displayState) {
        is CompanionUiState.Working -> workingFullHeight
        is CompanionUiState.Peeking -> peekFullHeight
        is CompanionUiState.Expanded -> expandedFullHeight
        else -> IslandCollapsedHeight
    }

    val scaleX = (width / contentTargetWidth).coerceIn(MIN_ZOOM_SCALE, FULL_ZOOM_SCALE)
    val scaleY = (height / contentTargetHeight).coerceIn(MIN_ZOOM_SCALE, FULL_ZOOM_SCALE)

    Box(
        modifier
            .testTag(DYNAMIC_ISLAND_TAG)
            .size(width = width, height = height)
            .then(surfaceModifier)
            .then(hoverModifier),
        contentAlignment = Alignment.TopCenter,
    ) {
        when (displayState) {
            is CompanionUiState.Idle -> CollapsedIdleRow(
                unreadCount = displayState.unreadCount,
                lookAt = lookAt,
                notchMode = notchMode,
                onClick = onClickCollapsed ?: onOpenApp,
            )
            is CompanionUiState.Working -> Box(
                Modifier
                    .wrapContentSize(align = Alignment.TopCenter, unbounded = true)
                    .size(width = contentTargetWidth, height = contentTargetHeight)
                    .graphicsLayer {
                        transformOrigin = TransformOrigin(ORIGIN_CENTER_X, ORIGIN_TOP_Y)
                        this.scaleX = scaleX
                        this.scaleY = scaleY
                        this.alpha = activeAlpha
                    },
                contentAlignment = Alignment.BottomCenter,
            ) {
                CollapsedWorkingRow(
                    agentType = displayState.agentType,
                    phase = displayState.phase,
                    unreadCount = displayState.unreadCount,
                    lookAt = lookAt,
                    onClick = onClickCollapsed ?: onOpenApp,
                )
            }
            is CompanionUiState.Peeking -> Box(
                Modifier
                    .wrapContentSize(align = Alignment.TopCenter, unbounded = true)
                    .size(width = contentTargetWidth, height = contentTargetHeight)
                    .graphicsLayer {
                        transformOrigin = TransformOrigin(ORIGIN_CENTER_X, ORIGIN_TOP_Y)
                        this.scaleX = scaleX
                        this.scaleY = scaleY
                        this.alpha = activeAlpha
                    },
                contentAlignment = Alignment.TopCenter,
            ) {
                PeekIslandCard(
                    card = displayState.card,
                    lookAt = lookAt,
                    topPadding = ExpandedPadding + if (notchMode) clearance else 0.dp,
                    onOpenLink = onOpenLink,
                )
            }
            is CompanionUiState.Expanded -> Box(
                Modifier
                    .wrapContentSize(align = Alignment.TopCenter, unbounded = true)
                    .size(width = contentTargetWidth, height = contentTargetHeight)
                    .graphicsLayer {
                        transformOrigin = TransformOrigin(ORIGIN_CENTER_X, ORIGIN_TOP_Y)
                        this.scaleX = scaleX
                        this.scaleY = scaleY
                        this.alpha = activeAlpha
                    },
                contentAlignment = Alignment.TopCenter,
            ) {
                CompanionPanel(
                    overview = overview,
                    card = displayState.card,
                    working = displayState.working,
                    palette = notchPalette(),
                    onAsk = onAsk ?: { _, _ -> },
                    onOpenLink = onOpenLink,
                    onOpenApp = onOpenApp,
                    onNewConversation = onNewConversation,
                    onOpenConversation = onOpenConversation,
                    lookAt = lookAt,
                    fillHeight = true,
                    modifier = Modifier
                        .fillMaxSize()
                        .padding(
                            start = ExpandedPadding,
                            end = ExpandedPadding,
                            top = ExpandedPadding + if (notchMode) clearance + BelowNotchGap else 0.dp,
                            bottom = ExpandedPadding,
                        ),
                )
            }
            CompanionUiState.Snoozed -> Spacer(Modifier.height(IslandCollapsedHeight))
        }
    }
}

@Composable
private fun CollapsedIdleRow(
    unreadCount: Int,
    lookAt: Offset?,
    notchMode: Boolean,
    onClick: () -> Unit,
) {
    val interaction = remember { MutableInteractionSource() }
    if (notchMode) {
        // When idle inside the Mac notch, stay tucked inside the black notch area and only expand on hover/click.
        Box(
            Modifier
                .fillMaxWidth()
                .height(IslandCollapsedHeight)
                .clickable(interaction, indication = null, onClick = onClick),
        )
        return
    }
    Row(
        Modifier
            .fillMaxWidth()
            .heightIn(min = IslandCollapsedHeight)
            .clickable(interaction, indication = null, onClick = onClick)
            .padding(horizontal = CompactHorizontalPadding, vertical = CompactVerticalPadding),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(ItemGap),
    ) {
        AgentShape(
            identity = NomiIdentity,
            size = CompactAvatarSize,
            working = false,
            face = true,
            lookAt = lookAt,
        )
        Text(
            "nomi",
            style = MaterialTheme.typography.titleSmall.copy(
                fontFamily = MaterialTheme.typography.displaySmall.fontFamily,
                fontWeight = FontWeight(BRAND_WEIGHT),
            ),
            color = NotchForeground,
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
    lookAt: Offset?,
    onClick: () -> Unit,
) {
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
        AgentShape(
            identity = identity,
            size = CompactAvatarSize,
            working = true,
            face = true,
            lookAt = lookAt,
        )
        Text(
            formatAgentPhase(phase),
            style = MaterialTheme.typography.labelLarge,
            color = NotchForeground,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            modifier = Modifier.weight(FILL),
        )
        if (unreadCount > 0) {
            UnreadCountPill(unreadCount)
        }
    }
}

/** A peek: who it's from, its first lines, and a way in. */
@Composable
private fun PeekIslandCard(
    card: CompanionCard,
    lookAt: Offset?,
    topPadding: Dp,
    onOpenLink: (String) -> Unit,
) {
    val identity = card.agentType?.let(::agentIdentity) ?: NomiIdentity
    Row(
        Modifier
            .fillMaxWidth()
            .padding(start = ExpandedPadding, end = ExpandedPadding, top = topPadding, bottom = ExpandedPadding),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(ItemGap),
    ) {
        AgentShape(identity = identity, size = ExpandedAvatarSize, working = true, face = true, lookAt = lookAt)
        Column(Modifier.weight(FILL)) {
            Text(
                card.title,
                style = MaterialTheme.typography.titleSmall.copy(fontWeight = FontWeight(TITLE_WEIGHT)),
                color = NotchForeground,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            Text(
                card.body,
                style = MaterialTheme.typography.bodyMedium,
                color = NotchForeground.copy(alpha = MUTED_TEXT_ALPHA),
                maxLines = PEEK_BODY_MAX_LINES,
                overflow = TextOverflow.Ellipsis,
            )
        }
        NomiButton(onClick = { onOpenLink(card.link) }, variant = NomiButtonVariant.Tonal, size = NomiButtonSize.Small) {
            Text("Open")
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
