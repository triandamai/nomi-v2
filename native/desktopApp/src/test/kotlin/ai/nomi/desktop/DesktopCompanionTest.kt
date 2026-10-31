package ai.nomi.desktop

import androidx.compose.ui.unit.IntRect
import java.nio.file.Files
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNotNull
import kotlin.test.assertNull
import kotlin.test.assertTrue

class DesktopCompanionTest {
    @Test
    fun detectsWaylandAndDisablesIsland() {
        assertEquals(
            DesktopOs.LinuxWayland,
            detectDesktopOs("Linux", mapOf("XDG_SESSION_TYPE" to "wayland")),
        )
        assertFalse(DesktopOs.LinuxWayland.supportsIsland)

        assertEquals(
            DesktopOs.LinuxX11,
            detectDesktopOs("Linux", mapOf("XDG_SESSION_TYPE" to "x11")),
        )
        assertTrue(DesktopOs.LinuxX11.supportsIsland)
        assertTrue(detectDesktopOs("Mac OS X", emptyMap()).supportsIsland)
        assertTrue(detectDesktopOs("Windows 11", emptyMap()).supportsIsland)
    }

    @Test
    fun islandGeometryHandlesNotchAndMultipleMonitors() {
        val primary = DisplayBoundsDp(xDp = 0, yDp = 0, widthDp = 1512, heightDp = 982, menuBarHeightDp = 38, notchWidthDp = 180)
        val external = DisplayBoundsDp(xDp = 1512, yDp = 0, widthDp = 2560, heightDp = 1440, menuBarHeightDp = 24, notchWidthDp = null)

        val notchedPlacement = computeIslandPlacement(primary, isExpanded = false)
        assertEquals(0, notchedPlacement.windowYDp)
        assertEquals(180, notchedPlacement.collapsedPillWidthDp)

        val picked = selectDisplayForCursor(listOf(primary, external), cursorXDp = 2100, cursorYDp = 300)
        assertEquals(external, picked)

        val externalPlacement = computeIslandPlacement(picked, isExpanded = false)
        assertEquals(24 + ISLAND_TOP_GAP_DP, externalPlacement.windowYDp)
        assertEquals(ISLAND_COLLAPSED_WIDTH_DP, externalPlacement.collapsedPillWidthDp)
    }

    @Test
    fun peekAndOpenPanelStartBelowTheNotch() {
        val notched = DisplayBoundsDp(xDp = 0, yDp = 0, widthDp = 1512, heightDp = 982, menuBarHeightDp = 38, notchWidthDp = 180)
        val plain = DisplayBoundsDp(xDp = 0, yDp = 0, widthDp = 2560, heightDp = 1440, menuBarHeightDp = 24, notchWidthDp = null)

        val peek = computeIslandPlacement(notched, isExpanded = false, isPeeking = true)
        assertEquals(ISLAND_PEEK_WIDTH_DP, peek.pillBoundsInScreenDp.width)
        assertEquals(ISLAND_PEEK_HEIGHT_DP + 38, peek.pillBoundsInScreenDp.height)

        val open = computeIslandPlacement(notched, isExpanded = true)
        assertEquals(ISLAND_EXPANDED_WIDTH_DP, open.pillBoundsInScreenDp.width)
        assertEquals(ISLAND_EXPANDED_HEIGHT_DP + 38, open.pillBoundsInScreenDp.height)
        // The window never resizes: it already fits the open panel.
        assertTrue(open.windowHeightDp >= open.pillBoundsInScreenDp.height)
        assertEquals(peek.windowHeightDp, open.windowHeightDp)

        assertEquals(ISLAND_EXPANDED_HEIGHT_DP, computeIslandPlacement(plain, isExpanded = true).pillBoundsInScreenDp.height)
    }

    @Test
    fun onlyATallMenuBarMeansANotch() {
        assertTrue(hasNotch(38))
        assertTrue(hasNotch(32))
        assertFalse(hasNotch(24))
        assertFalse(hasNotch(25))
    }

    @Test
    fun cursorSampleComputesHoverAndEyeTrackingVector() {
        val pill = IntRect(left = 600, top = 10, right = 800, bottom = 46)
        val inside = evaluateCursorSample(cursorXDp = 700, cursorYDp = 28, pillBounds = pill)
        assertTrue(inside.insideIsland)
        assertEquals(0f, inside.distanceDp)
        assertNotNull(inside.lookAt)

        val nearby = evaluateCursorSample(cursorXDp = 850, cursorYDp = 60, pillBounds = pill)
        assertFalse(nearby.insideIsland)
        assertTrue(nearby.distanceDp <= NEAR_DISTANCE_DP)
        val lookAt = assertNotNull(nearby.lookAt)
        assertTrue(lookAt.x > 0f)

        val far = evaluateCursorSample(cursorXDp = 100, cursorYDp = 800, pillBounds = pill)
        assertFalse(far.insideIsland)
        assertNull(far.lookAt)
    }

    @Test
    fun autostartWritesLaunchAgentDesktopEntryAndRegistry() {
        val tempHome = Files.createTempDirectory("nomi-autostart-test")
        try {
            val mac = AutostartManager(DesktopOs.MacOS, tempHome, "/Applications/Nomi.app/Contents/MacOS/Nomi")
            assertFalse(mac.isEnabled())
            mac.enable()
            assertTrue(mac.isEnabled())
            assertTrue(Files.readString(mac.macLaunchAgentPath).contains("--background"))
            mac.disable()
            assertFalse(mac.isEnabled())

            val linux = AutostartManager(DesktopOs.LinuxX11, tempHome, "/usr/bin/nomi")
            linux.enable()
            assertTrue(linux.isEnabled())
            assertTrue(Files.readString(linux.linuxDesktopEntryPath).contains("X-GNOME-Autostart-enabled=true"))
            linux.disable()
            assertFalse(linux.isEnabled())

            val fakeRegistry = object : RegistryAdapter {
                val values = mutableMapOf<Pair<String, String>, String>()
                override fun setStringValue(keyPath: String, name: String, value: String) {
                    values[keyPath to name] = value
                }
                override fun deleteValue(keyPath: String, name: String) {
                    values.remove(keyPath to name)
                }
                override fun getStringValue(keyPath: String, name: String): String? = values[keyPath to name]
            }
            val win = AutostartManager(DesktopOs.Windows, tempHome, "C:\\Program Files\\Nomi\\Nomi.exe", fakeRegistry)
            win.enable()
            assertTrue(win.isEnabled())
            win.disable()
            assertFalse(win.isEnabled())
        } finally {
            tempHome.toFile().deleteRecursively()
        }
    }
}
