package ai.nomi.feature.chat

import androidx.compose.runtime.staticCompositionLocalOf

// A sent message carries one reference tag per attached file after the typed text:
// <attachment id="…" name="…" kind="…" mime="…" size="…"/>. Messages from before uploads carried a text file
// inline instead: <attachment name="…" kind="file|voice">…</attachment>. The chat shows the tags as previews and
// cards, never as text (`$lib/attachments`).

/** What a file is, as the backend files it; [InlineText] is an older message's inline text file. */
enum class AttachmentKind { Text, Pdf, Document, Spreadsheet, Presentation, Image, Audio, Voice, Video, Other, InlineText }

/** A file in a sent message: uploaded ones have an [id] (and their type and size), older inline ones their [text]. */
data class Attachment(
    val name: String,
    val kind: AttachmentKind,
    val id: String? = null,
    val mime: String? = null,
    val size: Long? = null,
    val text: String? = null,
)

/** A message's typed text and its attached files, in the order they appear. */
data class SplitMessage(val text: String, val files: List<Attachment>)

private val REFERENCE = Regex("""<attachment ([^<>]*?)/>""")
private val INLINE = Regex("""<attachment name="([^"]*)"(?: kind="(file|voice)")?>\n([\s\S]*?)\n</attachment>""")
private const val DEFAULT_NAME = "file"
private const val INLINE_VOICE = "voice"

private fun attribute(attributes: String, name: String): String? =
    Regex("""(?:^|\s)$name="([^"]*)"""").find(attributes)?.groupValues?.get(1)

private fun kindOf(raw: String?): AttachmentKind = when (raw) {
    "text" -> AttachmentKind.Text
    "pdf" -> AttachmentKind.Pdf
    "document" -> AttachmentKind.Document
    "spreadsheet" -> AttachmentKind.Spreadsheet
    "presentation" -> AttachmentKind.Presentation
    "image" -> AttachmentKind.Image
    "audio" -> AttachmentKind.Audio
    "voice" -> AttachmentKind.Voice
    "video" -> AttachmentKind.Video
    else -> AttachmentKind.Other
}

/** The typed text and attached files of [content]; a message without tags comes back as its text alone. */
fun splitAttachments(content: String): SplitMessage {
    if ("<attachment " !in content) return SplitMessage(content, emptyList())
    val found = mutableListOf<Pair<Int, Attachment>>()
    var text = REFERENCE.replace(content) { match ->
        val attributes = match.groupValues[1]
        val id = attribute(attributes, "id") ?: return@replace match.value
        found += match.range.first to Attachment(
            name = attribute(attributes, "name") ?: DEFAULT_NAME,
            kind = kindOf(attribute(attributes, "kind")),
            id = id,
            mime = attribute(attributes, "mime"),
            size = attribute(attributes, "size")?.toLongOrNull(),
        )
        ""
    }
    text = INLINE.replace(text) { match ->
        val (name, kind, body) = match.destructured
        val inlineKind = if (kind == INLINE_VOICE) AttachmentKind.Voice else AttachmentKind.InlineText
        found += match.range.first to Attachment(name = name, kind = inlineKind, text = body)
        ""
    }
    return SplitMessage(text.trim(), found.sortedBy { it.first }.map { it.second })
}

private const val KB = 1024L
private const val MB = KB * KB
private const val ONE_DECIMAL_BELOW = 10L

/** "512 B", "4.2 KB", "13 MB", as the web shows a file's size. */
fun formatBytes(bytes: Long): String = when {
    bytes < KB -> "$bytes B"
    bytes < MB -> scaled(bytes, KB, "KB")
    else -> scaled(bytes, MB, "MB")
}

private fun scaled(bytes: Long, unit: Long, label: String): String {
    if (bytes >= ONE_DECIMAL_BELOW * unit) return "${(bytes + unit / 2) / unit} $label"
    val tenths = (bytes * 10 + unit / 2) / unit
    return "${tenths / 10}.${tenths % 10} $label"
}

/** Loads an attached image's preview (null when it can't be had); the app provides one backed by the engine. */
val LocalAttachmentPreviews = staticCompositionLocalOf<suspend (String) -> ByteArray?> { { null } }
