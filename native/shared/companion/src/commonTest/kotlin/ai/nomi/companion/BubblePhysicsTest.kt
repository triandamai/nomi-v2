package ai.nomi.companion

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertTrue

class BubblePhysicsTest {
    private val area = ScreenAreaPx(
        widthPx = 1080,
        heightPx = 2400,
        marginPx = 24,
        topInsetPx = 80,
        bottomInsetPx = 100,
    )
    private val bubbleSize = 180

    @Test
    fun snapsToNearestHorizontalEdgeAndClampsVerticalInsets() {
        val leftRelease = snapToEdge(BubblePointPx(x = 200, y = 10), bubbleSize, area)
        assertEquals(24, leftRelease.x)
        assertEquals(104, leftRelease.y) // topInsetPx (80) + marginPx (24)

        val rightRelease = snapToEdge(BubblePointPx(x = 600, y = 2500), bubbleSize, area)
        assertEquals(1080 - 180 - 24, rightRelease.x)
        assertEquals(2400 - 100 - 180 - 24, rightRelease.y)
    }

    @Test
    fun flingVelocityOverridesMidpointSide() {
        val flungRightFromLeft = snapToEdge(
            point = BubblePointPx(x = 120, y = 500),
            bubbleSizePx = bubbleSize,
            area = area,
            velocityXPxPerSec = 1200f,
        )
        assertEquals(1080 - 180 - 24, flungRightFromLeft.x)
    }

    @Test
    fun detectsDropInsideBottomCenterDismissTarget() {
        val target = dismissTargetCenter(area, bottomOffsetPx = 160)
        val insideTopLeft = BubblePointPx(x = target.x - bubbleSize / 2, y = target.y - bubbleSize / 2)
        assertTrue(
            isInsideDismissTarget(
                bubbleTopLeft = insideTopLeft,
                bubbleSizePx = bubbleSize,
                area = area,
                bottomOffsetPx = 160,
                captureRadiusPx = 140,
            ),
        )

        val farTopLeft = BubblePointPx(x = 24, y = 400)
        assertFalse(
            isInsideDismissTarget(
                bubbleTopLeft = farTopLeft,
                bubbleSizePx = bubbleSize,
                area = area,
                bottomOffsetPx = 160,
                captureRadiusPx = 140,
            ),
        )
    }

    @Test
    fun remeasuresExpandedOverlayWindowPositionInsideScreenBounds() {
        val rightDockedAnchor = BubblePointPx(x = 1080 - bubbleSize - 24, y = 2100)
        val expandedWidth = 840
        val expandedHeight = 600
        val resolved = resolveOverlayWindowPosition(
            bubbleAnchor = rightDockedAnchor,
            bubbleSizePx = bubbleSize,
            viewWidthPx = expandedWidth,
            viewHeightPx = expandedHeight,
            dockedRight = true,
            area = area,
        )
        assertEquals(1080 - expandedWidth - 24, resolved.x)
        assertEquals(2400 - 100 - expandedHeight - 24, resolved.y)
    }
}
