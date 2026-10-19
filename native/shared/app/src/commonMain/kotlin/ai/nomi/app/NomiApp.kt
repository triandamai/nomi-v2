package ai.nomi.app

import ai.nomi.app.nav.Chat
import ai.nomi.app.nav.Chats
import ai.nomi.app.nav.Home
import ai.nomi.app.nav.NomiNavDisplay
import ai.nomi.app.nav.NomiRoute
import ai.nomi.app.nav.Notifications
import ai.nomi.app.nav.Settings
import ai.nomi.app.nav.SignIn
import ai.nomi.app.nav.TopLevelRoutes
import ai.nomi.app.nav.currentTopLevel
import ai.nomi.app.nav.isWideWindow
import ai.nomi.app.nav.open
import ai.nomi.app.nav.openChat
import ai.nomi.app.nav.resetTo
import ai.nomi.app.nav.selectTopLevel
import ai.nomi.companion.BubbleDismissTarget
import ai.nomi.companion.BubblePointPx
import ai.nomi.companion.BubbleSizeDp
import ai.nomi.companion.CompanionController
import ai.nomi.companion.CompanionUiState
import ai.nomi.companion.MessengerBubble
import ai.nomi.companion.ScreenAreaPx
import ai.nomi.companion.clampBubblePosition
import ai.nomi.companion.isInsideDismissTarget
import ai.nomi.companion.resolveOverlayWindowPosition
import ai.nomi.companion.snapToEdge
import ai.nomi.data.AppearanceRepository
import ai.nomi.data.CompanionSignal
import ai.nomi.data.NotificationRepository
import ai.nomi.data.RealtimeSync
import ai.nomi.data.SessionGate
import ai.nomi.data.THEME_DARK
import ai.nomi.data.THEME_LIGHT
import ai.nomi.designsystem.component.ComposerPlatform
import ai.nomi.designsystem.component.LocalComposerPlatform
import ai.nomi.designsystem.component.NomiLoadingIndicator
import ai.nomi.designsystem.component.NomiSnackbarHost
import ai.nomi.designsystem.icon.NomiIcons
import ai.nomi.designsystem.navigation.NomiNavItem
import ai.nomi.designsystem.navigation.NomiNavigationBar
import ai.nomi.designsystem.navigation.NomiSidebar
import ai.nomi.designsystem.theme.NomiTheme
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.snap
import androidx.compose.animation.core.spring
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.scaleIn
import androidx.compose.animation.scaleOut
import androidx.compose.foundation.background
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.consumeWindowInsets
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SnackbarDuration
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.SnackbarResult
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.IntSize
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import org.koin.compose.koinInject
import kotlin.math.abs
import kotlin.math.roundToInt

private const val FILL = 1f
private const val HALF = 2
private const val INITIAL_BUBBLE_TOP_FRACTION = 0.28f
private const val LIQUID_SNAP_DAMPING = 0.72f
private const val LIQUID_SNAP_STIFFNESS = 380f
private const val LIQUID_STRETCH_DIVISOR = 2.2f
private const val LIQUID_MAX_STRETCH = 0.22f
private const val LIQUID_SQUISH_FACTOR = 0.55f
private const val MIN_STRETCH = 0f
private val SnackbarPadding = 16.dp
private val BubbleEdgeMargin = 12.dp
private val BubbleTopInset = 56.dp
private val BubbleBottomInset = 24.dp
private val DismissBottomOffset = 36.dp
private val DismissCaptureRadius = 72.dp

private fun NomiRoute.navItem(unread: Int): NomiNavItem<NomiRoute>? = when (this) {
    Home -> NomiNavItem(this, "Home", NomiIcons.Home)
    Chats -> NomiNavItem(this, "Chats", NomiIcons.Chat)
    Notifications -> NomiNavItem(this, "Inbox", NomiIcons.Bell, badge = unread)
    Settings -> NomiNavItem(this, "Settings", NomiIcons.Settings)
    else -> null
}

/**
 * Root of the Nomi UI, hosted by the Android, iOS and desktop shells (after `initKoin`). Routes on
 * [SessionGate.signedIn]: sign-in, or the signed-in shell (navigation + [NomiNavDisplay]).
 */
