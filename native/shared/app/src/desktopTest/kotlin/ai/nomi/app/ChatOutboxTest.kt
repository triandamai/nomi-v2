package ai.nomi.app

import ai.nomi.data.ChatState
import ai.nomi.data.MESSAGE_STATUS_FAILED
import ai.nomi.data.MESSAGE_STATUS_SENDING
import ai.nomi.data.MESSAGE_STATUS_SENT
import ai.nomi.data.Message
import ai.nomi.designsystem.theme.NomiTheme
import ai.nomi.feature.chat.ChatComposerState
import ai.nomi.feature.chat.ChatContent
import androidx.compose.ui.test.assertCountEquals
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onAllNodesWithText
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import kotlinx.datetime.TimeZone
import org.junit.Rule
import kotlin.test.Test
import kotlin.test.assertEquals

/** The person's messages that haven't reached the backend: what they say, and what their actions send back. */
class ChatOutboxTest {
    @get:Rule val rule = createComposeRule()

    private fun message(id: String, text: String, status: String, at: Long) =
        Message(id, "s1", "user", null, null, text, null, at, status)

    @Test
    fun a_failed_message_offers_retry_and_discard_and_a_queued_one_says_it_is_sending() {
        val retried = mutableListOf<String>()
        val discarded = mutableListOf<String>()
        val chat = ChatState(
            messages = listOf(
                message("m1", "Delivered", MESSAGE_STATUS_SENT, at = 1),
                message("local-a", "Didn't make it", MESSAGE_STATUS_FAILED, at = 2),
                message("local-b", "On its way", MESSAGE_STATUS_SENDING, at = 3),
            ),
            draft = null,
            activeAgent = null,
        )
        rule.setContent {
            NomiTheme(reducedMotion = true) {
                ChatContent(
                    "Chat", chat, ChatComposerState(), {}, {}, {}, onBack = null, timeZone = TimeZone.UTC,
                    onRetrySend = { retried += it },
                    onDiscardSend = { discarded += it },
                )
            }
        }

        rule.onNodeWithText("Not sent").assertExists()
        rule.onAllNodesWithText("Not sent").assertCountEquals(1)
        rule.onNodeWithText("Sending…").assertExists()

        rule.onNodeWithText("Retry").performClick()
        rule.onNodeWithText("Discard").performClick()

        assertEquals(listOf("local-a"), retried)
        assertEquals(listOf("local-a"), discarded)
    }
}
