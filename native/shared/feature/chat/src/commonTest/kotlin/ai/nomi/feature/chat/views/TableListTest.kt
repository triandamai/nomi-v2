package ai.nomi.feature.chat.views

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class TableListTest {
    private fun col(key: String, label: String = key.replaceFirstChar { it.uppercase() }, format: ValueFormat? = null) = ChatView.Column(key, label, format)

    @Test
    fun spendingTableLeadsWithWhatAndEndsWithTheAmount() {
        val columns = listOf(col("date", format = ValueFormat.Date), col("what"), col("amount", format = ValueFormat.Money))
        val rows = listOf(
            mapOf("date" to Scalar.Text("2026-10-12"), "what" to Scalar.Text("Lunch"), "amount" to Scalar.Num(45000.0)),
            mapOf("date" to Scalar.Text("2026-10-13"), "what" to Scalar.Text("Grab"), "amount" to Scalar.Num(28500.0)),
        )
        val parts = tableListColumns(columns, rows)
        assertEquals("what", parts.headline?.key)
        assertEquals("amount", parts.trailing?.key)
        assertEquals(listOf("date"), parts.supporting.map { it.key })
    }

    @Test
    fun categoryColumnIsPickedOutAndANumericLastColumnTrails() {
        val columns = listOf(col("merchant"), col("category"), col("visits"))
        val rows = listOf(mapOf("merchant" to Scalar.Text("Kopi"), "category" to Scalar.Text("food"), "visits" to Scalar.Num(3.0)))
        val parts = tableListColumns(columns, rows)
        assertEquals("merchant", parts.headline?.key)
        assertEquals("category", parts.category?.key)
        assertEquals("visits", parts.trailing?.key)
    }

    @Test
    fun allTextTableHasNoTrailing() {
        val parts = tableListColumns(listOf(col("name"), col("note")), listOf(mapOf("name" to Scalar.Text("A"), "note" to Scalar.Text("b"))))
        assertEquals("name", parts.headline?.key)
        assertNull(parts.trailing)
    }
}
