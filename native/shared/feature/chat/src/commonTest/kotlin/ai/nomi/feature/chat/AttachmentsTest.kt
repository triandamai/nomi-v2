package ai.nomi.feature.chat

import kotlin.test.Test
import kotlin.test.assertEquals

class AttachmentsTest {
    @Test
    fun splitsReferenceTagsFromTheTypedText() {
        val content = "Look at these\n\n" +
            """<attachment id="a1" name="beach.jpg" kind="image" mime="image/jpeg" size="2048"/>""" + "\n\n" +
            """<attachment id="a2" name="plan.pdf" kind="pdf" mime="application/pdf" size="5242880"/>"""
        val split = splitAttachments(content)
        assertEquals("Look at these", split.text)
        assertEquals(
            listOf(
                Attachment("beach.jpg", AttachmentKind.Image, id = "a1", mime = "image/jpeg", size = 2048),
                Attachment("plan.pdf", AttachmentKind.Pdf, id = "a2", mime = "application/pdf", size = 5_242_880),
            ),
            split.files,
        )
    }

    @Test
    fun keepsOlderInlineFilesAndTheirText() {
        val split = splitAttachments("<attachment name=\"notes.txt\" kind=\"file\">\nline one\nline two\n</attachment>")
        assertEquals("", split.text)
        assertEquals(listOf(Attachment("notes.txt", AttachmentKind.InlineText, text = "line one\nline two")), split.files)
    }

    @Test
    fun leavesTextWithoutTagsAndTagsWithoutAnIdAlone() {
        assertEquals(SplitMessage("Just words", emptyList()), splitAttachments("Just words"))
        val noId = """<attachment name="x"/>"""
        assertEquals(SplitMessage(noId, emptyList()), splitAttachments(noId))
    }

    @Test
    fun unknownKindsAreOtherFiles() {
        val split = splitAttachments("""<attachment id="z" name="data.bin" kind="mystery"/>""")
        assertEquals(AttachmentKind.Other, split.files.single().kind)
    }

    @Test
    fun formatsSizesAsTheWebDoes() {
        assertEquals("512 B", formatBytes(512))
        assertEquals("1.5 KB", formatBytes(1536))
        assertEquals("13 KB", formatBytes(13 * 1024))
        assertEquals("5.0 MB", formatBytes(5L * 1024 * 1024))
        assertEquals("12 MB", formatBytes(12L * 1024 * 1024))
    }
}
