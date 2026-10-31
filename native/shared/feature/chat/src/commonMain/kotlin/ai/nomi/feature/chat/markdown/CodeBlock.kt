package ai.nomi.feature.chat.markdown

import ai.nomi.designsystem.generated.NomiCorners
import ai.nomi.designsystem.icon.NomiIcon
import ai.nomi.designsystem.icon.NomiIcons
import ai.nomi.designsystem.theme.NomiTheme
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.em
import kotlinx.coroutines.delay

private const val COPIED_RESET_MS = 1500L
private const val PLAIN = "text"
private const val LANG_TRACKING_EM = 0.04f
private val HeaderPaddingH = 10.dp
private val HeaderPaddingV = 4.dp
private val HeaderButton = 28.dp
private val HeaderGlyph = 14.dp
private val BodyPadding = 12.dp
/** `.code-block__body`: long code scrolls inside the block. */
private val BodyMaxHeight = 480.dp
private val Hairline = 1.dp

/**
 * `.code-block` (`MessageBubble.svelte`'s `enhanceCodeBlocks`): the code under a header with its language, Copy
 * (the code alone) and Collapse. Coloured with the theme roles the web's `nm-code-*` tokens use.
 */
@Composable
internal fun CodeBlock(code: String, language: String?, style: TextStyle, modifier: Modifier = Modifier) {
    val colors = MaterialTheme.colorScheme
    val clipboard = LocalClipboardManager.current
    var copied by remember { mutableStateOf(false) }
    var collapsed by rememberSaveable(code) { mutableStateOf(false) }
    LaunchedEffect(copied) {
        if (copied) {
            delay(COPIED_RESET_MS)
            copied = false
        }
    }
    val shape = RoundedCornerShape(NomiCorners.largeIncreased.dp)
    val text = code.trimEnd('\n')
    val palette = CodePalette(keyword = colors.tertiary, string = colors.primary, function = colors.secondary, number = colors.error, comment = colors.onSurfaceVariant)
    val highlighted = remember(text, language, palette) { highlight(text, language, palette) }
    Column(modifier.fillMaxWidth().padding(vertical = HeaderPaddingV).clip(shape).border(Hairline, colors.outlineVariant, shape)) {
        Row(
            Modifier.fillMaxWidth().background(colors.surfaceContainerHighest).padding(horizontal = HeaderPaddingH, vertical = HeaderPaddingV),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text(
                (language?.takeIf { it.isNotBlank() } ?: PLAIN).uppercase(),
                style = NomiTheme.metaStyle.copy(letterSpacing = LANG_TRACKING_EM.em),
                color = colors.onSurfaceVariant,
                modifier = Modifier.weight(1f),
            )
            Row(horizontalArrangement = Arrangement.spacedBy(HeaderPaddingV)) {
                IconButton(onClick = { clipboard.setText(AnnotatedString(text)); copied = true }, modifier = Modifier.size(HeaderButton)) {
                    NomiIcon(
                        if (copied) NomiIcons.Check else NomiIcons.Copy,
                        if (copied) "Copied" else "Copy code",
                        size = HeaderGlyph,
                        tint = if (copied) colors.primary else colors.onSurfaceVariant,
                    )
                }
                IconButton(onClick = { collapsed = !collapsed }, modifier = Modifier.size(HeaderButton)) {
                    NomiIcon(
                        if (collapsed) NomiIcons.ChevronDown else NomiIcons.ChevronUp,
                        if (collapsed) "Expand code" else "Collapse code",
                        size = HeaderGlyph,
                        tint = colors.onSurfaceVariant,
                    )
                }
            }
        }
        if (!collapsed) {
            Text(
                highlighted,
                style = style.copy(color = colors.onSurface),
                softWrap = false,
                modifier = Modifier
                    .fillMaxWidth()
                    .heightIn(max = BodyMaxHeight)
                    .background(colors.surfaceContainerLow)
                    .verticalScroll(rememberScrollState())
                    .horizontalScroll(rememberScrollState())
                    .padding(BodyPadding),
            )
        }
    }
}

internal data class CodePalette(val keyword: Color, val string: Color, val function: Color, val number: Color, val comment: Color)

private val Keywords = setOf(
    "fun", "val", "var", "class", "object", "interface", "data", "enum", "when", "if", "else", "for", "while", "do", "return",
    "import", "package", "private", "public", "internal", "protected", "override", "suspend", "const", "let", "function",
    "async", "await", "export", "default", "from", "new", "try", "catch", "finally", "throw", "def", "self", "fn", "pub",
    "impl", "struct", "use", "mod", "match", "mut", "type", "in", "is", "as", "null", "true", "false", "None", "True",
    "False", "nil", "this", "static", "void", "int", "string", "bool", "with", "yield", "lambda", "elif", "pass", "break",
    "continue", "switch", "case", "extends", "implements", "where", "select", "insert", "update", "delete", "create",
    "table", "and", "or", "not", "SELECT", "FROM", "WHERE", "INSERT", "UPDATE", "DELETE", "CREATE", "TABLE", "AND", "OR",
)

/** The languages whose comments start with `#` rather than `//`. */
private val HashComments = setOf("python", "py", "bash", "sh", "shell", "zsh", "yaml", "yml", "toml", "ruby", "rb", "dockerfile")

/** A light, language-agnostic colouring: comments, strings, numbers, keywords, and names before `(`. */
internal fun highlight(code: String, language: String?, palette: CodePalette): AnnotatedString = buildAnnotatedString {
    val hash = language?.lowercase() in HashComments
    var i = 0
    while (i < code.length) {
        val c = code[i]
        val rest = code.length - i
        when {
            (!hash && c == '/' && rest > 1 && code[i + 1] == '/') || (hash && c == '#') || (c == '-' && rest > 1 && code[i + 1] == '-' && language?.lowercase() == "sql") -> {
                val end = code.indexOf('\n', i).let { if (it < 0) code.length else it }
                withStyle(SpanStyle(color = palette.comment, fontStyle = FontStyle.Italic)) { append(code, i, end) }
                i = end
            }
            !hash && c == '/' && rest > 1 && code[i + 1] == '*' -> {
                val end = code.indexOf("*/", i + 2).let { if (it < 0) code.length else it + 2 }
                withStyle(SpanStyle(color = palette.comment, fontStyle = FontStyle.Italic)) { append(code, i, end) }
                i = end
            }
            c == '"' || c == '\'' || c == '`' -> {
                var end = i + 1
                while (end < code.length && code[end] != c && code[end] != '\n') end += if (code[end] == '\\') 2 else 1
                end = minOf(end + 1, code.length)
                withStyle(SpanStyle(color = palette.string)) { append(code, i, end) }
                i = end
            }
            c.isDigit() -> {
                var end = i
                while (end < code.length && (code[end].isLetterOrDigit() || code[end] == '.' || code[end] == '_')) end++
                withStyle(SpanStyle(color = palette.number)) { append(code, i, end) }
                i = end
            }
            c.isLetter() || c == '_' -> {
                var end = i
                while (end < code.length && (code[end].isLetterOrDigit() || code[end] == '_')) end++
                val word = code.substring(i, end)
                when {
                    word in Keywords -> withStyle(SpanStyle(color = palette.keyword)) { append(word) }
                    end < code.length && code[end] == '(' -> withStyle(SpanStyle(color = palette.function)) { append(word) }
                    else -> append(word)
                }
                i = end
            }
            else -> {
                append(c)
                i++
            }
        }
    }
}