@Composable
fun NomiApp(reducedMotion: Boolean = rememberReducedMotion()) {
    val gate = koinInject<SessionGate>()
    val appearance = koinInject<AppearanceRepository>()
    val restored by gate.restored.collectAsStateWithLifecycle()
    val signedIn by gate.signedIn.collectAsStateWithLifecycle()
    val themeMode by appearance.theme.collectAsStateWithLifecycle()
    val accentColor by appearance.accentColor.collectAsStateWithLifecycle()
    val systemDark = isSystemInDarkTheme()
    val dark = when (themeMode) {
        THEME_LIGHT -> false
        THEME_DARK -> true
        else -> systemDark
    }
    LaunchedEffect(gate) { gate.restore() }
    LaunchedEffect(signedIn, appearance) {
        if (signedIn) appearance.refresh()
    }

    NomiTheme(accent = appearance.paletteKey(accentColor), dark = dark, reducedMotion = reducedMotion) {
        if (!restored) {
            Box(Modifier.fillMaxSize().background(NomiTheme.extendedColors.ground), contentAlignment = Alignment.Center) {
                NomiLoadingIndicator(contentDescription = "Loading")
            }
        } else {
            NomiShell(signedIn)
        }
    }
}

@Composable
private fun NomiShell(signedIn: Boolean) {
    val backStack = remember { mutableStateListOf<NomiRoute>(if (signedIn) Home else SignIn) }
    LaunchedEffect(signedIn) {
        val onSignIn = backStack.lastOrNull() == SignIn
        if (signedIn && onSignIn) backStack.resetTo(Home)
        if (!signedIn && !onSignIn) backStack.resetTo(SignIn)
    }
    val wide = isWideWindow()
    val inApp = signedIn && backStack.lastOrNull() != SignIn
    val unread by koinInject<NotificationRepository>().observeUnreadCount().collectAsStateWithLifecycle(0)
    val sync = koinInject<RealtimeSync>()
    val companion = koinInject<CompanionController>()
    val companionState by companion.uiState.collectAsStateWithLifecycle()
    val bubbleEnabled by companion.bubbleEnabled.collectAsStateWithLifecycle()
    val snackbarHostState = remember { SnackbarHostState() }
    val platform = LocalComposerPlatform.current

    var dismissedInApp by remember { mutableStateOf(false) }
    LaunchedEffect(bubbleEnabled) {
        if (bubbleEnabled) dismissedInApp = false
    }
    LaunchedEffect(companionState) {
        if (companionState is CompanionUiState.Peeking) {
            dismissedInApp = false
        }
    }

    LaunchedEffect(inApp, companion) {
        if (!inApp) return@LaunchedEffect
        companion.navigationRequests.collect { link ->
            val route = NomiRoute.fromLink(link) ?: Home
            backStack.open(route)
        }
    }

    LaunchedEffect(inApp, sync) {
        if (!inApp) return@LaunchedEffect
        sync.companionSignals.collect { signal ->
            when (signal) {
                is CompanionSignal.NewReply -> {
                    val result = snackbarHostState.showSnackbar(
                        message = signal.preview.ifBlank { "New reply" },
                        actionLabel = "View",
                        duration = SnackbarDuration.Short,
                    )
                    if (result == SnackbarResult.ActionPerformed) backStack.openChat(signal.sessionId)
                }
                is CompanionSignal.Notification -> {
                    if (backStack.currentTopLevel != Notifications) {
                        val result = snackbarHostState.showSnackbar(
                            message = signal.title,
                            actionLabel = "View",
                            duration = SnackbarDuration.Short,
                        )
                        if (result == SnackbarResult.ActionPerformed) {
                            val target = signal.link?.let(NomiRoute::fromLink)
                            if (target != null) backStack.open(target) else backStack.selectTopLevel(Notifications)
                        }
                    }
                }
                is CompanionSignal.NeedsYou -> {
                    val result = snackbarHostState.showSnackbar(
                        message = signal.title?.takeIf { it.isNotBlank() } ?: "A chat needs your input",
                        actionLabel = "View",
                        duration = SnackbarDuration.Short,
                    )
                    if (result == SnackbarResult.ActionPerformed) backStack.openChat(signal.sessionId)
                }
                is CompanionSignal.ChatDone -> {
                    val result = snackbarHostState.showSnackbar(
                        message = signal.title?.takeIf { it.isNotBlank() } ?: "Chat finished",
                        actionLabel = "View",
                        duration = SnackbarDuration.Short,
                    )
                    if (result == SnackbarResult.ActionPerformed) backStack.openChat(signal.sessionId)
                }
                is CompanionSignal.ChatFailed -> {
                    val result = snackbarHostState.showSnackbar(
                        message = signal.title?.takeIf { it.isNotBlank() } ?: "Something went wrong in chat",
                        actionLabel = "View",
                        duration = SnackbarDuration.Short,
                    )
                    if (result == SnackbarResult.ActionPerformed) backStack.openChat(signal.sessionId)
                }
                is CompanionSignal.Working, CompanionSignal.Idle -> Unit
            }
        }
    }

    val items = TopLevelRoutes.mapNotNull { it.navItem(unread) }
    var collapsed by rememberSaveable { mutableStateOf(false) }
    // Starting a conversation happens in Home's composer (the server saves a chat on its first message).
    val newChat = { backStack.selectTopLevel(Home) }

    val showInAppBubble = platform == ComposerPlatform.Mobile &&
        bubbleEnabled &&
        !dismissedInApp &&
        companionState !is CompanionUiState.Snoozed &&
        backStack.lastOrNull() !is Chat

    Row(Modifier.fillMaxSize().background(NomiTheme.extendedColors.ground)) {
        if (inApp && wide) {
            NomiSidebar(
                items, backStack.currentTopLevel, backStack::selectTopLevel,
                onNewChat = newChat, collapsed = collapsed, onToggleCollapsed = { collapsed = !collapsed },
            )
        }
        Column(Modifier.weight(FILL).fillMaxHeight()) {
            // A phone shows a chat full screen; the bar comes back when it is popped.
            val bottomBar = inApp && !wide && backStack.lastOrNull() !is Chat
            val consumed = if (bottomBar) Modifier.consumeWindowInsets(WindowInsets.navigationBars) else Modifier
            Box(Modifier.weight(FILL).then(consumed)) {
                NomiNavDisplay(backStack, Modifier.fillMaxSize(), wide = wide)
                if (showInAppBubble) {
                    InAppFloatingBubbleOverlay(
                        state = companionState,
                        companion = companion,
                        onDismissBubble = {
                            dismissedInApp = true
                            companion.stateMachine.clearUnread()
                            companion.stateMachine.collapse()
                        },
                    )
                }
                NomiSnackbarHost(snackbarHostState, Modifier.align(Alignment.BottomCenter).padding(SnackbarPadding))
            }
            if (bottomBar) NomiNavigationBar(items, backStack.currentTopLevel, backStack::selectTopLevel)
        }
    }
}

