package ai.nomi.realtime

import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.json.JsonNull
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.contentOrNull

/** A decoded realtime event. Chat and project ids come from the [topic]; payloads do not repeat them. */
sealed interface RealtimeEvent {
    val topic: Topic

    // Chat topic (StreamEnvelope)
    data class ChatDelta(override val topic: Topic, val sessionId: String, val turnJobId: String, val event: JsonObject) : RealtimeEvent
    data class TurnCompleted(override val topic: Topic, val sessionId: String, val turnJobId: String, val messageId: String) : RealtimeEvent
    data class TurnFailed(override val topic: Topic, val sessionId: String, val turnJobId: String, val error: String) : RealtimeEvent
    data class MessageCreated(override val topic: Topic, val sessionId: String, val messageId: String) : RealtimeEvent
    data class MessageUpdated(override val topic: Topic, val sessionId: String, val messageId: String) : RealtimeEvent
    data class SessionRenamed(override val topic: Topic, val sessionId: String, val title: String) : RealtimeEvent
    data class PersonalityChanged(override val topic: Topic, val sessionId: String) : RealtimeEvent
    data class AgentDelegationUpdated(override val topic: Topic, val delegationId: String) : RealtimeEvent
    data class AgentPhaseChanged(override val topic: Topic, val agentSessionId: String, val phase: String, val detail: String?) : RealtimeEvent
    data class AgentSessionStarted(
        override val topic: Topic,
        val agentSessionId: String,
        val sessionId: String,
        val agentType: String,
        val agentDisplayName: String,
        val channel: String,
        val senderLabel: String,
    ) : RealtimeEvent
    data class AgentSessionEnded(override val topic: Topic, val agentSessionId: String, val sessionId: String, val reason: String) : RealtimeEvent

    // Project topic
    data class DeployChanged(override val topic: Topic, val projectId: String, val event: String) : RealtimeEvent

    // User topic (UserEvent)
    data class NotificationCreated(
        override val topic: Topic,
        val notificationId: String,
        val category: String,
        val title: String,
        val body: String,
        val link: String?,
    ) : RealtimeEvent
    data class ChatReply(override val topic: Topic, val sessionId: String, val messageId: String, val preview: String) : RealtimeEvent

    /**
     * The crew is done with a chat for now (`user/{id}/activity`, relayed under the user topic).
     * [outcome] is "done", "failed" or "waiting"; [waiting] is "question", "approval" or "connect" when
     * the chat waits on the person. [projectId] is set for a project's chat; [agent] answered last.
     */
    data class ChatActivity(
        override val topic: Topic,
        val sessionId: String,
        val title: String?,
        val projectId: String?,
        val outcome: String,
        val waiting: String?,
        val agent: String?,
    ) : RealtimeEvent

    /** Hub-emitted after a reconnect: state for [topic] may have been missed, refetch it. Never decoded. */
    data class Resynced(override val topic: Topic) : RealtimeEvent

    /** Hub-emitted when the server denied a subscription. Never decoded. */
    data class AccessLost(override val topic: Topic) : RealtimeEvent

    /** An unrecognised or malformed payload; kept so callers can log or ignore it. */
    data class Unknown(override val topic: Topic, val kind: String?, val raw: JsonObject) : RealtimeEvent
}

private const val KEY_KIND = "kind"
private const val KEY_TEXT_DELTA = "TextDelta"
private const val KEY_TEXT = "text"

/** The text appended by a text-delta stream event (`{"TextDelta":{"index":n,"text":"..."}}`), else null. */
fun RealtimeEvent.ChatDelta.textOrNull(): String? =
    (event[KEY_TEXT_DELTA] as? JsonObject)?.get(KEY_TEXT)?.asStringOrNull()

private fun JsonElement?.asStringOrNull(): String? = (this as? JsonPrimitive)?.takeIf { it.isString }?.contentOrNull

private class PayloadReader(private val payload: JsonObject) {
    fun str(key: String): String = payload[key].asStringOrNull() ?: error("missing string '$key'")
    fun strOrNull(key: String): String? = payload[key]?.takeUnless { it is JsonNull }?.let { it.asStringOrNull() ?: error("bad string '$key'") }
    fun obj(key: String): JsonObject = payload[key] as? JsonObject ?: error("missing object '$key'")
}

/** Decodes an `event` frame's payload. Unknown or malformed payloads become [RealtimeEvent.Unknown]; this never throws. */
fun decodeEvent(topic: Topic, payload: JsonObject): RealtimeEvent {
    val kind = payload[KEY_KIND].asStringOrNull()
    return try {
        decodeKnown(topic, kind, payload) ?: RealtimeEvent.Unknown(topic, kind, payload)
    } catch (_: IllegalStateException) {
        RealtimeEvent.Unknown(topic, kind, payload)
    }
}

private fun decodeKnown(topic: Topic, kind: String?, payload: JsonObject): RealtimeEvent? {
    val p = PayloadReader(payload)
    fun chatId(): String = (topic as? Topic.Chat)?.sessionId ?: error("chat event off a chat topic")
    return when (kind) {
        "Delta" -> RealtimeEvent.ChatDelta(topic, chatId(), p.str("turn_job_id"), p.obj("event"))
        "TurnCompleted" -> RealtimeEvent.TurnCompleted(topic, chatId(), p.str("turn_job_id"), p.str("message_id"))
        "TurnFailed" -> RealtimeEvent.TurnFailed(topic, chatId(), p.str("turn_job_id"), p.str("error"))
        "MessageCreated" -> RealtimeEvent.MessageCreated(topic, chatId(), p.str("message_id"))
        "MessageUpdated" -> RealtimeEvent.MessageUpdated(topic, chatId(), p.str("message_id"))
        "SessionRenamed" -> RealtimeEvent.SessionRenamed(topic, chatId(), p.str("title"))
        "PersonalityChanged" -> RealtimeEvent.PersonalityChanged(topic, chatId())
        "AgentDelegationUpdated" -> RealtimeEvent.AgentDelegationUpdated(topic, p.str("delegation_id"))
        "AgentPhaseChanged" -> RealtimeEvent.AgentPhaseChanged(topic, p.str("agent_session_id"), p.str("phase"), p.strOrNull("detail"))
        "AgentSessionStarted" -> RealtimeEvent.AgentSessionStarted(
            topic, p.str("agent_session_id"), p.str("session_id"), p.str("agent_type"),
            p.str("agent_display_name"), p.str("channel"), p.str("sender_label"),
        )
        "AgentSessionEnded" -> RealtimeEvent.AgentSessionEnded(topic, p.str("agent_session_id"), p.str("session_id"), p.str("reason"))
        "DeployChanged" -> RealtimeEvent.DeployChanged(
            topic,
            (topic as? Topic.Project)?.projectId ?: error("project event off a project topic"),
            p.str("event"),
        )
        "NotificationCreated" -> RealtimeEvent.NotificationCreated(
            topic, p.str("notification_id"), p.str("category"), p.str("title"), p.str("body"), p.strOrNull("link"),
        )
        "ChatReply" -> RealtimeEvent.ChatReply(topic, p.str("session_id"), p.str("message_id"), p.str("preview"))
        "ChatActivity" -> {
            if (topic != Topic.User) error("user event off the user topic")
            RealtimeEvent.ChatActivity(
                topic, p.str("session_id"), p.strOrNull("title"), p.strOrNull("project_id"),
                p.str("outcome"), p.strOrNull("waiting"), p.strOrNull("agent"),
            )
        }
        else -> null
    }
}
