package ai.nomi.feature.chat.views

import kotlinx.datetime.LocalDate
import kotlinx.datetime.LocalDateTime
import kotlinx.datetime.TimeZone
import kotlinx.datetime.toLocalDateTime
import kotlin.math.abs
import kotlin.math.ceil
import kotlin.math.floor
import kotlin.math.log10
import kotlin.math.pow
import kotlin.math.roundToLong
import kotlin.time.Instant

// How a view shows a value (frontend/src/lib/views/format.ts), in English like the web's default locale: money in
// whole units with two decimals, numbers grouped, 0.4 and 40 both as 40%, ISO dates in the medium style.

private const val MONEY_DECIMALS = 2
private const val NUMBER_MAX_DECIMALS = 3
private const val PERCENT_MAX_DECIMALS = 1
private const val COMPACT_DECIMALS = 1
private const val COMPACT_FROM = 10_000.0
private const val HUNDRED = 100.0
private const val GROUP = 3
private const val TEN = 10.0
private const val NOON = 12
private const val HOURS_HALF_DAY = 12
private const val PAD_TWO = 2
private const val DEFAULT_STEPS = 4
private val NiceFactors = listOf(1.0, 2.0, 2.5, 5.0, 10.0)
private val CompactUnits = listOf(1e12 to "T", 1e9 to "B", 1e6 to "M", 1e3 to "K")

private val IsoDate = Regex("""^\d{4}-\d{2}-\d{2}""")
private val BareDate = Regex("""^\d{4}-\d{2}-\d{2}$""")
private val HasTime = Regex("""T\d{2}:\d{2}""")
private val HasOffset = Regex("""(Z|[+-]\d{2}:?\d{2})$""")

private val MonthsShort = listOf("Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec")
private val WeekdaysShort = listOf("Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun")

/** A view value as text, the web's `formatValue`. */
fun formatValue(value: Scalar, format: ValueFormat?, timeZone: TimeZone): String {
    if (value is Scalar.Bool) return if (value.value) "✓" else "—"
    return when (format) {
        ValueFormat.Money -> value.asNumber()?.let { decimal(it, MONEY_DECIMALS, MONEY_DECIMALS) } ?: value.raw()
        ValueFormat.Number -> value.asNumber()?.let { decimal(it, 0, NUMBER_MAX_DECIMALS) } ?: value.raw()
        ValueFormat.Percent -> value.asNumber()?.let { n -> decimal((if (abs(n) <= 1) n else n / HUNDRED) * HUNDRED, 0, PERCENT_MAX_DECIMALS) + "%" } ?: value.raw()
        ValueFormat.Date, ValueFormat.DateTime -> value.asDateTime(timeZone)?.let { at ->
            val day = "${MonthsShort[at.month.ordinal]} ${at.day}, ${at.year}"
            if (format == ValueFormat.Date) day else "$day, ${clock12(at, padHour = false)}"
        } ?: value.raw()
        else -> if (value is Scalar.Num) decimal(value.value, 0, NUMBER_MAX_DECIMALS) else value.raw()
    }
}

/** A timeline's `at`: the day ("Sat, Oct 24") and, when it carries one, the time ("08:00 AM"). */
data class When(val day: String, val time: String?)

fun formatWhen(at: String, timeZone: TimeZone): When {
    val date = Scalar.Text(at).asDateTime(timeZone) ?: return When(at, null)
    val day = "${WeekdaysShort[date.dayOfWeek.ordinal]}, ${MonthsShort[date.month.ordinal]} ${date.day}"
    return When(day, if (HasTime.containsMatchIn(at)) clock12(date, padHour = true) else null)
}

/** Axis ticks: the largest value rounded up to a round number, in a few steps. */
fun chartScale(max: Double, steps: Int = DEFAULT_STEPS): List<Double> {
    if (!(max > 0)) return listOf(0.0, 1.0)
    val raw = max / steps
    val magnitude = TEN.pow(floor(log10(raw)))
    val step = NiceFactors.map { it * magnitude }.firstOrNull { it >= raw } ?: raw
    return (0..ceil(max / step).toInt()).map { i -> (i * step * TEN.pow(NUMBER_MAX_DECIMALS)).roundToLong() / TEN.pow(NUMBER_MAX_DECIMALS) }
}

