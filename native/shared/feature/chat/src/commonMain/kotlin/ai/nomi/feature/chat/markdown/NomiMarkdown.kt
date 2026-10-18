package ai.nomi.feature.chat.markdown

import ai.nomi.designsystem.brand.brandStyle
import ai.nomi.designsystem.generated.NomiCorners
import ai.nomi.designsystem.theme.NomiTheme
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.TextLinkStyles
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.unit.TextUnit
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.em
import androidx.compose.ui.unit.sp
import com.mikepenz.markdown.m3.Markdown
import com.mikepenz.markdown.m3.markdownColor
import com.mikepenz.markdown.m3.markdownTypography
import com.mikepenz.markdown.model.markdownDimens
import com.mikepenz.markdown.model.markdownPadding
import com.mikepenz.markdown.model.rememberMarkdownState

/**
 * Texts up to this long are parsed during composition, so a message has its final height on its first frame (no
 * jump while the thread scrolls); longer ones parse off the main thread.
 */
private const val IMMEDIATE_PARSE_MAX_CHARS = 6_000

private const val H1_SP = 24f
private const val H2_SP = 20f
private const val H3_SP = 18f
private const val HEADING_LINE_HEIGHT = 1.25f
private const val HEADING_WEIGHT = 700
private const val MINOR_HEADING_WEIGHT = 650
private const val CODE_SCALE_EM = 0.85f
private const val CODE_LINE_HEIGHT = 1.5f

/** `:not(pre) > code`: the text colour at 12% behind inline code. */
private const val INLINE_CODE_ALPHA = 0.12f

private val CodeBlockPadding = 12.dp
private val BlockGap = 6.dp
private val ListGap = 4.dp
private val ListIndent = 10.dp
private val QuotePaddingH = 14.dp
private val QuotePaddingV = 4.dp
private val NoGap = 0.dp

/**
 * A message's markdown (`MessageBubble.svelte`'s rendered `content_html`): headings in the brand face, code in
 * JetBrains Mono, links open in the system browser (through `LocalUriHandler`). [style] is the paragraph text.
 */
@Composable
fun NomiMarkdown(
    content: String,
    style: TextStyle,
    color: Color,
    modifier: Modifier = Modifier,
) {
    val colors = MaterialTheme.colorScheme
    val mono = NomiTheme.metaStyle.fontFamily
    val code = style.copy(fontFamily = mono, fontSize = scaled(style.fontSize), lineHeight = scaled(style.fontSize) * CODE_LINE_HEIGHT)
    val brand = brandStyle(H1_SP.sp).copy(fontWeight = FontWeight(HEADING_WEIGHT), color = color)
    fun heading(size: Float) = brand.copy(fontSize = size.sp, lineHeight = (size * HEADING_LINE_HEIGHT).sp)
    val minor = MaterialTheme.typography.titleMedium.copy(fontWeight = FontWeight(MINOR_HEADING_WEIGHT), color = color)

    val state = rememberMarkdownState(content, retainState = true, immediate = content.length <= IMMEDIATE_PARSE_MAX_CHARS)
    Markdown(
        markdownState = state,
        colors = markdownColor(
            text = color,
            codeBackground = colors.surfaceContainerHighest,
            inlineCodeBackground = color.copy(alpha = INLINE_CODE_ALPHA),
            dividerColor = colors.outlineVariant,
            tableBackground = colors.surfaceContainerLowest,
        ),
        typography = markdownTypography(
            h1 = heading(H1_SP),
            h2 = heading(H2_SP),
            h3 = heading(H3_SP),
            h4 = minor,
            h5 = minor,
            h6 = minor,
            text = style,
            code = code,
            inlineCode = style.copy(fontFamily = mono, fontSize = CODE_SCALE_EM.em),
            quote = style.plus(SpanStyle(fontStyle = FontStyle.Italic)),
            paragraph = style,
            ordered = style,
            bullet = style,
            list = style,
            textLink = TextLinkStyles(style = SpanStyle(color = color, textDecoration = TextDecoration.Underline)),
            table = style,
        ),
        padding = markdownPadding(
            block = BlockGap,
            list = ListGap,
            listItemTop = NoGap,
            listItemBottom = NoGap,
            listIndent = ListIndent,
            codeBlock = PaddingValues(CodeBlockPadding),
            blockQuote = PaddingValues(horizontal = QuotePaddingH, vertical = NoGap),
            blockQuoteText = PaddingValues(vertical = QuotePaddingV),
        ),
        dimens = markdownDimens(codeBackgroundCornerSize = NomiCorners.small.dp, tableCornerSize = NomiCorners.largeIncreased.dp),
        modifier = modifier.fillMaxWidth(),
    )
}

private fun scaled(size: TextUnit): TextUnit = if (size == TextUnit.Unspecified) size else size * CODE_SCALE_EM
