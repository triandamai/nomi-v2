package ai.nomi.app.android

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
import ai.nomi.data.SessionGate
import ai.nomi.data.THEME_DARK
import ai.nomi.data.THEME_LIGHT
import ai.nomi.designsystem.theme.NomiTheme
import android.content.Context
import android.content.Intent
import android.graphics.PixelFormat
import android.net.Uri
import android.provider.Settings
import android.view.Choreographer
import android.view.Gravity
import android.view.MotionEvent
import android.view.View
import android.view.ViewConfiguration
import android.view.WindowManager
import android.widget.FrameLayout
import androidx.compose.animation.core.animate
import androidx.compose.animation.core.spring
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.MonotonicFrameClock
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.platform.ComposeView
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.launch
import kotlinx.coroutines.suspendCancellableCoroutine
import kotlin.coroutines.resume
import kotlin.math.abs
import kotlin.math.hypot
import kotlin.math.roundToInt

private const val EDGE_MARGIN_DP = 12
private const val INITIAL_TOP_FRACTION = 0.25f
private const val TOP_INSET_DP = 48
private const val BOTTOM_INSET_DP = 48
private const val DISMISS_BOTTOM_OFFSET_DP = 96
private const val DISMISS_CAPTURE_RADIUS_DP = 76
private const val HALF = 2
private const val LIQUID_PULLBACK_DAMPING = 0.72f
private const val LIQUID_PULLBACK_STIFFNESS = 380f
private const val LIQUID_STRETCH_MAX_EXTRA_X = 0.16f
private const val LIQUID_STRETCH_MAX_SQUISH_Y = 0.10f
private const val LIQUID_STRETCH_DISTANCE_PX = 280f
private const val BASE_SCALE = 1f
private const val ZERO_FLOAT = 0f
private const val ONE_FLOAT = 1f
private const val ZERO_INT = 0
private const val FOCUS_RELEASE_DELAY_MS = 220L

private object ChoreographerFrameClock : MonotonicFrameClock {
    override suspend fun <R> withFrameNanos(onFrame: (Long) -> R): R =
        suspendCancellableCoroutine { continuation ->
            val choreographer = Choreographer.getInstance()
            val callback = Choreographer.FrameCallback { frameTimeNanos ->
                if (continuation.isActive) {
                    continuation.resume(onFrame(frameTimeNanos))
                }
            }
            choreographer.postFrameCallback(callback)
            continuation.invokeOnCancellation {
                choreographer.removeFrameCallback(callback)
            }
        }
}

