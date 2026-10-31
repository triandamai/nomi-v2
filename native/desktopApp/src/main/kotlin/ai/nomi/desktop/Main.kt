package ai.nomi.desktop

import ai.nomi.app.NomiApp
import ai.nomi.app.desktopPlatformModule
import ai.nomi.app.di.initKoin
import ai.nomi.companion.CompanionController
import ai.nomi.companion.CompanionUiState
import ai.nomi.companion.DynamicIslandPill
import ai.nomi.data.AppearanceRepository
import ai.nomi.data.SessionGate
import ai.nomi.data.THEME_DARK
import ai.nomi.data.THEME_LIGHT
import ai.nomi.designsystem.component.ComposerPlatform
import ai.nomi.designsystem.component.LocalComposerPlatform
import ai.nomi.designsystem.theme.NomiTheme
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.graphics.painter.Painter
import androidx.compose.ui.unit.DpSize
import androidx.compose.ui.unit.IntRect
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Tray
import androidx.compose.ui.window.Window
import androidx.compose.ui.window.WindowPosition
import androidx.compose.ui.window.application
import androidx.compose.ui.window.rememberWindowState
import kotlinx.coroutines.delay
import org.koin.core.context.GlobalContext
import java.awt.GraphicsEnvironment
import java.awt.Toolkit

private const val WINDOW_TITLE = "Nomi"
private const val ISLAND_TITLE = "Nomi Island"
private const val BACKGROUND_ARG = "--background"
private const val TRAY_ICON_SIZE = 24f
private const val TRAY_CIRCLE_FACTOR = 0.42f
private const val TRAY_EYE_OFFSET_X = 0.14f
private const val TRAY_EYE_OFFSET_Y = 0.05f
private const val TRAY_EYE_RADIUS = 0.06f
private const val HALF_F = 2f
private const val TRAY_PRIMARY_ARGB = 0xFF5BE08FL
private const val TRAY_INK_ARGB = 0xFF08100CL
private const val HANDLE_POLL_MS = 16L
private const val HANDLE_MAX_ATTEMPTS = 60

private object NomiTrayPainter : Painter() {
    override val intrinsicSize: Size = Size(TRAY_ICON_SIZE, TRAY_ICON_SIZE)

    override fun DrawScope.onDraw() {
        val radius = size.minDimension * TRAY_CIRCLE_FACTOR
        val center = Offset(size.width / HALF_F, size.height / HALF_F)
        drawCircle(color = Color(TRAY_PRIMARY_ARGB), radius = radius, center = center)
        val eyeRadius = size.minDimension * TRAY_EYE_RADIUS
        val eyeY = center.y - size.height * TRAY_EYE_OFFSET_Y
        val eyeDx = size.width * TRAY_EYE_OFFSET_X
        drawCircle(color = Color(TRAY_INK_ARGB), radius = eyeRadius, center = Offset(center.x - eyeDx, eyeY))
        drawCircle(color = Color(TRAY_INK_ARGB), radius = eyeRadius, center = Offset(center.x + eyeDx, eyeY))
    }
}

