package ai.nomi.data

/** What the always-on companions (desktop island, Android chat head) should show; emitted by [RealtimeSync]. */
sealed interface CompanionSignal {
    /** A reply arrived in a chat that is not on screen. [agentType] is null when the reply's agent is unknown. */
    data class NewReply(val sessionId: String, val preview: String, val agentType: String?) : CompanionSignal

    data class Notification(val title: String, val body: String, val link: String?) : CompanionSignal

    /** Some crew agent is working; [phase] is the backend's (`thinking`, `calling_tool`, `writing`). */
    data class Working(val agentType: String, val phase: String) : CompanionSignal

    /** No agent is working. */
    data object Idle : CompanionSignal
}
