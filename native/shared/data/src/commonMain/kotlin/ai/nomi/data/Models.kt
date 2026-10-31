package ai.nomi.data

/** A chat in the list. [title] is empty until the server names the chat. */
data class Session(
    val id: String,
    val title: String,
    val lastMessagePreview: String,
    val updatedAt: Long,
    val unread: Boolean,
)

/**
 * One stored chat message. [role] is `user` or `assistant`; [content] is the markdown text and [contentJson] the
 * server's content-blocks JSON array (null when the message has none). [status] is `sent` for what the server stored,
 * `sending` or `failed` for the person's message still in the engine's outbox.
 */
data class Message(
    val id: String,
    val sessionId: String,
    val role: String,
    val agentType: String?,
    val agentDisplayName: String?,
    val content: String,
    val contentJson: String?,
    val createdAt: Long,
    val status: String,
)

/** The assistant reply streaming in for the current turn. */
data class Draft(val turnJobId: String, val text: String, val phase: String?, val updatedAt: Long)

/** A crew agent that is alive in a chat; [phase] is the backend's (`thinking`, `calling_tool`, `writing`, `waiting`). */
data class AgentActivity(
    val agentSessionId: String,
    val sessionId: String,
    val agentType: String,
    val phase: String,
    val detail: String?,
    val updatedAt: Long,
)

data class NotificationItem(
    val id: String,
    val kind: String,
    val title: String,
    val body: String,
    val link: String?,
    val createdAt: Long,
    val readAt: Long?,
)

/**
 * Everything the chat screen renders, read from the engine's store only.
 * [lastError] is the reason the latest turn failed (cleared by the next delta, completed turn or send);
 * [accessLost] means the server denied this chat's realtime topic (deleted, or access revoked).
 */
data class ChatState(
    val messages: List<Message>,
    val draft: Draft?,
    val activeAgent: AgentActivity?,
    val lastError: String? = null,
    val accessLost: Boolean = false,
)

/** [Message.status] values. */
const val MESSAGE_STATUS_SENT = "sent"
const val MESSAGE_STATUS_SENDING = "sending"
const val MESSAGE_STATUS_FAILED = "failed"
