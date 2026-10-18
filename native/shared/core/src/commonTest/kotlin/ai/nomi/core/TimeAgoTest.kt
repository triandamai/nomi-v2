package ai.nomi.core

import kotlin.test.Test
import kotlin.test.assertEquals

class TimeAgoTest {
    private val now = 1_800_000_000_000L
    private fun ago(minutes: Long) = timeAgo(now - minutes * 60_000L, now)

    @Test fun underAMinuteIsJustNow() = assertEquals("just now", ago(0))
    @Test fun minutesHoursAndDays() {
        assertEquals("5m ago", ago(5))
        assertEquals("2h ago", ago(2 * 60))
        assertEquals("yesterday", ago(24 * 60))
        assertEquals("3d ago", ago(3 * 24 * 60))
    }
    @Test fun monthsAndYears() {
        assertEquals("last mo.", ago(30L * 24 * 60))
        assertEquals("4mo ago", ago(4 * 30L * 24 * 60))
        assertEquals("last yr.", ago(12 * 30L * 24 * 60))
        assertEquals("2y ago", ago(24 * 30L * 24 * 60))
    }
    @Test fun aFutureTimeIsJustNow() = assertEquals("just now", timeAgo(now + 60_000L, now))
}
