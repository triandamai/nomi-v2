package ai.nomi.feature.chat.views

import kotlinx.datetime.TimeZone
import kotlin.test.Test
import kotlin.test.assertEquals

class ViewFormatTest {
    private val utc = TimeZone.UTC

    @Test
    fun moneyHasTwoDecimalsAndGrouping() {
        assertEquals("45,000.00", formatValue(Scalar.Num(45000.0), ValueFormat.Money, utc))
        assertEquals("212,750.50", formatValue(Scalar.Num(212750.5), ValueFormat.Money, utc))
        assertEquals("-12.50", formatValue(Scalar.Num(-12.5), ValueFormat.Money, utc))
    }

    @Test
    fun numbersPercentsAndBooleans() {
        assertEquals("1,234,567", formatValue(Scalar.Num(1234567.0), ValueFormat.Number, utc))
        assertEquals("1,234.5", formatValue(Scalar.Num(1234.5), null, utc))
        assertEquals("40%", formatValue(Scalar.Num(0.4), ValueFormat.Percent, utc))
        assertEquals("12.5%", formatValue(Scalar.Num(12.5), ValueFormat.Percent, utc))
        assertEquals("✓", formatValue(Scalar.Bool(true), null, utc))
        assertEquals("—", formatValue(Scalar.Bool(false), ValueFormat.Money, utc))
        assertEquals("1.24 kg", formatValue(Scalar.Text("1.24 kg"), null, utc))
        assertEquals("45,000.00", formatValue(Scalar.Text("45000"), ValueFormat.Money, utc))
    }

    @Test
    fun datesInTheMediumStyle() {
        assertEquals("Oct 12, 2026", formatValue(Scalar.Text("2026-10-12"), ValueFormat.Date, utc))
        assertEquals("Oct 24, 2026, 1:00 AM", formatValue(Scalar.Text("2026-10-24T08:00:00+07:00"), ValueFormat.DateTime, utc))
        assertEquals("soon", formatValue(Scalar.Text("soon"), ValueFormat.Date, utc))
    }

    @Test
    fun timelineWhenSplitsDayAndTime() {
        assertEquals(When("Sat, Oct 24", "01:00 AM"), formatWhen("2026-10-24T08:00:00+07:00", utc))
        assertEquals(When("Mon, Oct 12", null), formatWhen("2026-10-12", utc))
        assertEquals(When("Tomorrow", null), formatWhen("Tomorrow", utc))
    }

    @Test
    fun chartScaleRoundsUpToNiceSteps() {
        assertEquals(listOf(0.0, 25.0, 50.0, 75.0, 100.0), chartScale(100.0))
        assertEquals(listOf(0.0, 500_000.0, 1_000_000.0, 1_500_000.0), chartScale(1_500_000.0))
        assertEquals(listOf(0.0, 1.0), chartScale(0.0))
    }

    @Test
    fun ticksAreShort() {
        assertEquals("1.3M", formatTick(1_250_000.0, ValueFormat.Money))
        assertEquals("12.5K", formatTick(12_500.0, null))
        assertEquals("500", formatTick(500.0, null))
        assertEquals("40%", formatTick(0.4, ValueFormat.Percent))
    }

    @Test
    fun formAndQuestionAnswersBecomeOneMessage() {
        val fields = listOf("destination" to "Where to", "nights" to "Nights", "notes" to "Anything else")
        assertEquals("Trip details\nWhere to: Bandung\nNights: 2", composeForm("Trip details", fields, mapOf("destination" to " Bandung ", "nights" to "2", "notes" to "")))
        assertEquals("", composeForm(null, fields, emptyMap()))
        assertEquals(
            "Which day? Saturday\nWho is coming? Ana, Budi\nBring snacks",
            composeAnswers(listOf("Which day?", "Who is coming?", "Budget?"), listOf(listOf("Saturday"), listOf("Ana", "Budi"), emptyList()), " Bring snacks "),
        )
    }
}