fun main(args: Array<String>) {
    initKoin(desktopPlatformModule())
    val os = detectDesktopOs()
    val startHidden = args.contains(BACKGROUND_ARG)

    application(exitProcessOnExit = false) {
        var mainVisible by remember { mutableStateOf(!startHidden) }
        val koin = remember { GlobalContext.get() }
        val gate = remember(koin) { koin.get<SessionGate>() }
        val companion = remember(koin) { koin.get<CompanionController>() }
        val appearance = remember(koin) { koin.get<AppearanceRepository>() }

        val islandEnabled by companion.islandEnabled.collectAsState()
        val snoozed by companion.snoozed.collectAsState()
        val uiState by companion.uiState.collectAsState()

        LaunchedEffect(gate) { gate.restore() }

        LaunchedEffect(companion) {
            companion.navigationRequests.collect {
                mainVisible = true
            }
        }

        Tray(
            icon = NomiTrayPainter,
            tooltip = WINDOW_TITLE,
            onAction = { mainVisible = true },
            menu = {
                Item("Open Nomi", onClick = { mainVisible = true })
                if (os.supportsIsland) {
                    Item(
                        if (snoozed) "Resume island" else "Pause island",
                        onClick = { companion.toggleSnoozed() },
                    )
                }
                Separator()
                Item("Quit", onClick = ::exitApplication)
            },
        )

        Window(
            onCloseRequest = { mainVisible = false },
            visible = mainVisible,
            title = WINDOW_TITLE,
        ) {
            LaunchedEffect(mainVisible) {
                if (mainVisible) window.toFront()
            }
            CompositionLocalProvider(LocalComposerPlatform provides ComposerPlatform.Desktop) {
                NomiApp()
            }
        }

        val showIsland = islandEnabled && !snoozed && os.supportsIsland && uiState !is CompanionUiState.Snoozed
        if (showIsland) {
            DesktopIslandWindow(
                os = os,
                companion = companion,
                appearance = appearance,
                uiState = uiState,
                onRevealMainWindow = { mainVisible = true },
            )
        }
    }
}

@Composable
private fun DesktopIslandWindow(
    os: DesktopOs,
    companion: CompanionController,
    appearance: AppearanceRepository,
    uiState: CompanionUiState,
    onRevealMainWindow: () -> Unit,
) {
    val scope = rememberCoroutineScope()
    val isOpen = uiState is CompanionUiState.Expanded
    val isPeeking = uiState is CompanionUiState.Peeking
    val isExpanded = isOpen || isPeeking
    val isWorking = uiState is CompanionUiState.Working
    val overview by companion.overview.collectAsState()
    LaunchedEffect(isOpen) {
        if (isOpen) companion.refreshOverview()
    }

    val displays = remember(os) { readDesktopDisplays(os) }
    val initialPlacement = remember(displays) {
        computeIslandPlacement(displays.first(), isExpanded = false, isWorking = false)
    }

    var currentPillBounds by remember { mutableStateOf(initialPlacement.pillBoundsInScreenDp) }
    val tracker = remember(scope) {
        CursorTracker(scope = scope, pillBoundsProvider = { currentPillBounds })
    }
    DisposableEffect(tracker) {
        tracker.start()
        onDispose { tracker.stop() }
    }
    val cursorSample by tracker.sample.collectAsState()

    val activeDisplay = selectDisplayForCursor(displays, cursorSample?.xDp, cursorSample?.yDp)
    val placement = computeIslandPlacement(activeDisplay, isExpanded = isOpen, isWorking = isWorking, isPeeking = isPeeking)

    LaunchedEffect(cursorSample?.insideIsland) {
        val inside = cursorSample?.insideIsland ?: false
        companion.stateMachine.setHovered(inside)
    }

    val windowState = rememberWindowState(
        position = WindowPosition.Absolute(placement.windowXDp.dp, placement.windowYDp.dp),
        size = DpSize(placement.windowWidthDp.dp, placement.windowHeightDp.dp),
    )
    LaunchedEffect(placement.windowXDp, placement.windowYDp, placement.windowWidthDp, placement.windowHeightDp) {
        windowState.position = WindowPosition.Absolute(placement.windowXDp.dp, placement.windowYDp.dp)
        windowState.size = DpSize(placement.windowWidthDp.dp, placement.windowHeightDp.dp)
    }

    val themeMode by appearance.theme.collectAsState()
    val accentColor by appearance.accentColor.collectAsState()
    val systemDark = isSystemInDarkTheme()
    val dark = when (themeMode) {
        THEME_LIGHT -> false
        THEME_DARK -> true
        else -> systemDark
    }

    Window(
        onCloseRequest = {},
        state = windowState,
        visible = true,
        title = ISLAND_TITLE,
        undecorated = true,
        transparent = true,
        alwaysOnTop = true,
        resizable = false,
        focusable = true,
    ) {
        LaunchedEffect(window, placement.windowXDp, placement.windowYDp) {
            var attempts = 0
            while (window.windowHandle == 0L && attempts < HANDLE_MAX_ATTEMPTS) {
                delay(HANDLE_POLL_MS)
                attempts++
            }
            delay(HANDLE_POLL_MS)
            DesktopWindowInterop.configureIslandWindow(window, os)
            DesktopWindowInterop.positionIslandWindow(window, os, placement.windowXDp, placement.windowYDp)
            delay(HANDLE_POLL_MS)
            val actualWindowY = runCatching { if (window.isShowing) window.locationOnScreen.y else null }.getOrNull()
            currentPillBounds = resolveLivePillBounds(placement, actualWindowY, isExpanded)
        }

        val insidePill = cursorSample?.insideIsland ?: false
        LaunchedEffect(window, insidePill, isExpanded, placement) {
            val actualWindowY = runCatching { if (window.isShowing) window.locationOnScreen.y else null }.getOrNull()
            currentPillBounds = resolveLivePillBounds(placement, actualWindowY, isExpanded)
            DesktopWindowInterop.setIslandClickThrough(
                window = window,
                os = os,
                clickThrough = !insidePill && !isExpanded,
            )
        }

        NomiTheme(accent = appearance.paletteKey(accentColor), dark = dark) {
            Box(
                Modifier
                    .fillMaxSize()
                    .padding(horizontal = ISLAND_WINDOW_PADDING_DP.dp),
                contentAlignment = Alignment.TopCenter,
            ) {
                DynamicIslandPill(
                    state = uiState,
                    onOpenLink = { link ->
                        onRevealMainWindow()
                        companion.openLink(link)
                    },
                    onOpenApp = {
                        onRevealMainWindow()
                        companion.openApp()
                    },
                    lookAt = cursorSample?.lookAt,
                    collapsedWidth = placement.collapsedPillWidthDp.dp,
                    notchMode = activeDisplay.notchWidthDp != null,
                    notchHeight = activeDisplay.menuBarHeightDp.dp,
                    onClickCollapsed = { companion.stateMachine.toggleExpanded() },
                    overview = overview,
                    onAsk = companion::ask,
                    onNewConversation = companion::newConversation,
                    onOpenConversation = {
                        onRevealMainWindow()
                        companion.openConversation()
                    },
                )
            }
        }
    }
}

