package ai.nomi.designsystem.ds.chat

import ai.nomi.designsystem.ds.Crew
import ai.nomi.designsystem.ds.CopyGlyph
import ai.nomi.designsystem.ds.Ds
import ai.nomi.designsystem.ds.footerNote
import ai.nomi.designsystem.ds.DsBareIcon
import ai.nomi.designsystem.ds.DsButton
import ai.nomi.designsystem.ds.DsButtonVariant
import ai.nomi.designsystem.ds.DsCard
import ai.nomi.designsystem.ds.DsCardTitle
import ai.nomi.designsystem.ds.DsIcon
import ai.nomi.designsystem.ds.DsIconButton
import ai.nomi.designsystem.ds.DsIconButtonVariant
import ai.nomi.designsystem.ds.DsLink
import ai.nomi.designsystem.ds.DsMotion
import ai.nomi.designsystem.ds.DsShapes
import ai.nomi.designsystem.ds.DsText
import ai.nomi.designsystem.ds.DsTextAction
import ai.nomi.designsystem.ds.Hairline
import ai.nomi.designsystem.ds.Mascot
import ai.nomi.designsystem.ds.MinTouch
import ai.nomi.designsystem.ds.SectionLabel
import ai.nomi.designsystem.ds.dsDisabled
import ai.nomi.designsystem.ds.dsOutline
import ai.nomi.designsystem.ds.rememberCopier
import ai.nomi.designsystem.icon.Lucide
import ai.nomi.designsystem.icon.NomiGlyph
import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.animateColorAsState
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.dp

private const val CODE_LINE_HEIGHT = 1.7f
private const val OUT_OF_STOCK_ALPHA = 0.4f

/** A source's number inside a sentence: a small mono chip ("1"). */
@Composable
fun Citation(n: Int, title: String, modifier: Modifier = Modifier, onClick: (() -> Unit)? = null) {
    Box(
        modifier
            .height(16.dp)
            .clip(DsShapes.xs)
            .background(Ds.surface2)
            .then(if (onClick != null) Modifier.clickable(role = Role.Button, onClick = onClick) else Modifier)
            .semantics { contentDescription = "Source $n: $title" }
            .padding(horizontal = 5.dp),
        contentAlignment = Alignment.Center,
    ) { Text(n.toString(), style = Ds.micro, color = Ds.text2) }
}

enum class SourceKind { File, Web }
data class Source(val title: String, val kind: SourceKind, val detail: String? = null, val onOpen: (() -> Unit)? = null)

/** Where a reply's facts came from, numbered to match its citations. */
@Composable
fun Sources(sources: List<Source>, modifier: Modifier = Modifier) {
    Column(modifier.fillMaxWidth(), verticalArrangement = Arrangement.spacedBy(8.dp)) {
        SectionLabel("Sources")
        sources.forEachIndexed { index, source ->
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
                Citation(index + 1, source.title)
                DsIcon(if (source.kind == SourceKind.File) Lucide.FileText else Lucide.Globe, 12.dp, Ds.text3)
                if (source.onOpen != null) {
                    Text(
                        source.title, style = Ds.caption.copy(textDecoration = TextDecoration.Underline), color = Ds.text2, maxLines = 1,
                        modifier = Modifier.weight(1f, fill = false).clickable(role = Role.Button, onClick = source.onOpen),
                    )
                } else {
                    DsText(source.title, Ds.caption, Ds.text2, Modifier.weight(1f, fill = false), singleLine = true)
                }
                if (source.detail != null) DsText(source.detail, Ds.label, Ds.text3, singleLine = true)
            }
        }
    }
}

enum class CodeTokenKind { Plain, Keyword, String, Fn, Number, Type, Comment, Punct }
data class CodeToken(val text: String, val kind: CodeTokenKind = CodeTokenKind.Plain)

@Composable
private fun CodeTokenKind.color(): Color = when (this) {
    CodeTokenKind.Keyword, CodeTokenKind.Type -> Ds.codeKeyword
    CodeTokenKind.String -> Ds.codeString
    CodeTokenKind.Fn -> Ds.codeFn
    CodeTokenKind.Number -> Ds.codeNumber
    CodeTokenKind.Comment -> Ds.text3
    CodeTokenKind.Punct -> Ds.text2
    CodeTokenKind.Plain -> Ds.text
}

/**
 * Code in a reply: its language and file, Copy (turning to "Copied"), and numbered lines that scroll sideways.
 * [tokens] highlights it line by line; without them it shows plain.
 */
