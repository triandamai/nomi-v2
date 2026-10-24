package ai.nomi.feature.chat.views

// How a table reads as a list on a narrow screen ($lib/tableList.ts): which column names the row, which number sits on
// the right, which is a category, and what's left for the line under the headline. Decided on the raw values (so an
// ISO date still reads as a date, where the web's view tables test already-formatted text).

private val Amount = Regex("amount|total|price|cost|spent|sum|balance|value|jumlah|nominal|harga|biaya", RegexOption.IGNORE_CASE)
private val Category = Regex("^(category|kategori|type)$", RegexOption.IGNORE_CASE)
private val Numberish = Regex("""^[^\d-]{0,4}-?[\d.,\s]+[^\d]{0,4}$""")
private val Dateish = Regex("""^\d{4}-\d{2}-\d{2}|^\d{1,2}[/ -]\w{1,9}[/ -]\d{2,4}|^(mon|tue|wed|thu|fri|sat|sun)""", RegexOption.IGNORE_CASE)
private const val MOSTLY = 0.6

data class TableListColumns(
    val headline: ChatView.Column?,
    val trailing: ChatView.Column?,
    val category: ChatView.Column?,
    val supporting: List<ChatView.Column>,
)

private fun Scalar?.text(): String = when (this) {
    null -> ""
    is Scalar.Text -> value.trim()
    is Scalar.Num -> value.toString().removeSuffix(".0")
    is Scalar.Bool -> value.toString()
}

private fun List<Map<String, Scalar?>>.mostly(key: String, test: (String) -> Boolean): Boolean {
    val values = map { it[key].text() }.filter { it.isNotEmpty() }
    return values.isNotEmpty() && values.count(test).toDouble() / values.size >= MOSTLY
}

/** Picks which column plays which part, once per table. */
fun tableListColumns(columns: List<ChatView.Column>, rows: List<Map<String, Scalar?>>): TableListColumns {
    fun label(c: ChatView.Column) = "${c.key} ${c.label}"
    fun numeric(c: ChatView.Column) = rows.mostly(c.key) { Numberish.matches(it) }
    fun dated(c: ChatView.Column) = rows.mostly(c.key) { Dateish.containsMatchIn(it) }
    val trailing = columns.firstOrNull { Amount.containsMatchIn(label(it)) || it.format == ValueFormat.Money }
        ?: columns.reversed().firstOrNull { numeric(it) && !dated(it) }
    val category = columns.firstOrNull { it != trailing && (Category.matches(it.key) || Category.matches(it.label)) }
    val rest = columns.filter { it != trailing && it != category }
    val headline = rest.firstOrNull { !numeric(it) && !dated(it) } ?: rest.firstOrNull() ?: trailing ?: columns.firstOrNull()
    return TableListColumns(headline, trailing.takeIf { it != headline }, category, rest.filter { it != headline })
}
