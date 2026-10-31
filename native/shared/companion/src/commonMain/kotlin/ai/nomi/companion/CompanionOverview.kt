package ai.nomi.companion

import ai.nomi.data.AgentActivity
import ai.nomi.data.ChatState
import ai.nomi.data.MESSAGE_STATUS_FAILED
import ai.nomi.data.Session

/** Chats the companion lists: the few most recent, unread first. */
const val COMPANION_CHATS_SHOWN = 3

/** Project shortcuts the companion offers. */
const val COMPANION_PROJECTS_SHOWN = 4

private const val ROLE_USER = "user"
private const val ROLE_ASSISTANT = "assistant"
private const val UNTITLED_CHAT = "New chat"

/** A crew agent working right now, and in which chat. [name] is the crew member's name when the roster is known. */
data class CompanionCrewMember(
    val agentType: String,
    val name: String,
    val phase: String,
    val sessionId: String,
    val chatTitle: String,
)

/** A recent chat, to jump back into from the companion. */
data class CompanionChat(val id: String, val title: String, val preview: String, val unread: Boolean, val working: Boolean)

/** A project shortcut: [sessionId] is the project's own chat, where it opens. */
data class CompanionProject(val id: String, val sessionId: String, val name: String, val status: String)

/**
 * What the person asked from the companion and the answer coming in: [answer] is the reply so far (the streaming
 * draft, then the stored message); [phase] says what the crew is doing while there's nothing to read yet.
 */
data class CompanionConversation(
    val sessionId: String,
    val question: String?,
    val answer: String?,
    val answering: Boolean,
    val phase: String?,
    val error: String?,
)

/** Everything the expanded companion shows besides the card that opened it. */
data class CompanionOverview(
    val crew: List<CompanionCrewMember> = emptyList(),
    val chats: List<CompanionChat> = emptyList(),
    val projects: List<CompanionProject> = emptyList(),
    val conversation: CompanionConversation? = null,
)

/** The crew at work, named from [roster] (agent type to crew name) and placed in their chats. */
fun crewAtWork(active: List<AgentActivity>, sessions: List<Session>, roster: Map<String, String>): List<CompanionCrewMember> {
    val titles = sessions.associate { it.id to it.title }
    return active
        .sortedByDescending { it.updatedAt }
        .distinctBy { it.agentType to it.sessionId }
        .map { agent ->
            CompanionCrewMember(
                agentType = agent.agentType,
                name = roster[agent.agentType] ?: agent.agentType.replaceFirstChar { it.uppercase() },
                phase = agent.phase,
                sessionId = agent.sessionId,
                chatTitle = titles[agent.sessionId]?.ifBlank { null } ?: UNTITLED_CHAT,
            )
        }
}

/** The chats worth jumping back into: unread ones first, then the most recent. */
fun recentChats(sessions: List<Session>, active: List<AgentActivity>): List<CompanionChat> {
    val working = active.map { it.sessionId }.toSet()
    return sessions
        .sortedWith(compareByDescending<Session> { it.unread }.thenByDescending { it.updatedAt })
        .take(COMPANION_CHATS_SHOWN)
        .map { CompanionChat(it.id, it.title.ifBlank { UNTITLED_CHAT }, it.lastMessagePreview, it.unread, it.id in working) }
}

/** The conversation in [sessionId] as the companion shows it: the latest question and what answers it. */
fun conversationOf(sessionId: String, chat: ChatState): CompanionConversation {
    val messages = chat.messages
    val lastQuestion = messages.indexOfLast { it.role == ROLE_USER }
    val question = messages.getOrNull(lastQuestion)
    val reply = messages.drop(lastQuestion + 1).lastOrNull { it.role == ROLE_ASSISTANT }
    val draft = chat.draft?.text?.ifBlank { null }
    val failedToSend = question?.status == MESSAGE_STATUS_FAILED
    val answering = !failedToSend && reply == null && chat.lastError == null && question != null
    return CompanionConversation(
        sessionId = sessionId,
        question = question?.content,
        answer = draft ?: reply?.content,
        answering = answering || draft != null,
        phase = chat.draft?.phase ?: chat.activeAgent?.phase,
        error = when {
            failedToSend -> "Not sent. Open the chat to retry."
            else -> chat.lastError
        },
    )
}
