package ai.nomi.database.entity

import androidx.room.Entity
import androidx.room.Index
import androidx.room.PrimaryKey

/** A chat session row. Timestamps throughout are epoch milliseconds. */
@Entity(tableName = "sessions")
data class SessionEntity(
    @PrimaryKey val id: String,
    val title: String,
    val lastMessagePreview: String,
    val updatedAt: Long,
    val unread: Boolean,
)

/** One chat message; [contentJson] is the content-blocks JSON exactly as the server sent it. */
@Entity(tableName = "messages", indices = [Index("sessionId")])
data class MessageEntity(
    @PrimaryKey val id: String,
    val sessionId: String,
    val role: String,
    val agentType: String?,
    val agentDisplayName: String?,
    val contentJson: String,
    val createdAt: Long,
    val status: String,
)

/** In-flight assistant reply for a session; removed on TurnCompleted. */
@Entity(tableName = "streaming_drafts")
data class StreamingDraftEntity(
    @PrimaryKey val sessionId: String,
    val turnJobId: String,
    val text: String,
    val phase: String?,
    val updatedAt: Long,
)

@Entity(tableName = "notifications")
data class NotificationEntity(
    @PrimaryKey val id: String,
    val kind: String,
    val title: String,
    val body: String,
    val link: String?,
    val createdAt: Long,
    val readAt: Long?,
)

@Entity(tableName = "agent_activity")
data class AgentActivityEntity(
    @PrimaryKey val agentSessionId: String,
    val sessionId: String,
    val agentType: String,
    val phase: String,
    val detail: String?,
    val updatedAt: Long,
)
