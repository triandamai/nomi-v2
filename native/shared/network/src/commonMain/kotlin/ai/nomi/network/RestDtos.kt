package ai.nomi.network

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.JsonElement

// Wire shapes of the chat and inbox REST endpoints (backend/crates/nomi-server/src/routes/{sessions,notifications}.rs).
// Timestamps are chrono `DateTime<Utc>` serialised as RFC 3339 strings; the data layer converts them to epoch millis.

private const val NO_COUNT = 0L
private const val FIRST_PAGE = 1L

/** `LastMessagePreview`. */
@Serializable
data class LastMessageDto(
    val content: String,
    @SerialName("created_at") val createdAt: String,
)

/** `SessionListItem` from `GET /api/sessions`. */
@Serializable
data class SessionDto(
    val id: String,
    val title: String? = null,
    @SerialName("last_message") val lastMessage: LastMessageDto? = null,
    @SerialName("agent_active") val agentActive: Boolean = false,
    @SerialName("updated_at") val updatedAt: String,
    val channel: String = "",
    @SerialName("chat_type") val chatType: String = "",
    @SerialName("chat_id") val chatId: String = "",
    @SerialName("project_id") val projectId: String? = null,
)

@Serializable
internal data class ListSessionsResponse(val sessions: List<SessionDto>)

@Serializable
internal data class CreateSessionResponse(@SerialName("session_id") val sessionId: String)

/**
 * `MessageItem`: [sender] is `"user"` or `"assistant"`; [contentBlocks] is the structured content (cards, tables,
 * approvals…) kept as raw JSON, [content] the plain/markdown text. `steps`/`sources` are not mirrored yet.
 */
@Serializable
data class MessageDto(
    val id: String,
    val sender: String,
    val content: String,
    @SerialName("content_blocks") val contentBlocks: JsonElement? = null,
    @SerialName("created_at") val createdAt: String,
    @SerialName("agent_display_name") val agentDisplayName: String? = null,
    @SerialName("my_feedback") val myFeedback: String? = null,
    @SerialName("memory_count") val memoryCount: Long = NO_COUNT,
    @SerialName("author_name") val authorName: String? = null,
)

/** `ListMessagesResponse`: oldest first; [hasMore] means older messages exist (ask again with `before`). */
@Serializable
data class MessagePageDto(
    val messages: List<MessageDto>,
    @SerialName("has_more") val hasMore: Boolean = false,
)

@Serializable
internal data class SendMessageRequest(val text: String)

/** `PUT /api/sessions/:id/messages/:message_id/approval`: [decision] is `approve` or `deny`. */
@Serializable
internal data class ApprovalDecisionRequest(val decision: String, val remember: Boolean)

/** `IngestMessageResponse`: the stored user message, plus the supervisor's reply when the text was a stop command. */
@Serializable
data class SendMessageResponseDto(
    @SerialName("user_message") val userMessage: MessageDto,
    @SerialName("supervisor_reply") val supervisorReply: MessageDto? = null,
)

/** `NotificationItem`: the server reports only whether it was read, not when. */
@Serializable
data class NotificationDto(
    val id: String,
    val kind: String,
    val title: String,
    val body: String,
    val link: String? = null,
    val read: Boolean,
    @SerialName("created_at") val createdAt: String,
)

/** `InboxPage` from `GET /api/notifications` (`counts` is not mirrored). */
@Serializable
data class NotificationPageDto(
    val items: List<NotificationDto>,
    val unread: Long,
    val page: Long = FIRST_PAGE,
    val pages: Long = FIRST_PAGE,
    val total: Long = NO_COUNT,
)

/** `LatestNotification`. */
@Serializable
data class LatestNotificationDto(val id: String, val title: String, val link: String? = null)

/** `UnreadCount` from `GET /api/notifications/unread`. */
@Serializable
data class UnreadCountDto(val unread: Long, val latest: LatestNotificationDto? = null)
