package ai.nomi.app.android

import ai.nomi.companion.BubbleDismissTarget
import ai.nomi.companion.BubblePointPx
import ai.nomi.companion.BubbleSizeDp
import ai.nomi.companion.CompanionController
import ai.nomi.companion.CompanionUiState
import ai.nomi.companion.MessengerBubble
import ai.nomi.companion.ScreenAreaPx
import ai.nomi.companion.isInsideDismissTarget
import ai.nomi.companion.snapToEdge
import ai.nomi.data.AppearanceRepository
import ai.nomi.data.SessionGate
import ai.nomi.data.THEME_DARK
import ai.nomi.data.THEME_LIGHT
import ai.nomi.designsystem.theme.NomiTheme
import android.content.Context
import android.content.Intent
import android.graphics.PixelFormat
import android.net.Uri
import android.provider.Settings
import android.view.Gravity
import android.view.WindowManager
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.platform.ComposeView
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.launch
import kotlin.math.roundToInt

private const val EDGE_MARGIN_DP = 12
private const val INITIAL_TOP_FRACTION = 0.25f
private const val TOP_INSET_DP = 48
private const val BOTTOM_INSET_DP = 48
private const val DISMISS_BOTTOM_OFFSET_DP = 96
private const val DISMISS_CAPTURE_RADIUS_DP = 76
private const val HALF = 2

/**
 * Manages the Messenger-style chat-head `ComposeView` overlay on Android's `WindowManager`:
 * - Uses `TYPE_APPLICATION_OVERLAY` with `FLAG_NOT_FOCUSABLE` while collapsed/peeking and clears `FLAG_NOT_FOCUSABLE`
 *   while expanded so the soft keyboard works for quick replies.
 * - Snaps to the nearest horizontal screen edge on release and shows a bottom-center dismiss target while dragging.
 * - Automatically hides while `MainActivity` is in the foreground and re-appears when backgrounded.
 */
