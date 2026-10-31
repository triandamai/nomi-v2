package ai.nomi.companion

import ai.nomi.data.AgentActivity
import ai.nomi.data.ChatState
import ai.nomi.data.Draft
import ai.nomi.data.MESSAGE_STATUS_FAILED
import ai.nomi.data.MESSAGE_STATUS_SENT
import ai.nomi.data.Message
import ai.nomi.data.Session
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertTrue

class CompanionOverviewTest {
    private fun session(id: String, updatedAt: Long, unread: Boolean = false, title: String = "Chat $id") =
        Session(id, title, "preview $id", updatedAt, unread)

    private fun agent(type: String, sessionId: String, updatedAt: Long, phase: String = "thinking") =
        AgentActivity("a-$type-$sessionId", sessionId, type, phase, null, updatedAt)

    private fun message(id: String, role: String, content: String, status: String = MESSAGE_STATUS_SENT) =
        Message(id, "s", role, null, null, content, null, createdAt = 0, status = status)

    @Test
    fun namesTheCrewAndPlacesThemInTheirChats() {
        val crew = crewAtWork(
            active = listOf(agent("money", "s1", 1), agent("research", "s2", 2), agent("money", "s1", 3)),
            sessions = listOf(session("s1", 1, title = "Budget"), session("s2", 2, title = "")),
            roster = mapOf("money" to "Finley"),
        )
        assertEquals(listOf("Finley", "Research"), crew.map { it.name })
        assertEquals(listOf("Budget", "New chat"), crew.map { it.chatTitle })
    }

    @Test
    fun listsUnreadChatsFirstThenTheMostRecent() {
        val chats = recentChats(
            sessions = listOf(session("old", 1), session("new", 5), session("unread", 2, unread = true), session("mid", 3)),
            active = listOf(agent("money", "mid", 1)),
        )
        assertEquals(listOf("unread", "new", "mid"), chats.map { it.id })
        assertTrue(chats.single { it.id == "mid" }.working)
    }

    @Test
    fun aQuestionWaitsThenStreamsThenIsAnswered() {
        val asked = listOf(message("q", "user", "Hi?"))
        val waiting = conversationOf("s", ChatState(asked, draft = null, activeAgent = null))
        assertTrue(waiting.answering)
        assertNull(waiting.answer)

        val streaming = conversationOf("s", ChatState(asked, Draft("t", "Hel", "writing", 1), activeAgent = null))
        assertEquals("Hel", streaming.answer)
        assertTrue(streaming.answering)

        val answered = conversationOf("s", ChatState(asked + message("r", "assistant", "Hello!"), draft = null, activeAgent = null))
        assertEquals("Hello!", answered.answer)
        assertFalse(answered.answering)
    }

    @Test
    fun anOlderReplyDoesntAnswerANewQuestion() {
        val messages = listOf(message("q1", "user", "One?"), message("r1", "assistant", "One."), message("q2", "user", "Two?"))
        val state = conversationOf("s", ChatState(messages, draft = null, activeAgent = null))
        assertEquals("Two?", state.question)
        assertNull(state.answer)
        assertTrue(state.answering)
    }

    @Test
    fun failuresStopTheWait() {
        val unsent = conversationOf("s", ChatState(listOf(message("q", "user", "Hi?", MESSAGE_STATUS_FAILED)), null, null))
        assertFalse(unsent.answering)
        assertEquals("Not sent. Open the chat to retry.", unsent.error)

        val failed = conversationOf("s", ChatState(listOf(message("q", "user", "Hi?")), null, null, lastError = "model overloaded"))
        assertFalse(failed.answering)
        assertEquals("model overloaded", failed.error)
    }
}