@Composable
fun CodeBlock(code: String, modifier: Modifier = Modifier, language: String? = null, file: String? = null, tokens: List<List<CodeToken>>? = null) {
    val copier = rememberCopier()
    val lines = tokens ?: code.lines().map { listOf(CodeToken(it)) }
    val style = Ds.mono.copy(lineHeight = Ds.mono.fontSize * CODE_LINE_HEIGHT)
    DsCard(
        modifier,
        header = {
            if (language != null) Text(language, style = Ds.label, color = Ds.text3)
            DsText(file ?: "", Ds.label, Ds.text2, Modifier.weight(1f), singleLine = true)
            Row(
                Modifier.heightIn(min = MinTouch).clip(DsShapes.xs).clickable(role = Role.Button) { copier.copy(code) },
                horizontalArrangement = Arrangement.spacedBy(6.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                CopyGlyph(copier.copied, 12.dp, Ds.text3)
                AnimatedContent(copier.copied, label = "code-copy") { Text(if (it) "Copied" else "Copy", style = Ds.caption, color = Ds.text2) }
            }
        },
    ) {
        Row(Modifier.fillMaxWidth().horizontalScroll(rememberScrollState()).padding(top = 12.dp, bottom = 12.dp, end = 14.dp)) {
            Column {
                lines.forEachIndexed { index, _ -> Text((index + 1).toString(), style = style, color = Ds.text3, textAlign = TextAlign.End, modifier = Modifier.width(36.dp)) }
            }
            Column(Modifier.padding(start = 14.dp)) {
                lines.forEach { line ->
                    Text(
                        buildAnnotatedString { line.forEach { token -> withStyle(SpanStyle(color = token.kind.color())) { append(token.text) } } },
                        style = style, softWrap = false,
                    )
                }
            }
        }
    }
}

enum class DiffLineKind { Context, Add, Del }
data class DiffLine(val kind: DiffLineKind, val text: String, val old: Int? = null, val new: Int? = null)
data class DiffHunk(val header: String, val lines: List<DiffLine>)
data class DiffRequest(val agent: Crew, val name: String, val files: Int)

/** A change to one file: its path and +/− counts, the hunks with old and new line numbers, and apply or reject. */
@Composable
fun Diff(path: String, hunks: List<DiffHunk>, modifier: Modifier = Modifier, request: DiffRequest? = null, onApply: (() -> Unit)? = null, onReject: (() -> Unit)? = null) {
    val added = hunks.sumOf { h -> h.lines.count { it.kind == DiffLineKind.Add } }
    val removed = hunks.sumOf { h -> h.lines.count { it.kind == DiffLineKind.Del } }
    val style = Ds.mono.copy(lineHeight = Ds.mono.fontSize * CODE_LINE_HEIGHT)
    DsCard(
        modifier,
        header = {
            DsIcon(Lucide.FileDiff, 14.dp, Ds.text2)
            DsText(path, Ds.label, Ds.text2, Modifier.weight(1f), singleLine = true)
            Row(Modifier.semantics(mergeDescendants = true) { contentDescription = "$added lines added, $removed removed" }, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                Text("+$added", style = Ds.label, color = Ds.accent)
                Text("−$removed", style = Ds.label, color = Ds.danger)
            }
        },
        footer = if (request != null || onApply != null || onReject != null) {
            {
                Row(Modifier.footerNote(this), horizontalArrangement = Arrangement.spacedBy(6.dp), verticalAlignment = Alignment.CenterVertically) {
                    if (request != null) {
                        Mascot(request.agent, 20.dp)
                        Text(
                            if (request.files == 1) "${request.name} wants to change 1 file" else "${request.name} wants to change ${request.files} files",
                            style = Ds.caption, color = Ds.text2,
                        )
                    }
                }
                if (onReject != null) DsButton("Reject", onReject, variant = DsButtonVariant.Secondary)
                if (onApply != null) DsButton("Apply change", onApply, iconEnd = Lucide.Check)
            }
        } else {
            null
        },
    ) {
        // Rows as wide as the widest line (and at least the card), so a line's colour runs the full width.
        BoxWithConstraints(Modifier.fillMaxWidth()) {
        val cardWidth = maxWidth
        Column(Modifier.horizontalScroll(rememberScrollState()).width(IntrinsicSize.Max).widthIn(min = cardWidth).padding(bottom = 6.dp)) {
            hunks.forEach { hunk ->
                Text(hunk.header, style = Ds.label, color = Ds.text3, modifier = Modifier.fillMaxWidth().background(Ds.surface).padding(horizontal = 14.dp, vertical = 4.dp))
                hunk.lines.forEach { line ->
                    val bg = when (line.kind) {
                        DiffLineKind.Add -> Ds.diffAdd
                        DiffLineKind.Del -> Ds.diffDel
                        DiffLineKind.Context -> Color.Transparent
                    }
                    Row(Modifier.fillMaxWidth().background(bg)) {
                        Text(line.old?.toString() ?: "", style = style, color = Ds.text3, textAlign = TextAlign.End, modifier = Modifier.width(36.dp))
                        Text(line.new?.toString() ?: "", style = style, color = Ds.text3, textAlign = TextAlign.End, modifier = Modifier.width(36.dp))
                        Text(
                            when (line.kind) { DiffLineKind.Add -> "+"; DiffLineKind.Del -> "−"; DiffLineKind.Context -> "" },
                            style = style, textAlign = TextAlign.Center, modifier = Modifier.width(24.dp),
                            color = when (line.kind) { DiffLineKind.Add -> Ds.accent; DiffLineKind.Del -> Ds.danger; DiffLineKind.Context -> Ds.text3 },
                        )
                        Text(line.text, style = style, softWrap = false, color = if (line.kind == DiffLineKind.Context) Ds.text2 else Ds.text, modifier = Modifier.padding(end = 14.dp))
                    }
                }
            }
        }
        }
    }
}

enum class CellTone { Good, Bad }
data class Cell(val text: String, val tone: CellTone? = null)

/**
 * A table in a reply: its title with copy (as tab-separated text), mono numbers right-aligned, an optional total row,
 * and where the numbers came from.
 */
@Composable
fun DataTable(
    title: String,
    columns: List<String>,
    rows: List<List<Cell>>,
    modifier: Modifier = Modifier,
    total: List<Cell>? = null,
    source: String? = null,
    linkLabel: String? = null,
    onLink: (() -> Unit)? = null,
    onDownload: (() -> Unit)? = null,
) {
    val copier = rememberCopier()
    val tsv = (listOf(columns) + rows.map { r -> r.map { it.text } } + listOfNotNull(total?.map { it.text })).joinToString("\n") { it.joinToString("\t") }
    DsCard(
        modifier,
        header = {
            DsIcon(Lucide.Table, 14.dp, Ds.text2)
            DsCardTitle(title, singleLine = true)
            DsBareIcon(if (copier.copied) Lucide.Check else Lucide.Copy, if (copier.copied) "Copied" else "Copy table", { copier.copy(tsv) })
            if (onDownload != null) DsBareIcon(Lucide.Download, "Download table", onDownload)
        },
        footer = if (source != null || linkLabel != null) {
            {
                DsText(source ?: "", Ds.label, Ds.text3, Modifier.footerNote(this), singleLine = true)
                if (linkLabel != null && onLink != null) DsLink(linkLabel, onLink, quiet = true)
            }
        } else {
            null
        },
    ) {
        Column(Modifier.fillMaxWidth().horizontalScroll(rememberScrollState())) {
            Row(Modifier.padding(top = 9.dp, bottom = 8.dp)) {
                columns.forEachIndexed { index, column -> TableCell(index, column.uppercase(), Ds.labelCaps, Ds.text3) }
            }
            HorizontalDivider(thickness = Hairline, color = Ds.border, modifier = Modifier.widthIn(min = TABLE_MIN_WIDTH))
            rows.forEach { row ->
                Row(Modifier.padding(vertical = 9.dp)) {
                    row.forEachIndexed { index, cell -> TableCell(index, cell.text, if (index == 0) Ds.ui else Ds.mono, cell.color(index, false)) }
                }
            }
            if (total != null) {
                HorizontalDivider(thickness = Hairline, color = Ds.border, modifier = Modifier.widthIn(min = TABLE_MIN_WIDTH))
                Row(Modifier.background(Ds.surface).padding(top = 8.dp, bottom = 9.dp)) {
                    total.forEachIndexed { index, cell ->
                        TableCell(index, cell.text, (if (index == 0) Ds.ui else Ds.mono).copy(fontWeight = FontWeight.Medium), cell.color(index, true))
                    }
                }
            }
        }
    }
}

private val TABLE_MIN_WIDTH = 480.dp

@Composable
private fun Cell.color(index: Int, isTotal: Boolean): Color = when {
    tone == CellTone.Bad -> Ds.danger
    tone == CellTone.Good -> Ds.accent
    index == 0 && !isTotal -> Ds.text2
    else -> Ds.text
}

@Composable
private fun TableCell(index: Int, text: String, style: androidx.compose.ui.text.TextStyle, color: Color) {
    if (index == 0) {
        Text(text, style = style, color = color, modifier = Modifier.width(180.dp).padding(start = 14.dp, end = 6.dp), maxLines = 1)
    } else {
        Text(text, style = style, color = color, textAlign = TextAlign.End, modifier = Modifier.width(108.dp).padding(horizontal = 6.dp), maxLines = 1)
    }
}

/** A link inside a reply's prose: underlined in the border colour, text-2 until pressed. */
@Composable
fun Hyperlink(label: String, onClick: () -> Unit, modifier: Modifier = Modifier) {
    Text(
        label,
        style = Ds.body.copy(textDecoration = TextDecoration.Underline),
        color = Ds.text,
        modifier = modifier.clip(DsShapes.xs).clickable(role = Role.Button, onClick = onClick),
    )
}

/** A link a reply mentions, previewed: its site, title, description, and an image when it has one. */
@Composable
fun LinkPreview(host: String, title: String, onOpen: () -> Unit, modifier: Modifier = Modifier, description: String? = null, image: (@Composable () -> Unit)? = null) {
    Row(modifier.fillMaxWidth().clip(DsShapes.md).background(Ds.bg).dsOutline(Ds.borderStrong, DsShapes.md).clickable(role = Role.Button, onClick = onOpen)) {
        Column(Modifier.weight(1f).padding(14.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
            Row(horizontalArrangement = Arrangement.spacedBy(6.dp), verticalAlignment = Alignment.CenterVertically) {
                DsIcon(Lucide.Globe, 12.dp, Ds.text3)
                DsText(host, Ds.label, Ds.text3, singleLine = true)
            }
            Text(title, style = Ds.ui.copy(fontWeight = FontWeight.Medium), color = Ds.text)
            if (description != null) Text(description, style = Ds.caption, color = Ds.text3)
        }
        if (image != null) Box(Modifier.width(112.dp).height(112.dp)) { image() }
    }
}

/** A crew member mentioned in a message: "@Koda" on their tint. */
@Composable
fun MentionCrew(agent: Crew, name: String, modifier: Modifier = Modifier) {
    Text("@$name", style = Ds.ui.copy(fontWeight = FontWeight.Medium), color = Ds.text, modifier = modifier.clip(DsShapes.xs).background(agent.tint).padding(horizontal = 4.dp))
}

/** A file mentioned in a message, in mono with its icon. */
@Composable
fun MentionFile(name: String, modifier: Modifier = Modifier, icon: NomiGlyph = Lucide.File) {
    Row(
        modifier.clip(DsShapes.xs).background(Ds.surface2).padding(horizontal = 4.dp, vertical = 1.dp),
        horizontalArrangement = Arrangement.spacedBy(4.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        DsIcon(icon, 12.dp, Ds.text2)
        Text(name, style = Ds.mono, color = Ds.text)
    }
}

sealed interface MentionOption {
    data class CrewMember(val agent: Crew, val name: String, val role: String) : MentionOption
    data class FileRef(val name: String, val where: String, val icon: NomiGlyph? = null) : MentionOption
}

/** What "@" offers: the crew, then files; the highlighted one shows ↵ and is what Enter picks. */
@Composable
fun MentionPicker(options: List<MentionOption>, active: Int, onActive: (Int) -> Unit, onSelect: (MentionOption) -> Unit, modifier: Modifier = Modifier) {
    val crew = options.filterIsInstance<MentionOption.CrewMember>()
    val files = options.filterIsInstance<MentionOption.FileRef>()
    Column(
        modifier.widthIn(max = 360.dp).clip(DsShapes.md).background(Ds.surface).dsOutline(Ds.borderStrong, DsShapes.md).padding(6.dp),
        verticalArrangement = Arrangement.spacedBy(2.dp),
    ) {
        @Composable
        fun option(option: MentionOption, index: Int) {
            val bg by animateColorAsState(if (index == active) Ds.surface2 else Color.Transparent, DsMotion.fast(), label = "mention-active")
            Row(
                Modifier.fillMaxWidth().heightIn(min = 34.dp).clip(DsShapes.xs).background(bg)
                    .clickable(role = Role.Button) { onActive(index); onSelect(option) }.padding(horizontal = 8.dp),
                horizontalArrangement = Arrangement.spacedBy(8.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                when (option) {
                    is MentionOption.CrewMember -> {
                        Mascot(option.agent, 20.dp)
                        Text(option.name, style = Ds.ui, color = Ds.text)
                        DsText(option.role, Ds.caption, Ds.text3, Modifier.weight(1f), singleLine = true)
                    }
                    is MentionOption.FileRef -> {
                        DsIcon(option.icon ?: Lucide.File, 14.dp, Ds.text2)
                        DsText(option.name, Ds.mono, Ds.text, Modifier.weight(1f), singleLine = true)
                        Text(option.where, style = Ds.label, color = Ds.text3)
                    }
                }
                if (index == active) Text("↵", style = Ds.label, color = Ds.text3)
            }
        }
        if (crew.isNotEmpty()) {
            Text("CREW", style = Ds.labelCaps, color = Ds.text3, modifier = Modifier.padding(horizontal = 8.dp, vertical = 4.dp))
            crew.forEachIndexed { index, item -> option(item, index) }
        }
        if (files.isNotEmpty()) {
            Text("FILES", style = Ds.labelCaps, color = Ds.text3, modifier = Modifier.padding(horizontal = 8.dp, vertical = 4.dp))
            files.forEachIndexed { index, item -> option(item, crew.size + index) }
        }
    }
}

/**
 * A reminder in a reply: what and when, who keeps it. When it's [due] it turns to the warning colour, rings, and
 * offers snooze and done.
 */
@Composable
fun Reminder(
    title: String,
    whenLabel: String,
    modifier: Modifier = Modifier,
    due: Boolean = false,
    owner: StepOwner? = null,
    doneLabel: String = "Mark done",
    onEdit: (() -> Unit)? = null,
    onSnooze: (() -> Unit)? = null,
    onDone: (() -> Unit)? = null,
) {
    val edge by animateColorAsState(if (due) Ds.warning else Ds.borderStrong, DsMotion.effects(), label = "reminder-edge")
    FlowRow(
        modifier.fillMaxWidth().clip(DsShapes.md).background(Ds.bg).dsOutline(edge, DsShapes.md).padding(horizontal = 14.dp, vertical = 12.dp),
        horizontalArrangement = Arrangement.spacedBy(12.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp),
        itemVerticalAlignment = Alignment.CenterVertically,
    ) {
        Box(
            Modifier.size(32.dp).clip(DsShapes.sm).background(if (due) Ds.warningBg else Ds.surface2),
            contentAlignment = Alignment.Center,
        ) {
            AnimatedContent(due, label = "reminder-bell") { DsIcon(if (it) Lucide.BellRing else Lucide.Bell, 16.dp, if (it) Ds.warning else Ds.text2) }
        }
        Column(Modifier.weight(1f).widthIn(min = 160.dp), verticalArrangement = Arrangement.spacedBy(3.dp)) {
            Text(title, style = Ds.ui.copy(fontWeight = FontWeight.Medium), color = Ds.text)
            Text(if (due) "due now · $whenLabel" else whenLabel, style = Ds.label, color = if (due) Ds.warning else Ds.text3)
        }
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
            if (due) {
                if (onSnooze != null) DsButton("Snooze 1h", onSnooze, variant = DsButtonVariant.Ghost, iconStart = Lucide.AlarmClock)
                if (onDone != null) DsButton(doneLabel, onDone, variant = DsButtonVariant.Secondary, iconEnd = Lucide.Check)
            } else {
                if (owner != null) {
                    Mascot(owner.agent, 20.dp)
                    Text(owner.name, style = Ds.caption, color = Ds.text3)
                }
                if (onEdit != null) DsIconButton(Lucide.Pencil, "Edit reminder", onEdit, variant = DsIconButtonVariant.Ghost, small = true)
            }
        }
    }
}

data class CartItem(
    val name: String,
    val quantity: Int,
    val price: String,
    val detail: String? = null,
    val icon: NomiGlyph? = null,
    val available: Boolean = true,
    val swap: String? = null,
)

/**
 * A shopping list ready to check out: each item with its quantity stepper and price, what's out of stock (with a
 * swap), the subtotal, delivery and total, and the store's checkout.
 */
@Composable
fun Cart(
    title: String,
    store: String,
    items: List<CartItem>,
    subtotal: String,
    total: String,
    onCheckout: () -> Unit,
    modifier: Modifier = Modifier,
    delivery: String? = null,
    note: String? = null,
    onQuantity: ((index: Int, quantity: Int) -> Unit)? = null,
    onSwap: ((Int) -> Unit)? = null,
    onSave: (() -> Unit)? = null,
) {
    DsCard(
        modifier,
        header = {
            DsIcon(Lucide.Basket, 14.dp, Ds.text2)
            DsCardTitle(title, singleLine = true)
            Text("$store · ${if (items.size == 1) "1 item" else "${items.size} items"}", style = Ds.label, color = Ds.text3)
        },
        footer = {
            DsText(note ?: "", Ds.caption, Ds.text3, Modifier.footerNote(this))
            if (onSave != null) DsButton("Save as list", onSave, variant = DsButtonVariant.Ghost, iconStart = Lucide.Bookmark)
            DsButton("Checkout on $store", onCheckout, iconEnd = Lucide.ArrowUpRight)
        },
    ) {
        items.forEachIndexed { index, item ->
            if (index > 0) HorizontalDivider(thickness = Hairline, color = Ds.border)
            Row(
                Modifier.fillMaxWidth().padding(start = 14.dp, end = 14.dp, top = if (index > 0) 9.dp else 10.dp, bottom = 10.dp),
                horizontalArrangement = Arrangement.spacedBy(12.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Box(Modifier.size(36.dp).clip(DsShapes.sm).background(Ds.surface2), contentAlignment = Alignment.Center) {
                    DsIcon(item.icon ?: Lucide.Package, 16.dp, if (item.available) Ds.text2 else Ds.text3)
                }
                Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
                    DsText(item.name, Ds.ui, if (item.available) Ds.text else Ds.text3, singleLine = true)
                    when {
                        !item.available -> {
                            Text("out of stock", style = Ds.label, color = Ds.warning)
                            if (item.swap != null && onSwap != null) DsTextAction(item.swap, { onSwap(index) }, underline = true)
                            else if (item.swap != null) Text(item.swap, style = Ds.caption, color = Ds.text2)
                        }
                        item.detail != null -> DsText(item.detail, Ds.label, Ds.text3, singleLine = true)
                    }
                }
                Row(
                    Modifier.height(26.dp).clip(DsShapes.sm).dsOutline(Ds.borderStrong).alpha(if (item.available) 1f else OUT_OF_STOCK_ALPHA)
                        .semantics(mergeDescendants = false) { contentDescription = "Quantity of ${item.name}" },
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    StepperButton(Lucide.Minus, "Fewer ${item.name}", item.available && onQuantity != null && item.quantity > 0) { onQuantity?.invoke(index, item.quantity - 1) }
                    AnimatedContent(item.quantity, label = "cart-qty") {
                        Text(it.toString(), style = Ds.label, color = Ds.text, textAlign = TextAlign.Center, modifier = Modifier.width(20.dp))
                    }
                    StepperButton(Lucide.Plus, "More ${item.name}", item.available && onQuantity != null) { onQuantity?.invoke(index, item.quantity + 1) }
                }
                Text(item.price, style = Ds.mono, color = if (item.available) Ds.text else Ds.text3, textAlign = TextAlign.End, maxLines = 1, softWrap = false, modifier = Modifier.widthIn(min = 72.dp))
            }
        }
        HorizontalDivider(thickness = Hairline, color = Ds.border)
        Column(Modifier.fillMaxWidth().background(Ds.surface).padding(start = 14.dp, end = 14.dp, top = 9.dp, bottom = 10.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
            CartLine("Subtotal", subtotal)
            if (delivery != null) CartLine("Delivery", delivery)
            CartLine("Total", total, strong = true)
        }
    }
}

@Composable
private fun StepperButton(icon: NomiGlyph, label: String, enabled: Boolean, onClick: () -> Unit) {
    Box(
        Modifier.size(24.dp).clip(DsShapes.xs).clickable(enabled = enabled, role = Role.Button, onClick = onClick).semantics { contentDescription = label }.dsDisabled(!enabled),
        contentAlignment = Alignment.Center,
    ) { DsIcon(icon, 12.dp, Ds.text2) }
}

@Composable
private fun CartLine(label: String, value: String, strong: Boolean = false) {
    Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
        Text(label, style = Ds.ui.copy(fontWeight = if (strong) FontWeight.Medium else FontWeight.Normal), color = if (strong) Ds.text else Ds.text2)
        Text(value, style = Ds.mono.copy(fontWeight = if (strong) FontWeight.Medium else FontWeight.Normal, fontSize = Ds.mono.fontSize), color = Ds.text)
    }
}
