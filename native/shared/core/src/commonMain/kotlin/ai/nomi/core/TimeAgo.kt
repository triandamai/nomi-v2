package ai.nomi.core

private const val MS_PER_MINUTE = 60_000L
private const val MINUTES_PER_HOUR = 60L
private const val HOURS_PER_DAY = 24L
private const val DAYS_PER_MONTH = 30L
private const val MONTHS_PER_YEAR = 12L
private const val ONE = 1L

/**
 * How long ago [epochMs] was, the web's `timeAgo` (`Intl.RelativeTimeFormat`, narrow, numeric "auto", English):
 * "just now", "5m ago", "2h ago", "yesterday", "3d ago", "last mo.", "4mo ago", "last yr.", "2y ago".
 */
fun timeAgo(epochMs: Long, nowMs: Long): String {
    val minutes = (nowMs - epochMs) / MS_PER_MINUTE
    if (minutes < ONE) return "just now"
    if (minutes < MINUTES_PER_HOUR) return "${minutes}m ago"
    val hours = minutes / MINUTES_PER_HOUR
    if (hours < HOURS_PER_DAY) return "${hours}h ago"
    val days = hours / HOURS_PER_DAY
    if (days < DAYS_PER_MONTH) return if (days == ONE) "yesterday" else "${days}d ago"
    val months = days / DAYS_PER_MONTH
    if (months < MONTHS_PER_YEAR) return if (months == ONE) "last mo." else "${months}mo ago"
    val years = months / MONTHS_PER_YEAR
    return if (years == ONE) "last yr." else "${years}y ago"
}
