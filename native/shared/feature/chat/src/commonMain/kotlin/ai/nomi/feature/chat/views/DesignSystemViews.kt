package ai.nomi.feature.chat.views

import ai.nomi.designsystem.ds.Crew
import ai.nomi.designsystem.ds.Ds
import ai.nomi.designsystem.ds.chat.Alternative
import ai.nomi.designsystem.ds.chat.Boundary
import ai.nomi.designsystem.ds.chat.Cart
import ai.nomi.designsystem.ds.chat.CartItem
import ai.nomi.designsystem.ds.chat.CodeBlock
import ai.nomi.designsystem.ds.chat.Diff
import ai.nomi.designsystem.ds.chat.DiffHunk
import ai.nomi.designsystem.ds.chat.DiffLine
import ai.nomi.designsystem.ds.chat.DiffLineKind
import ai.nomi.designsystem.ds.chat.QuickReplies
import ai.nomi.designsystem.ds.chat.SelectionCard
import ai.nomi.designsystem.ds.chat.SelectionItem
import ai.nomi.designsystem.ds.chat.Source
import ai.nomi.designsystem.ds.chat.SourceKind
import ai.nomi.designsystem.ds.chat.Sources
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalUriHandler
import androidx.compose.ui.unit.dp
import kotlinx.datetime.TimeZone

// The views the web draws with the design system's chat cards (`components/views/*View.svelte`), drawn here with the
// same cards from `ai.nomi.designsystem.ds.chat`.

private val AnsweredGap = 6.dp
private const val ANSWERED = "Answered"

private fun money(amount: Double, timeZone: TimeZone) = formatValue(Scalar.Num(amount), ValueFormat.Money, timeZone)

/** Under a card the person has answered: what they sent, or just that they answered. */
@Composable
private fun AnsweredNote(text: String) {
    Text(text, style = Ds.caption, color = Ds.text2, modifier = Modifier.padding(top = AnsweredGap))
}

/** Code in a reply (`CodeView.svelte`): its language and file, Copy, numbered lines. */
@Composable
internal fun CodeViewBlock(view: ChatView.Code) {
    CodeBlock(view.code, language = view.language, file = view.file)
}

/** A change to one file (`DiffView.svelte`): its path, +/− counts and hunks. */
@Composable
internal fun DiffViewBlock(view: ChatView.Diff) {
    Diff(
        view.path,
        view.hunks.map { hunk ->
            DiffHunk(
                hunk.header,
                hunk.lines.map {
                    val kind = when (it.kind) {
                        ChatView.DiffKind.Add -> DiffLineKind.Add
                        ChatView.DiffKind.Del -> DiffLineKind.Del
                        ChatView.DiffKind.Context -> DiffLineKind.Context
                    }
                    DiffLine(kind, it.text, it.old, it.new)
                },
            )
        },
    )
}

/** Where a reply's facts came from (`SourcesView.svelte`); web sources open in the browser. */
@Composable
internal fun SourcesViewBlock(view: ChatView.Sources) {
    val uri = LocalUriHandler.current
    Sources(
        view.items.map { item ->
            Source(
                item.title,
                if (item.web) SourceKind.Web else SourceKind.File,
                item.detail,
                onOpen = item.url?.let { url -> { runCatching { uri.openUri(url) } } },
            )
        },
    )
}

/**
 * Several of many (`SelectionView.svelte`): tick what you want, with a running count and total of what's picked;
 * confirming sends the picked names as the answer.
 */