@Composable
private fun InAppFloatingBubbleOverlay(
    state: CompanionUiState,
    companion: CompanionController,
    onDismissBubble: () -> Unit,
) {
    val density = LocalDensity.current
    BoxWithConstraints(Modifier.fillMaxSize()) {
        val widthPx = constraints.maxWidth
        val heightPx = constraints.maxHeight
        val bubbleSizePx = with(density) { BubbleSizeDp.roundToPx() }
        val marginPx = with(density) { BubbleEdgeMargin.roundToPx() }
        val topInsetPx = with(density) { BubbleTopInset.roundToPx() }
        val bottomInsetPx = with(density) { BubbleBottomInset.roundToPx() }
        val dismissOffsetPx = with(density) { DismissBottomOffset.roundToPx() }
        val captureRadiusPx = with(density) { DismissCaptureRadius.roundToPx() }

        val area = remember(widthPx, heightPx, marginPx, topInsetPx, bottomInsetPx) {
            ScreenAreaPx(
                widthPx = widthPx,
                heightPx = heightPx,
                marginPx = marginPx,
                topInsetPx = topInsetPx,
                bottomInsetPx = bottomInsetPx,
            )
        }

        var initialized by remember { mutableStateOf(false) }
        var posX by remember { mutableFloatStateOf((widthPx - bubbleSizePx - marginPx).toFloat()) }
        var posY by remember { mutableFloatStateOf(heightPx * INITIAL_BUBBLE_TOP_FRACTION) }
        var dockedRight by remember { mutableStateOf(true) }
        var dragging by remember { mutableStateOf(false) }
        var capturedInDismiss by remember { mutableStateOf(false) }

        LaunchedEffect(widthPx, heightPx) {
            if (widthPx > 0 && heightPx > 0) {
                if (!initialized) {
                    posX = (widthPx - bubbleSizePx - marginPx).toFloat()
                    posY = (heightPx * INITIAL_BUBBLE_TOP_FRACTION).coerceIn(
                        topInsetPx.toFloat(),
                        (heightPx - bottomInsetPx - bubbleSizePx).coerceAtLeast(topInsetPx).toFloat(),
                    )
                    dockedRight = true
                    initialized = true
                } else {
                    val snapped = snapToEdge(
                        point = BubblePointPx(posX.roundToInt(), posY.roundToInt()),
                        bubbleSizePx = bubbleSizePx,
                        area = area,
                    )
                    posX = snapped.x.toFloat()
                    posY = snapped.y.toFloat()
                    dockedRight = snapped.x >= widthPx / HALF
                }
            }
        }

        var measuredSize by remember { mutableStateOf(IntSize(bubbleSizePx, bubbleSizePx)) }
        val liquidSnapSpec = remember {
            spring<Float>(
                dampingRatio = LIQUID_SNAP_DAMPING,
                stiffness = LIQUID_SNAP_STIFFNESS,
            )
        }
        val animatedAnchorX by animateFloatAsState(
            targetValue = posX,
            animationSpec = if (dragging) snap() else liquidSnapSpec,
            label = "in-app-bubble-anchor-x",
        )
        val animatedAnchorY by animateFloatAsState(
            targetValue = posY,
            animationSpec = if (dragging) snap() else liquidSnapSpec,
            label = "in-app-bubble-anchor-y",
        )
        val pullStretch = (abs(posX - animatedAnchorX) / (bubbleSizePx * LIQUID_STRETCH_DIVISOR))
            .coerceIn(MIN_STRETCH, LIQUID_MAX_STRETCH)
        val liquidStretchX = FILL + pullStretch
        val liquidStretchY = FILL - pullStretch * LIQUID_SQUISH_FACTOR

        Box(
            modifier = Modifier
                .onSizeChanged { size ->
                    if (size.width > 0 && size.height > 0) {
                        measuredSize = size
                    }
                }
                .offset {
                    val resolved = resolveOverlayWindowPosition(
                        bubbleAnchor = BubblePointPx(animatedAnchorX.roundToInt(), animatedAnchorY.roundToInt()),
                        bubbleSizePx = bubbleSizePx,
                        viewWidthPx = measuredSize.width,
                        viewHeightPx = measuredSize.height,
                        dockedRight = dockedRight,
                        area = area,
                    )
                    IntOffset(resolved.x, resolved.y)
                },
        ) {
            MessengerBubble(
                state = state,
                onTapHead = { companion.stateMachine.toggleExpanded() },
                onOpenLink = companion::openLink,
                onOpenApp = companion::openApp,
                dockedRight = dockedRight,
                liquidStretchX = liquidStretchX,
                liquidStretchY = liquidStretchY,
                onDragStart = {
                    dragging = true
                    capturedInDismiss = false
                },
                onDragBy = { delta ->
                    val next = clampBubblePosition(
                        point = BubblePointPx((posX + delta.x).roundToInt(), (posY + delta.y).roundToInt()),
                        bubbleSizePx = bubbleSizePx,
                        area = area,
                    )
                    posX = next.x.toFloat()
                    posY = next.y.toFloat()
                    capturedInDismiss = isInsideDismissTarget(
                        bubbleTopLeft = next,
                        bubbleSizePx = bubbleSizePx,
                        area = area,
                        bottomOffsetPx = dismissOffsetPx,
                        captureRadiusPx = captureRadiusPx,
                    )
                },
                onDragEnd = {
                    dragging = false
                    if (capturedInDismiss) {
                        capturedInDismiss = false
                        onDismissBubble()
                    } else {
                        val snapped = snapToEdge(
                            point = BubblePointPx(posX.roundToInt(), posY.roundToInt()),
                            bubbleSizePx = bubbleSizePx,
                            area = area,
                        )
                        posX = snapped.x.toFloat()
                        posY = snapped.y.toFloat()
                        dockedRight = snapped.x >= widthPx / HALF
                    }
                },
                onQuickReply = companion::sendQuickReply,
                onDismiss = onDismissBubble,
            )
        }

        AnimatedVisibility(
            visible = dragging,
            modifier = Modifier
                .align(Alignment.BottomCenter)
                .padding(bottom = DismissBottomOffset),
            enter = fadeIn(MaterialTheme.motionScheme.fastEffectsSpec()) + scaleIn(MaterialTheme.motionScheme.defaultSpatialSpec()),
            exit = fadeOut(MaterialTheme.motionScheme.fastEffectsSpec()) + scaleOut(MaterialTheme.motionScheme.defaultSpatialSpec()),
        ) {
            BubbleDismissTarget(captured = capturedInDismiss)
        }
    }
}

/** Whether the person asked the system for less motion (read where it is cheap; desktop has no such setting here). */
@Composable
expect fun rememberReducedMotion(): Boolean
