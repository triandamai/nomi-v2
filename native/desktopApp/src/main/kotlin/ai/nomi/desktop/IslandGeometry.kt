package ai.nomi.desktop

import androidx.compose.ui.unit.IntRect

const val ISLAND_COLLAPSED_WIDTH_DP = 172
const val ISLAND_WORKING_WIDTH_DP = 248
const val ISLAND_EXPANDED_WIDTH_DP = 360
const val ISLAND_COLLAPSED_HEIGHT_DP = 36
const val ISLAND_EXPANDED_HEIGHT_DP = 180
const val ISLAND_WINDOW_PADDING_DP = 16
const val ISLAND_TOP_GAP_DP = 6
const val ISLAND_NOTCH_MARGIN_DP = 28
const val DEFAULT_MENU_BAR_HEIGHT_DP = 28
private const val HALF = 2

/** Bounds and top-edge characteristics of one desktop monitor in dp. */
data class DisplayBoundsDp(
    val xDp: Int,
    val yDp: Int,
    val widthDp: Int,
    val heightDp: Int,
    val menuBarHeightDp: Int = DEFAULT_MENU_BAR_HEIGHT_DP,
    val notchWidthDp: Int? = null,
) {
    fun contains(pxDp: Int, pyDp: Int): Boolean =
        pxDp in xDp until (xDp + widthDp) && pyDp in yDp until (yDp + heightDp)
}

/**
 * Computed outer window frame (sized to expanded bounds so expanding never resizes the OS window) and the current
 * interactive pill rectangle in screen dp (used by [CursorTracker] for hover and click-through toggling).
 */
data class IslandPlacementDp(
    val windowXDp: Int,
    val windowYDp: Int,
    val windowWidthDp: Int,
    val windowHeightDp: Int,
    val collapsedPillWidthDp: Int,
    val pillBoundsInScreenDp: IntRect,
)

/** Picks the display containing `(cursorXDp, cursorYDp)`, falling back to the primary display. */
fun selectDisplayForCursor(
    displays: List<DisplayBoundsDp>,
    cursorXDp: Int?,
    cursorYDp: Int?,
): DisplayBoundsDp {
    require(displays.isNotEmpty()) { "At least one display is required" }
    if (cursorXDp == null || cursorYDp == null) return displays.first()
    return displays.firstOrNull { it.contains(cursorXDp, cursorYDp) } ?: displays.first()
}

/**
 * Computes the island window placement and current pill hit rect on [display].
 *
 * - On a notched Mac (`notchWidthDp != null`), the island hugs `y = display.yDp` and its collapsed width is at least
 *   `notchWidthDp + 2 * ISLAND_NOTCH_MARGIN_DP`.
 * - On a non-notched screen, the island sits at `display.yDp + display.menuBarHeightDp + ISLAND_TOP_GAP_DP`.
 */
fun computeIslandPlacement(
    display: DisplayBoundsDp,
    isExpanded: Boolean,
    isWorking: Boolean = false,
): IslandPlacementDp {
    val collapsedWidth = display.notchWidthDp
        ?.let { maxOf(ISLAND_COLLAPSED_WIDTH_DP, it + ISLAND_NOTCH_MARGIN_DP * HALF) }
        ?: ISLAND_COLLAPSED_WIDTH_DP
    val activePillWidth = when {
        isExpanded -> ISLAND_EXPANDED_WIDTH_DP
        isWorking -> maxOf(collapsedWidth, ISLAND_WORKING_WIDTH_DP)
        else -> collapsedWidth
    }
    val activePillHeight = if (isExpanded) ISLAND_EXPANDED_HEIGHT_DP else ISLAND_COLLAPSED_HEIGHT_DP

    val windowWidth = maxOf(ISLAND_EXPANDED_WIDTH_DP, collapsedWidth) + ISLAND_WINDOW_PADDING_DP * HALF
    val windowHeight = ISLAND_EXPANDED_HEIGHT_DP + ISLAND_WINDOW_PADDING_DP * HALF
    val windowX = display.xDp + (display.widthDp - windowWidth) / HALF
    val pillTopInScreen = if (display.notchWidthDp != null) {
        display.yDp
    } else {
        display.yDp + display.menuBarHeightDp + ISLAND_TOP_GAP_DP
    }
    val windowY = pillTopInScreen

    val pillLeftInScreen = display.xDp + (display.widthDp - activePillWidth) / HALF
    val pillRect = IntRect(
        left = pillLeftInScreen,
        top = pillTopInScreen,
        right = pillLeftInScreen + activePillWidth,
        bottom = pillTopInScreen + activePillHeight,
    )

    return IslandPlacementDp(
        windowXDp = windowX,
        windowYDp = windowY,
        windowWidthDp = windowWidth,
        windowHeightDp = windowHeight,
        collapsedPillWidthDp = collapsedWidth,
        pillBoundsInScreenDp = pillRect,
    )
}
