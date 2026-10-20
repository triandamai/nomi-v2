package ai.nomi.app

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

class PlatformDesktopTest {
    @Test
    fun defaultDesktopApiUrlUsesHttps() {
        assertEquals("https://api-nomi.trian.space", DEFAULT_DESKTOP_API_URL)
        assertTrue(DEFAULT_DESKTOP_API_URL.startsWith("https://"))
    }

    @Test
    fun desktopApiUrlDefaultsToHttpsUrl() {
        // Without system properties or env overrides, it must point to https
        val originalProp = System.getProperty(API_URL_PROPERTY)
        try {
            System.clearProperty(API_URL_PROPERTY)
            val url = desktopApiUrl()
            assertEquals("https://api-nomi.trian.space", url)
        } finally {
            if (originalProp != null) {
                System.setProperty(API_URL_PROPERTY, originalProp)
            }
        }
    }
}