class BubbleOverlayController(
    private val context: Context,
    private val scope: CoroutineScope,
    private val gate: SessionGate,
    private val companion: CompanionController,
    private val appearance: AppearanceRepository,
) {
    private val windowManager = context.getSystemService(Context.WINDOW_SERVICE) as WindowManager
    private var lifecycleOwner: OverlayLifecycleOwner? = null
    private var bubbleView: ComposeView? = null
    private var bubbleParams: WindowManager.LayoutParams? = null
    private var dismissView: ComposeView? = null
    private var dismissParams: WindowManager.LayoutParams? = null

    private var appInForeground: Boolean = false
    private var dismissedUntilNextPeek: Boolean = false
    private var dockedRight by mutableStateOf(true)
    private var dragging by mutableStateOf(false)
    private var capturedInDismiss by mutableStateOf(false)

    init {
        scope.launch {
            combine(
                gate.signedIn,
                companion.bubbleEnabled,
                companion.uiState,
            ) { signedIn, bubbleEnabled, uiState ->
                Triple(signedIn, bubbleEnabled, uiState)
            }.collect { (signedIn, bubbleEnabled, uiState) ->
                if (uiState is CompanionUiState.Peeking) {
                    dismissedUntilNextPeek = false
                }
                updateVisibility(signedIn, bubbleEnabled, uiState)
            }
        }
    }

    fun canDrawOverlays(): Boolean = Settings.canDrawOverlays(context)

    fun createOverlayPermissionIntent(): Intent =
        Intent(Settings.ACTION_MANAGE_OVERLAY_PERMISSION, Uri.parse("package:${context.packageName}"))
            .addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)

    fun setAppInForeground(inForeground: Boolean) {
        appInForeground = inForeground
        updateVisibility(
            signedIn = gate.signedIn.value,
            bubbleEnabled = companion.bubbleEnabled.value,
            uiState = companion.uiState.value,
        )
    }

    fun destroy() {
        removeViews()
    }

    private fun updateVisibility(
        signedIn: Boolean,
        bubbleEnabled: Boolean,
        uiState: CompanionUiState,
    ) {
        val shouldShow = signedIn &&
            bubbleEnabled &&
            !appInForeground &&
            !dismissedUntilNextPeek &&
            uiState !is CompanionUiState.Snoozed &&
            canDrawOverlays()

        if (shouldShow) {
            ensureViewsAttached()
            updateFocusableFlag(isExpanded = uiState is CompanionUiState.Expanded)
        } else {
            removeViews()
        }
    }

    private fun screenArea(): ScreenAreaPx {
        val metrics = context.resources.displayMetrics
        val density = metrics.density
        return ScreenAreaPx(
            widthPx = metrics.widthPixels,
            heightPx = metrics.heightPixels,
            marginPx = (EDGE_MARGIN_DP * density).roundToInt(),
            topInsetPx = (TOP_INSET_DP * density).roundToInt(),
            bottomInsetPx = (BOTTOM_INSET_DP * density).roundToInt(),
        )
    }

    private fun bubbleSizePx(): Int =
        (BubbleSizeDp.value * context.resources.displayMetrics.density).roundToInt()

    private fun dismissOffsetPx(): Int =
        (DISMISS_BOTTOM_OFFSET_DP * context.resources.displayMetrics.density).roundToInt()

    private fun dismissCaptureRadiusPx(): Int =
        (DISMISS_CAPTURE_RADIUS_DP * context.resources.displayMetrics.density).roundToInt()

    private fun ensureViewsAttached() {
        if (bubbleView != null) return
        val area = screenArea()
        val sizePx = bubbleSizePx()
        val initialX = area.widthPx - sizePx - area.marginPx
        val initialY = (area.heightPx * INITIAL_TOP_FRACTION).roundToInt()
        dockedRight = true

        val owner = OverlayLifecycleOwner().also { it.onCreate() }
        lifecycleOwner = owner

        val bParams = WindowManager.LayoutParams(
            WindowManager.LayoutParams.WRAP_CONTENT,
            WindowManager.LayoutParams.WRAP_CONTENT,
            WindowManager.LayoutParams.TYPE_APPLICATION_OVERLAY,
            WindowManager.LayoutParams.FLAG_NOT_FOCUSABLE or WindowManager.LayoutParams.FLAG_LAYOUT_NO_LIMITS,
            PixelFormat.TRANSLUCENT,
        ).apply {
            gravity = Gravity.TOP or Gravity.START
            x = initialX
            y = initialY
        }
        bubbleParams = bParams

        val view = ComposeView(context).apply {
            owner.attachTo(this)
            setContent {
                val uiState by companion.uiState.collectAsStateWithLifecycle()
                val themeMode by appearance.theme.collectAsStateWithLifecycle()
                val accentColor by appearance.accentColor.collectAsStateWithLifecycle()
                val systemDark = isSystemInDarkTheme()
                val dark = when (themeMode) {
                    THEME_LIGHT -> false
                    THEME_DARK -> true
                    else -> systemDark
                }

                LaunchedEffect(uiState is CompanionUiState.Expanded) {
                    updateFocusableFlag(uiState is CompanionUiState.Expanded)
                }

                NomiTheme(accent = appearance.paletteKey(accentColor), dark = dark) {
                    MessengerBubble(
                        state = uiState,
                        onTapHead = { companion.stateMachine.toggleExpanded() },
                        onOpenLink = ::launchMainActivityWithLink,
                        onOpenApp = { launchMainActivityWithLink("/") },
                        dockedRight = dockedRight,
                        onDragStart = {
                            dragging = true
                            capturedInDismiss = false
                            ensureDismissViewAttached()
                        },
                        onDragBy = { delta ->
                            val params = bubbleParams ?: return@MessengerBubble
                            params.x += delta.x.roundToInt()
                            params.y += delta.y.roundToInt()
                            val currentArea = screenArea()
                            capturedInDismiss = isInsideDismissTarget(
                                bubbleTopLeft = BubblePointPx(params.x, params.y),
                                bubbleSizePx = bubbleSizePx(),
                                area = currentArea,
                                bottomOffsetPx = dismissOffsetPx(),
                                captureRadiusPx = dismissCaptureRadiusPx(),
                            )
                            runCatching { windowManager.updateViewLayout(this, params) }
                        },
                        onDragEnd = {
                            dragging = false
                            removeDismissView()
                            val params = bubbleParams ?: return@MessengerBubble
                            val currentArea = screenArea()
                            if (capturedInDismiss) {
                                capturedInDismiss = false
                                dismissedUntilNextPeek = true
                                companion.stateMachine.collapse()
                                removeViews()
                            } else {
                                val snapped = snapToEdge(
                                    point = BubblePointPx(params.x, params.y),
                                    bubbleSizePx = bubbleSizePx(),
                                    area = currentArea,
                                )
                                params.x = snapped.x
                                params.y = snapped.y
                                dockedRight = snapped.x >= currentArea.widthPx / HALF
                                runCatching { windowManager.updateViewLayout(this, params) }
                            }
                        },
                        onQuickReply = companion::sendQuickReply,
                        onDismiss = {
                            dismissedUntilNextPeek = true
                            companion.stateMachine.collapse()
                            removeViews()
                        },
                    )
                }
            }
        }
        bubbleView = view
        runCatching { windowManager.addView(view, bParams) }
    }

    private fun ensureDismissViewAttached() {
        if (dismissView != null) return
        val owner = lifecycleOwner ?: return
        val dParams = WindowManager.LayoutParams(
            WindowManager.LayoutParams.WRAP_CONTENT,
            WindowManager.LayoutParams.WRAP_CONTENT,
            WindowManager.LayoutParams.TYPE_APPLICATION_OVERLAY,
            WindowManager.LayoutParams.FLAG_NOT_FOCUSABLE or WindowManager.LayoutParams.FLAG_NOT_TOUCHABLE,
            PixelFormat.TRANSLUCENT,
        ).apply {
            gravity = Gravity.BOTTOM or Gravity.CENTER_HORIZONTAL
            y = dismissOffsetPx()
        }
        dismissParams = dParams
        val view = ComposeView(context).apply {
            owner.attachTo(this)
            setContent {
                val themeMode by appearance.theme.collectAsStateWithLifecycle()
                val accentColor by appearance.accentColor.collectAsStateWithLifecycle()
                val systemDark = isSystemInDarkTheme()
                val dark = when (themeMode) {
                    THEME_LIGHT -> false
                    THEME_DARK -> true
                    else -> systemDark
                }
                if (dragging) {
                    NomiTheme(accent = appearance.paletteKey(accentColor), dark = dark) {
                        BubbleDismissTarget(captured = capturedInDismiss)
                    }
                }
            }
        }
        dismissView = view
        runCatching { windowManager.addView(view, dParams) }
    }

    private fun removeDismissView() {
        dismissView?.let { view ->
            runCatching { windowManager.removeView(view) }
        }
        dismissView = null
        dismissParams = null
    }

    private fun updateFocusableFlag(isExpanded: Boolean) {
        val view = bubbleView ?: return
        val params = bubbleParams ?: return
        val desiredFlags = if (isExpanded) {
            params.flags and WindowManager.LayoutParams.FLAG_NOT_FOCUSABLE.inv()
        } else {
            params.flags or WindowManager.LayoutParams.FLAG_NOT_FOCUSABLE
        }
        if (desiredFlags != params.flags) {
            params.flags = desiredFlags
            runCatching { windowManager.updateViewLayout(view, params) }
        }
    }

    private fun removeViews() {
        removeDismissView()
        bubbleView?.let { view ->
            runCatching { windowManager.removeView(view) }
        }
        bubbleView = null
        bubbleParams = null
        lifecycleOwner?.onDestroy()
        lifecycleOwner = null
    }

    private fun launchMainActivityWithLink(link: String) {
        companion.openLink(link)
        val intent = Intent(context, MainActivity::class.java).apply {
            flags = Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_SINGLE_TOP or Intent.FLAG_ACTIVITY_CLEAR_TOP
            putExtra(MainActivity.EXTRA_DEEP_LINK, link)
        }
        context.startActivity(intent)
    }
}
