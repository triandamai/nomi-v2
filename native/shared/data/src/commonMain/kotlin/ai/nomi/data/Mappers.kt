package ai.nomi.data

import ai.nomi.database.entity.AgentActivityEntity
import ai.nomi.database.entity.MessageEntity
import ai.nomi.database.entity.NotificationEntity
import ai.nomi.database.entity.SessionEntity
import ai.nomi.database.entity.StreamingDraftEntity
import ai.nomi.network.MessageDto
import ai.nomi.network.NotificationDto
import ai.nomi.network.SessionDto
import kotlinx.serialization.json.JsonNull
import kotlin.time.Instant

/** Status of a message the server has stored (the only kind written today; no optimistic rows). */
internal const val MESSAGE_STATUS_SENT = "sent"
internal const val ROLE_ASSISTANT = "assistant"

/** Parses the backend's RFC 3339 timestamps (chrono `DateTime<Utc>`) to epoch millis. */
internal fun parseServerTime(value: String): Long = Instant.parse(value).toEpochMilliseconds()

internal fun MessageDto.toEntity(sessionId: String) = MessageEntity(
    id = id,
    sessionId = sessionId,
    role = sender,
    // The REST message carries only the display name; the agent type is known for live activity, not history.
    agentType = null,
    agentDisplayName = agentDisplayName,
    content = content,
    contentJson = contentBlocks?.takeUnless { it is JsonNull }?.toString(),
    createdAt = parseServerTime(createdAt),
    status = MESSAGE_STATUS_SENT,
)

/** [existing] keeps the local unread flag: the server has no notion of it. */
internal fun SessionDto.toEntity(existing: SessionEntity?) = SessionEntity(
    id = id,
    title = title.orEmpty(),
    lastMessagePreview = lastMessage?.content.orEmpty(),
    updatedAt = parseServerTime(updatedAt),
    unread = existing?.unread ?: false,
)

/** The server only says whether a notification was read; a newly-seen read one is stamped [now]. */
internal fun NotificationDto.toEntity(existing: NotificationEntity?, now: Long) = NotificationEntity(
    id = id,
    kind = kind,
    title = title,
    body = body,
    link = link,
    createdAt = parseServerTime(createdAt),
    readAt = if (read) existing?.readAt ?: now else null,
)

internal fun SessionEntity.toDomain() = Session(id, title, lastMessagePreview, updatedAt, unread)

internal fun MessageEntity.toDomain() =
    Message(id, sessionId, role, agentType, agentDisplayName, content, contentJson, createdAt, status)

internal fun StreamingDraftEntity.toDomain() = Draft(turnJobId, text, phase, updatedAt)

internal fun AgentActivityEntity.toDomain() = AgentActivity(agentSessionId, sessionId, agentType, phase, detail, updatedAt)

internal fun NotificationEntity.toDomain() = NotificationItem(id, kind, title, body, link, createdAt, readAt)
