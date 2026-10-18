package ai.nomi.companion

import kotlin.math.hypot

private const val HALF = 2

/** Bounds and insets of the screen in pixels, used for Messenger-style edge snapping and dismiss hit-testing. */
data class ScreenAreaPx(
    val widthPx: Int,
    val heightPx: Int,
    val marginPx: Int,
    val topInsetPx: Int = 0,
    val bottomInsetPx: Int = 0,
)

/** Pixel coordinates of the top-left corner of the floating bubble. */
data class BubblePointPx(val x: Int, val y: Int)

/**
 * Clamps [point] so a bubble of [bubbleSizePx] stays inside [area]'s safe vertical and horizontal bounds.
 */
fun clampBubblePosition(point: BubblePointPx, bubbleSizePx: Int, area: ScreenAreaPx): BubblePointPx {
    val minX = area.marginPx
    val maxX = (area.widthPx - bubbleSizePx - area.marginPx).coerceAtLeast(minX)
    val minY = area.topInsetPx + area.marginPx
    val maxY = (area.heightPx - area.bottomInsetPx - bubbleSizePx - area.marginPx).coerceAtLeast(minY)
    return BubblePointPx(
        x = point.x.coerceIn(minX, maxX),
        y = point.y.coerceIn(minY, maxY),
    )
}

/**
 * Snaps a released bubble at [point] (with optional horizontal velocity [velocityXPxPerSec]) to the left or right
 * screen edge, clamping its vertical coordinate to the safe area.
 */
fun snapToEdge(
    point: BubblePointPx,
    bubbleSizePx: Int,
    area: ScreenAreaPx,
    velocityXPxPerSec: Float = 0f,
    flingThresholdPxPerSec: Float = 800f,
): BubblePointPx {
    val clamped = clampBubblePosition(point, bubbleSizePx, area)
    val leftX = area.marginPx
    val rightX = (area.widthPx - bubbleSizePx - area.marginPx).coerceAtLeast(leftX)
    val centerX = point.x + bubbleSizePx / HALF
    val snapRight = when {
        velocityXPxPerSec >= flingThresholdPxPerSec -> true
        velocityXPxPerSec <= -flingThresholdPxPerSec -> false
        else -> centerX >= area.widthPx / HALF
    }
    return BubblePointPx(
        x = if (snapRight) rightX else leftX,
        y = clamped.y,
    )
}

/**
 * Center of the bottom-center dismiss target in screen pixels.
 */
fun dismissTargetCenter(area: ScreenAreaPx, bottomOffsetPx: Int): BubblePointPx = BubblePointPx(
    x = area.widthPx / HALF,
    y = area.heightPx - area.bottomInsetPx - bottomOffsetPx,
)

/**
 * True when the center of a bubble at [bubbleTopLeft] (size [bubbleSizePx]) is within [captureRadiusPx] of the
 * bottom-center dismiss target.
 */
fun isInsideDismissTarget(
    bubbleTopLeft: BubblePointPx,
    bubbleSizePx: Int,
    area: ScreenAreaPx,
    bottomOffsetPx: Int,
    captureRadiusPx: Int,
): Boolean {
    val target = dismissTargetCenter(area, bottomOffsetPx)
    val bubbleCenterX = bubbleTopLeft.x + bubbleSizePx / HALF
    val bubbleCenterY = bubbleTopLeft.y + bubbleSizePx / HALF
    val distance = hypot((bubbleCenterX - target.x).toDouble(), (bubbleCenterY - target.y).toDouble())
    return distance <= captureRadiusPx
}