private fun readDesktopDisplays(os: DesktopOs): List<DisplayBoundsDp> {
    val defaultNotch = if (os == DesktopOs.MacOS) DEFAULT_MAC_NOTCH_WIDTH_DP else null
    val devices = runCatching { GraphicsEnvironment.getLocalGraphicsEnvironment().screenDevices.toList() }
        .getOrDefault(emptyList())
    if (devices.isEmpty()) {
        return listOf(
            DisplayBoundsDp(
                xDp = 0,
                yDp = 0,
                widthDp = 1440,
                heightDp = 900,
                notchWidthDp = defaultNotch,
            ),
        )
    }
    val toolkit = runCatching { Toolkit.getDefaultToolkit() }.getOrNull()
    return devices.mapIndexed { index, device ->
        val config = device.defaultConfiguration
        val bounds = config.bounds
        val insets = toolkit?.let { runCatching { it.getScreenInsets(config) }.getOrNull() }
        val topInset = insets?.top?.takeIf { it > 0 } ?: DEFAULT_MENU_BAR_HEIGHT_DP
        DisplayBoundsDp(
            xDp = bounds.x,
            yDp = bounds.y,
            widthDp = bounds.width,
            heightDp = bounds.height,
            menuBarHeightDp = topInset,
            // Only the built-in screen has a notch, and only notched screens have a menu bar as tall as one.
            notchWidthDp = if (index == 0 && hasNotch(topInset)) defaultNotch else null,
        )
    }
}
