package ai.nomi.designsystem.ds.chat

// A light tokenizer for showing code in the design system's code views (CodeView, CodeBlock): comments, strings,
// numbers, keywords, types and punctuation, line by line. Good enough to read by, not a parser.

private val Keywords = setOf(
    "import", "export", "from", "default", "const", "let", "var", "function", "return", "if", "else", "for", "while",
    "do", "switch", "case", "break", "continue", "new", "class", "extends", "implements", "interface", "type", "enum",
    "async", "await", "try", "catch", "finally", "throw", "typeof", "instanceof", "in", "of", "this", "super", "null",
    "undefined", "true", "false", "void", "static", "public", "private", "protected", "readonly", "fun", "val", "fn",
    "pub", "impl", "struct", "use", "mod", "match", "self", "def", "lambda", "None", "True", "False", "and", "or", "not",
    "select", "where", "insert", "update", "delete", "create", "table", "as", "with",
)
private val TokenPattern = Regex(
    "(//.*|#(?![0-9a-fA-F]{3,8}\\b).*|/\\*.*?\\*/|<!--.*?-->)" + // comment
        "|(\"(?:[^\"\\\\]|\\\\.)*\"?|'(?:[^'\\\\]|\\\\.)*'?|`(?:[^`\\\\]|\\\\.)*`?)" + // string
        "|(\\b\\d[\\d_.xXa-fA-F]*\\b)" + // number
        "|([A-Za-z_$][\\w$]*)" + // word
        "|([{}()\\[\\];,.:=<>+\\-*/!&|?%^~@])", // punctuation
)
private const val GROUP_COMMENT = 1
private const val GROUP_STRING = 2
private const val GROUP_NUMBER = 3
private const val GROUP_WORD = 4

/** [code] as lines of coloured tokens. */
fun codeTokens(code: String): List<List<CodeToken>> = code.trimEnd('\n').split('\n').map(::lineTokens)

private fun lineTokens(line: String): List<CodeToken> {
    val out = mutableListOf<CodeToken>()
    var at = 0
    for (match in TokenPattern.findAll(line)) {
        if (match.range.first > at) out += CodeToken(line.substring(at, match.range.first))
        val text = match.value
        val kind = when {
            match.groups[GROUP_COMMENT] != null -> CodeTokenKind.Comment
            match.groups[GROUP_STRING] != null -> CodeTokenKind.String
            match.groups[GROUP_NUMBER] != null -> CodeTokenKind.Number
            match.groups[GROUP_WORD] != null -> when {
                text in Keywords || text.lowercase() in Keywords && text.all { it.isUpperCase() } -> CodeTokenKind.Keyword
                text.first().isUpperCase() -> CodeTokenKind.Type
                match.range.last + 1 < line.length && line[match.range.last + 1] == '(' -> CodeTokenKind.Fn
                else -> CodeTokenKind.Plain
            }
            else -> CodeTokenKind.Punct
        }
        out += CodeToken(text, kind)
        at = match.range.last + 1
    }
    if (at < line.length) out += CodeToken(line.substring(at))
    return out
}
