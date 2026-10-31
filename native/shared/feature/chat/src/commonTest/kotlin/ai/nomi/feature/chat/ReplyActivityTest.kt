package ai.nomi.feature.chat

import ai.nomi.data.Message
import ai.nomi.feature.chat.markdown.CodePalette
import ai.nomi.feature.chat.markdown.highlight
import androidx.compose.ui.graphics.Color
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

private fun message(id: String, role: String, content: String, at: Long, json: String? = null, agent: String? = null) =
    Message(id, "s1", role, null, agent, content, json, at, "sent")

private fun thought(id: String, text: String, at: Long, agent: String? = null) =
    message(id, "assistant", "", at, """[{"kind":"reasoning","text":"$text"}]""", agent)

class ReplyActivityTest {
    @Test
    fun thinkingFoldsIntoTheReplyItLedTo() {
        val entries = groupThinking(
            listOf(
                message("u1", "user", "Plan it", 1),
                thought("t1", "Check the weather", 2),
                message("legacy", "assistant", "🧠 Then the market hours", 3),
                message("a1", "assistant", "Here's the plan", 4),
            ),
        )
        assertEquals(listOf("u1", "a1"), entries.map { it.message.id })
        assertEquals(listOf("Check the weather", "Then the market hours"), entries[1].thinking)
        assertEquals(false, entries[1].thinkingOnly)
    }

    @Test
    fun thinkingWithNoReplyYetStaysWhereItBegan() {
        val entries = groupThinking(
            listOf(
                message("u1", "user", "Plan it", 1),
                thought("t1", "Working on it", 2, agent = "Koda"),
                message("a1", "assistant", "Nomi's note", 3),
            ),
        )
        assertEquals(listOf("u1", "t1", "a1"), entries.map { it.message.id })
        assertTrue(entries[1].thinkingOnly)
        assertEquals(listOf("Working on it"), entries[1].thinking)
    }

    @Test
    fun readsTheStepsAndNamesThemAsTheWebDoes() {
        val steps = parseReplySteps(
            """[{"tool":"web_search","detail":"rust","ok":true},{"tool":"delegate_to_agent","detail":"money","ok":true},{"tool":"odd_tool","ok":false,"output":"boom"}]""",
        )
        assertEquals(
            listOf("Searched the web", "Handed it to Money", "Used odd tool"),
            steps.map(::stepLabel),
        )
        assertEquals(false, steps[2].ok)
        assertEquals("boom", steps[2].output)
        assertEquals(emptyList(), parseReplySteps("not json"))
    }

    @Test
    fun phasesReadAsWhatTheCrewIsDoing() {
        assertEquals("checking your transactions", phaseActivity("calling_tool", "list_transactions"))
        assertEquals("using web search", phaseActivity("calling_tool", "web_search"))
        assertEquals("finalizing a reply", phaseActivity("writing_reply", null))
    }

    @Test
    fun codeIsColouredByKind() {
        val palette = CodePalette(Color.Red, Color.Green, Color.Blue, Color.Yellow, Color.Gray)
        val code = highlight("// note\nval n = parse(\"42\") + 7", "kotlin", palette)
        assertEquals("// note\nval n = parse(\"42\") + 7", code.text)
        fun colourOf(part: String) = code.spanStyles.first { code.text.substring(it.start, it.end) == part }.item.color
        assertEquals(Color.Gray, colourOf("// note"))
        assertEquals(Color.Red, colourOf("val"))
        assertEquals(Color.Blue, colourOf("parse"))
        assertEquals(Color.Green, colourOf("\"42\""))
        assertEquals(Color.Yellow, colourOf("7"))
    }
}
