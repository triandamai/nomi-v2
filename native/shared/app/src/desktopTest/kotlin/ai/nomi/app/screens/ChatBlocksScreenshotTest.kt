package ai.nomi.app.screens

import ai.nomi.data.ChatState
import ai.nomi.data.Message
import ai.nomi.feature.chat.ApprovalState
import ai.nomi.feature.chat.ChatComposerState
import ai.nomi.feature.chat.ChatContent
import androidx.compose.runtime.Composable
import androidx.compose.ui.test.junit4.createComposeRule
import kotlinx.datetime.TimeZone
import org.junit.Rule
import kotlin.test.Test

private const val MINUTE = 60_000L

private fun reply(id: String, text: String, minutesAgo: Long, blocks: String? = null, agent: String = "coding", name: String = "Koda") =
    Message(id, "s1", "assistant", agent, name, text, blocks, SNAPSHOT_NOW - minutesAgo * MINUTE, "done")

private fun ask(id: String, text: String, minutesAgo: Long) =
    Message(id, "s1", "user", null, null, text, null, SNAPSHOT_NOW - minutesAgo * MINUTE, "done")

private val MarkdownReply = """
    ## Saturday, sorted
    The market opens at **8** and is *quietest* before 9. Here's the plan:

    1. Farmers market, 8:00
    2. Hike at Batu Hill, 11:00
       - pack water and `sunscreen`
    3. Back home by 15:00

    > Rain is likely after 2pm.

    The forecast call I used:

    ```kotlin
    val rain = forecast.hourly.first { it.chance > 0.6 }
    ```

    Trail notes are on [the park's site](https://example.org/trails).
""".trimIndent()

private val MarkdownChat = ChatState(
    messages = listOf(
        ask("u1", "Can you plan my Saturday?", 6),
        reply("m1", MarkdownReply, 5, agent = "chitchat", name = "Nomi"),
    ),
    draft = null,
    activeAgent = null,
)

private val ApprovalAndTodoChat = ChatState(
    messages = listOf(
        ask("u1", "Clean up the old landing page.", 9),
        reply(
            "m1", "Plan for the cleanup", 8,
            """[{"kind":"todo_list","items":[
                 {"id":"1","text":"Find pages nobody links to","status":"done"},
                 {"id":"2","text":"Delete the old landing page","status":"in_progress"},
                 {"id":"3","text":"Redeploy the preview","status":"pending"}]}]""",
        ),
        reply(
            "m2", "Needs approval: delete src/routes/old/+page.svelte", 7,
            """[{"kind":"approval_request","id":"a2","tool_name":"delete_file","description":"Delete src/routes/old/+page.svelte",
                 "input":{"path":"src/routes/old/+page.svelte"},"status":"pending","decided_at":null}]""",
        ),
        reply(
            "m3", "Approved", 6,
            """[{"kind":"approval_request","id":"a1","tool_name":"write_file","description":"Update the README",
                 "input":{},"status":"approved","decided_at":"2026-10-18T09:00:00Z"}]""",
        ),
    ),
    draft = null,
    activeAgent = null,
)

private val PlanAndFilesChat = ChatState(
    messages = listOf(
        ask("u1", "Draft a launch plan, then tidy the repo.", 9),
        reply(
            "m1", "Launch week", 8,
            """[{"kind":"plan","plan_id":"p1","agent_session_id":"as1","title":"Launch week for the portfolio site","version":2}]""",
            agent = "planning", name = "Planning",
        ),
        reply(
            "m2", "Wrote src/lib/hero.ts", 7,
            """[{"kind":"reasoning","text":"The hero copy lives in **one** file, so a single write is enough."},
                {"kind":"file_write","project_id":"pr","path":"src/lib/hero.ts","content":"export const title = 'Hello';\nexport const lede = 'I build things.';\n",
                 "previous_content":"export const title = 'Hi';\n"},
                {"kind":"file_delete","project_id":"pr","path":"src/routes/old/+page.svelte","previous_content":"<h1>Old</h1>\n"}]""",
        ),
        reply(
            "m3", "Here is the **comparison** you asked for: the new host is cheaper.", 6,
            """[{"kind":"table","variant":"comparison","columns":[],"rows":[]}]""",
        ),
    ),
    draft = null,
    activeAgent = null,
)

/** The pending card mid-decision: the person ticked "remember" and the request is on its way. */
private val Deciding = mapOf("m2" to ApprovalState(sending = true))

class ChatBlocksScreenshotTest {
    @get:Rule val rule = createComposeRule()

    @Composable
    private fun Chat(state: ChatState, approvals: Map<String, ApprovalState> = emptyMap()) =
        ChatContent("Weekend", state, ChatComposerState(), {}, {}, {}, onBack = {}, timeZone = TimeZone.UTC, approvals = approvals)

    @Test fun markdownPhoneLight() = rule.captureScreen("chat_markdown", Device.Phone, false) { Chat(MarkdownChat) }
    @Test fun markdownPhoneDark() = rule.captureScreen("chat_markdown", Device.Phone, true) { Chat(MarkdownChat) }
    @Test fun markdownWideLight() = rule.captureScreen("chat_markdown", Device.Wide, false) { Chat(MarkdownChat) }
    @Test fun markdownWideDark() = rule.captureScreen("chat_markdown", Device.Wide, true) { Chat(MarkdownChat) }

    @Test fun approvalPhoneLight() = rule.captureScreen("chat_approval_todo", Device.Phone, false) { Chat(ApprovalAndTodoChat) }
    @Test fun approvalPhoneDark() = rule.captureScreen("chat_approval_todo", Device.Phone, true) { Chat(ApprovalAndTodoChat) }
    @Test fun approvalWideLight() = rule.captureScreen("chat_approval_todo", Device.Wide, false) { Chat(ApprovalAndTodoChat) }
    @Test fun approvalWideDark() = rule.captureScreen("chat_approval_todo", Device.Wide, true) { Chat(ApprovalAndTodoChat) }
    @Test fun approvalSendingPhoneLight() = rule.captureScreen("chat_approval_sending", Device.Phone, false) { Chat(ApprovalAndTodoChat, Deciding) }

    @Test fun planFilesPhoneLight() = rule.captureScreen("chat_plan_files", Device.Phone, false) { Chat(PlanAndFilesChat) }
    @Test fun planFilesPhoneDark() = rule.captureScreen("chat_plan_files", Device.Phone, true) { Chat(PlanAndFilesChat) }
    @Test fun planFilesWideLight() = rule.captureScreen("chat_plan_files", Device.Wide, false) { Chat(PlanAndFilesChat) }
    @Test fun planFilesWideDark() = rule.captureScreen("chat_plan_files", Device.Wide, true) { Chat(PlanAndFilesChat) }
}
