package ai.nomi.data

/**
 * What the always-on companions (desktop island, Android chat head) should show. [RealtimeSync] publishes
 * [Working]/[Idle] as state (`companionState`) and the rest as one-off events (`companionSignals`).
 */
sealed interface CompanionSignal {
    /** A reply arrived in a chat that is not on screen. [agentType] is null when the reply's agent is unknown. */
    data class NewReply(val sessionId: String, val preview: String, val agentType: String?) : CompanionSignal

    data class Notification(val title: String, val body: String, val link: String?) : CompanionSignal

    /** A chat not on screen is waiting on the person; [waiting] is "question", "approval" or "connect". */
    data class NeedsYou(val sessionId: String, val title: String?, val waiting: String) : CompanionSignal

    /** The crew finished a chat not on screen without a [NewReply] for it (e.g. only a delegated agent worked). */
    data class ChatDone(val sessionId: String, val title: String?) : CompanionSignal

    /** The crew's work in a chat not on screen failed. */
    data class ChatFailed(val sessionId: String, val title: String?) : CompanionSignal

    /** Some crew agent is working; [phase] is the backend's (`thinking`, `calling_tool`, `writing`). */
    data class Working(val agentType: String, val phase: String) : CompanionSignal

    /** No agent is working. */
    data object Idle : CompanionSignal
}