/**
 * Manages the Messenger-style chat-head `ComposeView` overlay on Android's `WindowManager`:
 * - Uses `TYPE_APPLICATION_OVERLAY` with `FLAG_NOT_FOCUSABLE` while collapsed/peeking and clears `FLAG_NOT_FOCUSABLE`
 *   while expanded so the soft keyboard works for quick replies.
 * - Uses hardware `MotionEvent.rawX`/`rawY` coordinates for butter-smooth dragging across the screen, pulls back to the
 *   nearest horizontal screen edge on release with a liquid spring animation, and shows a bottom-center dismiss target
 *   while dragging.
 * - Pins `Gravity.RIGHT` when docked right and `Gravity.LEFT` when docked left so expanding/collapsing the quick-chat
 *   card never shifts the bubble head horizontally.
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
    private var bubbleView: View? = null
    private var bubbleParams: WindowManager.LayoutParams? = null
    private var dismissView: ComposeView? = null
    private var dismissParams: WindowManager.LayoutParams? = null
    private var snapJob: Job? = null

    private var appInForeground: Boolean = false
    private var dismissedUntilNextPeek: Boolean = false
    private var bubbleAnchorX: Int = 0
    private var bubbleAnchorY: Int = 0
    private var dockedRight by mutableStateOf(true)
    private var dragging by mutableStateOf(false)
    private var capturedInDismiss by mutableStateOf(false)
    private var liquidStretchX by mutableFloatStateOf(BASE_SCALE)
    private var liquidStretchY by mutableFloatStateOf(BASE_SCALE)

    init {
        scope.launch {
            combine(
                gate.signedIn,
                companion.bubbleEnabled,
                companion.uiState,
            ) { signedIn, bubbleEnabled, uiState ->
                Triple(signedIn, bubbleEnabled, uiState)
            }.collect { (_, bubbleEnabled, uiState) ->
                if (uiState is CompanionUiState.Peeking) {
                    dismissedUntilNextPeek = false
                }
                updateVisibility(bubbleEnabled, uiState)
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
            bubbleEnabled = companion.bubbleEnabled.value,
            uiState = companion.uiState.value,
        )
    }

    fun destroy() {
        removeViews()
    }

    private fun updateVisibility(
        bubbleEnabled: Boolean,
        uiState: CompanionUiState,
    ) {
        val shouldShow = bubbleEnabled &&
            !appInForeground &&
            !dismissedUntilNextPeek &&
            uiState !is CompanionUiState.Snoozed &&
            canDrawOverlays()

        if (shouldShow) {
            ensureViewsAttached()
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

    private fun applyDockedGravityPosition(
        params: WindowManager.LayoutParams,
        area: ScreenAreaPx,
        viewWidthPx: Int,
        viewHeightPx: Int,
    ): Boolean {
        val sizePx = bubbleSizePx()
        val resolved = resolveOverlayWindowPosition(
            bubbleAnchor = BubblePointPx(bubbleAnchorX, bubbleAnchorY),
            bubbleSizePx = sizePx,
            viewWidthPx = viewWidthPx.coerceAtLeast(sizePx),
            viewHeightPx = viewHeightPx.coerceAtLeast(sizePx),
            dockedRight = dockedRight,
            area = area,
        )
        val desiredGravity = if (dockedRight) {
            Gravity.TOP or Gravity.RIGHT
        } else {
            Gravity.TOP or Gravity.LEFT
        }
        val desiredX = if (dockedRight) {
            (area.widthPx - (bubbleAnchorX + sizePx)).coerceAtLeast(ZERO_INT)
        } else {
            bubbleAnchorX.coerceAtLeast(ZERO_INT)
        }
        val desiredY = resolved.y
        val changed = params.gravity != desiredGravity || params.x != desiredX || params.y != desiredY
        params.gravity = desiredGravity
        params.x = desiredX
        params.y = desiredY
        return changed
    }

    private fun ensureViewsAttached() {
        if (bubbleView != null) return
        val area = screenArea()
        val sizePx = bubbleSizePx()
        val initialX = area.widthPx - sizePx - area.marginPx
        val initialY = (area.heightPx * INITIAL_TOP_FRACTION).roundToInt()
        bubbleAnchorX = initialX
        bubbleAnchorY = initialY
        dockedRight = true
        liquidStretchX = BASE_SCALE
        liquidStretchY = BASE_SCALE

        val owner = OverlayLifecycleOwner().also { it.onCreate() }
        lifecycleOwner = owner

        val bParams = WindowManager.LayoutParams(
            WindowManager.LayoutParams.WRAP_CONTENT,
            WindowManager.LayoutParams.WRAP_CONTENT,
            WindowManager.LayoutParams.TYPE_APPLICATION_OVERLAY,
            WindowManager.LayoutParams.FLAG_NOT_FOCUSABLE,
            PixelFormat.TRANSLUCENT,
        ).apply {
            applyDockedGravityPosition(
                params = this,
                area = area,
                viewWidthPx = sizePx,
                viewHeightPx = sizePx,
            )
        }
        bubbleParams = bParams

        val composeView = ComposeView(context).apply {
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

                val isExpanded = uiState is CompanionUiState.Expanded
                val overview by companion.overview.collectAsStateWithLifecycle()
                LaunchedEffect(isExpanded) {
                    if (isExpanded) companion.refreshOverview()
                    if (!isExpanded) {
                        kotlinx.coroutines.delay(FOCUS_RELEASE_DELAY_MS)
                    }
                    updateFocusableFlag(isExpanded)
                }

                NomiTheme(accent = appearance.paletteKey(accentColor), dark = dark) {
                    MessengerBubble(
                        state = uiState,
                        onTapHead = { companion.stateMachine.toggleExpanded() },
                        onOpenLink = ::launchMainActivityWithLink,
                        onOpenApp = { launchMainActivityWithLink("/") },
                        dockedRight = dockedRight,
                        liquidStretchX = liquidStretchX,
                        liquidStretchY = liquidStretchY,
                        onDismiss = {
                            dismissedUntilNextPeek = true
                            companion.stateMachine.collapse()
                            removeViews()
                        },
                        overview = overview,
                        onAsk = companion::ask,
                        onNewConversation = companion::newConversation,
                        onOpenConversation = { launchMainActivityWithLink(companion.conversationLink()) },
                    )
                }
            }
        }

        val touchSlop = ViewConfiguration.get(context).scaledTouchSlop
        val hostLayout = object : FrameLayout(context) {
            private var downRawX = 0f
            private var downRawY = 0f
            private var startAnchorX = 0
            private var startAnchorY = 0
            private var activeRawDrag = false

            override fun onLayout(changed: Boolean, left: Int, top: Int, right: Int, bottom: Int) {
                super.onLayout(changed, left, top, right, bottom)
                val measuredW = right - left
                val measuredH = bottom - top
                if (!activeRawDrag && measuredW > 0 && measuredH > 0) {
                    val params = bubbleParams ?: return
                    if (applyDockedGravityPosition(params, screenArea(), measuredW, measuredH)) {
                        post {
                            if (bubbleView === this) {
                                runCatching { windowManager.updateViewLayout(this, params) }
                            }
                        }
                    }
                }
            }

            override fun dispatchTouchEvent(ev: MotionEvent): Boolean {
                val isExpanded = companion.uiState.value is CompanionUiState.Expanded
                if (isExpanded) {
                    return super.dispatchTouchEvent(ev)
                }
                val params = bubbleParams ?: return super.dispatchTouchEvent(ev)
                when (ev.actionMasked) {
                    MotionEvent.ACTION_DOWN -> {
                        snapJob?.cancel()
                        liquidStretchX = BASE_SCALE
                        liquidStretchY = BASE_SCALE
                        downRawX = ev.rawX
                        downRawY = ev.rawY
                        startAnchorX = bubbleAnchorX
                        startAnchorY = bubbleAnchorY
                        activeRawDrag = false
                    }
                    MotionEvent.ACTION_MOVE -> {
                        val dx = ev.rawX - downRawX
                        val dy = ev.rawY - downRawY
                        if (!activeRawDrag && hypot(dx, dy) > touchSlop) {
                            activeRawDrag = true
                            dragging = true
                            capturedInDismiss = false
                            ensureDismissViewAttached()
                            val cancelEvent = MotionEvent.obtain(ev).apply { action = MotionEvent.ACTION_CANCEL }
                            super.dispatchTouchEvent(cancelEvent)
                            cancelEvent.recycle()
                        }
                        if (activeRawDrag) {
                            val currentArea = screenArea()
                            val clamped = clampBubblePosition(
                                point = BubblePointPx((startAnchorX + dx).roundToInt(), (startAnchorY + dy).roundToInt()),
                                bubbleSizePx = bubbleSizePx(),
                                area = currentArea,
                            )
                            bubbleAnchorX = clamped.x
                            bubbleAnchorY = clamped.y
                            params.gravity = Gravity.TOP or Gravity.LEFT
                            params.x = clamped.x
                            params.y = clamped.y
                            capturedInDismiss = isInsideDismissTarget(
                                bubbleTopLeft = clamped,
                                bubbleSizePx = bubbleSizePx(),
                                area = currentArea,
                                bottomOffsetPx = dismissOffsetPx(),
                                captureRadiusPx = dismissCaptureRadiusPx(),
                            )
                            runCatching { windowManager.updateViewLayout(this, params) }
                            return true
                        }
                    }
                    MotionEvent.ACTION_UP, MotionEvent.ACTION_CANCEL -> {
                        if (activeRawDrag) {
                            activeRawDrag = false
                            dragging = false
                            removeDismissView()
                            val currentArea = screenArea()
                            if (capturedInDismiss) {
                                capturedInDismiss = false
                                dismissedUntilNextPeek = true
                                companion.stateMachine.collapse()
                                removeViews()
                            } else {
                                val releaseX = bubbleAnchorX
                                val releaseY = bubbleAnchorY
                                val snapped = snapToEdge(
                                    point = BubblePointPx(releaseX, releaseY),
                                    bubbleSizePx = bubbleSizePx(),
                                    area = currentArea,
                                )
                                dockedRight = snapped.x >= currentArea.widthPx / HALF
                                val hostRef = this
                                snapJob?.cancel()
                                snapJob = scope.launch(ChoreographerFrameClock) {
                                    animate(
                                        initialValue = ZERO_FLOAT,
                                        targetValue = ONE_FLOAT,
                                        animationSpec = spring(
                                            dampingRatio = LIQUID_PULLBACK_DAMPING,
                                            stiffness = LIQUID_PULLBACK_STIFFNESS,
                                        ),
                                    ) { progress, _ ->
                                        bubbleAnchorX = (releaseX + (snapped.x - releaseX) * progress).roundToInt()
                                        bubbleAnchorY = (releaseY + (snapped.y - releaseY) * progress).roundToInt()
                                        val distanceX = abs(snapped.x - bubbleAnchorX).toFloat()
                                        val stretchProgress = (distanceX / LIQUID_STRETCH_DISTANCE_PX)
                                            .coerceIn(ZERO_FLOAT, ONE_FLOAT)
                                        liquidStretchX = BASE_SCALE + stretchProgress * LIQUID_STRETCH_MAX_EXTRA_X
                                        liquidStretchY = BASE_SCALE - stretchProgress * LIQUID_STRETCH_MAX_SQUISH_Y
                                        val latestArea = screenArea()
                                        applyDockedGravityPosition(
                                            params = params,
                                            area = latestArea,
                                            viewWidthPx = hostRef.width.coerceAtLeast(bubbleSizePx()),
                                            viewHeightPx = hostRef.height.coerceAtLeast(bubbleSizePx()),
                                        )
                                        if (bubbleView === hostRef) {
                                            runCatching { windowManager.updateViewLayout(hostRef, params) }
                                        }
                                    }
                                    bubbleAnchorX = snapped.x
                                    bubbleAnchorY = snapped.y
                                    liquidStretchX = BASE_SCALE
                                    liquidStretchY = BASE_SCALE
                                    val finalArea = screenArea()
                                    applyDockedGravityPosition(
                                        params = params,
                                        area = finalArea,
                                        viewWidthPx = hostRef.width.coerceAtLeast(bubbleSizePx()),
                                        viewHeightPx = hostRef.height.coerceAtLeast(bubbleSizePx()),
                                    )
                                    if (bubbleView === hostRef) {
                                        runCatching { windowManager.updateViewLayout(hostRef, params) }
                                    }
                                }
                            }
                            return true
                        }
                    }
                }
                return super.dispatchTouchEvent(ev)
            }
        }.apply {
            owner.attachTo(this)
            addView(composeView)
        }

        bubbleView = hostLayout
        runCatching { windowManager.addView(hostLayout, bParams) }
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
        snapJob?.cancel()
        snapJob = null
        liquidStretchX = BASE_SCALE
        liquidStretchY = BASE_SCALE
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