/** An axis label: like [formatValue], but short (12.5K) so ticks fit. */
fun formatTick(value: Double, format: ValueFormat?): String {
    if (format == ValueFormat.Percent) return formatValue(Scalar.Num(value), format, TimeZone.UTC)
    if (format == ValueFormat.Money || abs(value) >= COMPACT_FROM) {
        val (unit, suffix) = CompactUnits.firstOrNull { abs(value) >= it.first } ?: return decimal(value, 0, COMPACT_DECIMALS)
        return decimal(value / unit, 0, COMPACT_DECIMALS) + suffix
    }
    return decimal(value, 0, NUMBER_MAX_DECIMALS)
}

/** A form's answers as one message, in field order: "Label: value" lines, under the form's title. */
fun composeForm(title: String?, fields: List<Pair<String, String>>, values: Map<String, String>): String {
    val lines = fields.mapNotNull { (key, label) -> values[key]?.trim()?.takeIf { it.isNotEmpty() }?.let { "$label: $it" } }
    if (lines.isEmpty()) return ""
    return (listOfNotNull(title) + lines).joinToString("\n")
}

/** The questions card's answers: "Question picked, picked" per answered question, then the free note. */
fun composeAnswers(questions: List<String>, picked: List<List<String>>, note: String): String {
    val lines = questions.mapIndexedNotNull { i, q -> picked.getOrNull(i)?.takeIf { it.isNotEmpty() }?.let { "$q ${it.joinToString(", ")}" } }.toMutableList()
    note.trim().takeIf { it.isNotEmpty() }?.let(lines::add)
    return lines.joinToString("\n")
}

private fun Scalar.raw(): String = when (this) {
    is Scalar.Text -> value
    is Scalar.Num -> decimal(value, 0, NUMBER_MAX_DECIMALS)
    is Scalar.Bool -> if (value) "✓" else "—"
}

private fun Scalar.asNumber(): Double? = when (this) {
    is Scalar.Num -> value.takeIf { it.isFinite() }
    is Scalar.Text -> value.trim().takeIf { it.isNotEmpty() }?.toDoubleOrNull()
    is Scalar.Bool -> null
}

/** An ISO date or date-time; a bare date is that calendar day (at noon), not midnight UTC. */
private fun Scalar.asDateTime(timeZone: TimeZone): LocalDateTime? {
    val text = (this as? Scalar.Text)?.value ?: return null
    if (!IsoDate.containsMatchIn(text)) return null
    return runCatching {
        when {
            BareDate.matches(text) -> LocalDate.parse(text).let { LocalDateTime(it.year, it.month, it.day, NOON, 0) }
            HasOffset.containsMatchIn(text) -> Instant.parse(text).toLocalDateTime(timeZone)
            else -> LocalDateTime.parse(text)
        }
    }.getOrNull()
}

/** "1:00 AM", or "01:00 AM" when [padHour] (the timeline's 2-digit style). */
private fun clock12(at: LocalDateTime, padHour: Boolean): String {
    val hour = (at.hour % HOURS_HALF_DAY).let { if (it == 0) HOURS_HALF_DAY else it }
    val h = if (padHour) hour.toString().padStart(PAD_TWO, '0') else hour.toString()
    return "$h:${at.minute.toString().padStart(PAD_TWO, '0')} ${if (at.hour < HOURS_HALF_DAY) "AM" else "PM"}"
}

/** [value] with grouped thousands and between [minDecimals] and [maxDecimals] fraction digits, half up. */
private fun decimal(value: Double, minDecimals: Int, maxDecimals: Int): String {
    val scale = TEN.pow(maxDecimals)
    val scaled = (abs(value) * scale).roundToLong()
    val whole = (scaled / scale.toLong()).toString()
    var fraction = (scaled % scale.toLong()).toString().padStart(maxDecimals, '0')
    while (fraction.length > minDecimals && fraction.endsWith('0')) fraction = fraction.dropLast(1)
    val grouped = whole.reversed().chunked(GROUP).joinToString(",").reversed()
    val sign = if (value < 0 && scaled != 0L) "-" else ""
    return sign + grouped + if (fraction.isEmpty()) "" else ".$fraction"
}