@Composable
internal fun SelectionViewBlock(view: ChatView.Selection, agent: String?, answered: Boolean, onSend: ((String) -> Unit)?, timeZone: TimeZone) {
    val picked = remember(view) { mutableStateListOf<Boolean>().apply { addAll(view.items.map { it.selected }) } }
    var sent by rememberSaveable(view) { mutableStateOf<String?>(null) }
    val done = answered || sent != null || onSend == null
    val priced = view.items.any { it.amount != null }
    val sum = view.items.indices.sumOf { if (picked[it]) view.items[it].amount ?: 0.0 else 0.0 }
    val total = if (!priced) null else money(sum, timeZone).let { if (view.per != null) "$it / ${view.per}" else it }
    Column {
        SelectionCard(
            agent = Crew.forAgentType(agent),
            question = view.question,
            items = view.items.map { SelectionItem(it.name, it.detail, it.amount?.let { a -> money(a, timeZone) }) },
            confirmLabel = view.confirmLabel ?: "Confirm",
            onConfirm = { indices ->
                if (!done && indices.isNotEmpty()) {
                    val text = "I picked: ${indices.joinToString(", ") { view.items[it].name }}"
                    onSend?.invoke(text)
                    sent = text
                }
            },
            initial = view.items.map { it.selected },
            total = total,
            onChange = { next -> next.forEachIndexed { index, on -> if (index < picked.size) picked[index] = on } },
        )
        if (done && (answered || sent != null)) AnsweredNote(sent ?: ANSWERED)
    }
}

/**
 * A shopping list ready to check out (`CartView.svelte`): quantities change the totals here; swaps and saving the
 * list are asked of the crew; checkout opens the store.
 */
@Composable
internal fun CartViewBlock(view: ChatView.Cart, answered: Boolean, onSend: ((String) -> Unit)?, timeZone: TimeZone) {
    val uri = LocalUriHandler.current
    val quantities = remember(view) { mutableStateListOf<Int>().apply { addAll(view.items.map { it.quantity }) } }
    val ask: ((String) -> Unit)? = if (!answered) onSend else null
    val subtotal = view.items.indices.sumOf { i -> val item = view.items[i]; if (!item.available) 0.0 else (item.price ?: 0.0) * quantities[i] }
    Cart(
        title = view.title,
        store = view.store,
        items = view.items.mapIndexed { i, item ->
            CartItem(
                name = item.name,
                quantity = quantities[i],
                price = if (!item.available || item.price == null) "—" else money(item.price * quantities[i], timeZone),
                detail = item.detail,
                available = item.available,
                swap = item.swap,
            )
        },
        subtotal = money(subtotal, timeZone),
        total = money(subtotal + (view.delivery ?: 0.0), timeZone),
        onCheckout = { runCatching { uri.openUri(view.checkoutUrl) } },
        delivery = view.delivery?.let { money(it, timeZone) },
        note = view.note,
        onQuantity = { index, quantity -> if (quantity >= 0) quantities[index] = quantity },
        onSwap = ask?.let { send -> { index: Int -> view.items[index].let { send("Swap ${it.name}: ${it.swap}") } } },
        onSave = ask?.let { send ->
            {
                val items = view.items.indices.filter { view.items[it].available && quantities[it] > 0 }.joinToString(", ") { "${quantities[it]}× ${view.items[it].name}" }
                send("Save ${view.title} as a shopping list: $items")
            }
        },
    )
}

/** Something the crew won't do, why, and what can help instead (`BoundaryView.svelte`): an alternative asks for it or opens it. */
@Composable
internal fun BoundaryViewBlock(view: ChatView.Boundary, answered: Boolean, onSend: ((String) -> Unit)?) {
    val uri = LocalUriHandler.current
    Boundary(
        view.title,
        view.reason,
        alternatives = view.alternatives.map { alt ->
            Alternative(
                agent = Crew.forAgentType(alt.agent),
                title = alt.title,
                detail = alt.detail,
                external = alt.url != null,
                onClick = {
                    when {
                        alt.url != null -> runCatching { uri.openUri(alt.url) }
                        alt.message != null && onSend != null && !answered -> onSend(alt.message)
                    }
                },
            )
        },
    )
}

/** Short answers to send with one press (`QuickRepliesView.svelte`); gone once the person has replied. */
@Composable
internal fun QuickRepliesViewBlock(view: ChatView.QuickReplies, answered: Boolean, onSend: ((String) -> Unit)?) {
    var sent by rememberSaveable(view) { mutableStateOf<String?>(null) }
    if (answered || sent != null || onSend == null) {
        if (sent != null) AnsweredNote(sent!!)
        return
    }
    QuickReplies(view.replies, { reply -> sent = reply; onSend(reply) })
}
