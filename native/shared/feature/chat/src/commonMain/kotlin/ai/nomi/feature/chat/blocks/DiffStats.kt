package ai.nomi.feature.chat.blocks

/** Past this many line pairs the changed middle is not diffed line by line; its sizes are counted instead. */
private const val MAX_DIFF_CELLS = 4_000_000L

data class DiffStats(val added: Int, val removed: Int)

private fun lines(text: String?): List<String> =
    if (text.isNullOrEmpty()) emptyList() else text.removeSuffix("\n").split('\n')

/** Lines added and removed going from [before] to [after] (`diffStats` in `frontend/src/lib/code/diffStats.ts`). */
fun diffStats(before: String?, after: String?): DiffStats {
    val a = lines(before)
    val b = lines(after)
    // Common lines at both ends don't count; what's left is diffed by longest common subsequence.
    var start = 0
    while (start < a.size && start < b.size && a[start] == b[start]) start++
    var endA = a.size
    var endB = b.size
    while (endA > start && endB > start && a[endA - 1] == b[endB - 1]) {
        endA--
        endB--
    }
    val midA = a.subList(start, endA)
    val midB = b.subList(start, endB)
    if (midA.size.toLong() * midB.size > MAX_DIFF_CELLS) return DiffStats(added = midB.size, removed = midA.size)
    var previous = IntArray(midB.size + 1)
    for (i in 1..midA.size) {
        val row = IntArray(midB.size + 1)
        for (j in 1..midB.size) {
            row[j] = if (midA[i - 1] == midB[j - 1]) previous[j - 1] + 1 else maxOf(previous[j], row[j - 1])
        }
        previous = row
    }
    val common = previous[midB.size]
    return DiffStats(added = midB.size - common, removed = midA.size - common)
}
