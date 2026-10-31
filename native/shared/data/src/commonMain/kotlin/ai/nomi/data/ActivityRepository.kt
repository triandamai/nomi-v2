package ai.nomi.data

import ai.nomi.sdk.activeAgents
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map

/**
 * Live crew activity, kept by the Rust engine from chat-topic agent events. Only chats kept in sync (an open chat
 * screen) feed it: the user topic carries no agent phases.
 */
class ActivityRepository(private val engine: NomiEngine) {
    /** Agents not in the idle `waiting` phase, most recently changed first, across every chat. */
    fun observeActive(): Flow<List<AgentActivity>> = engine.fromAccount(emptyList()) { store ->
        store.activeAgents().map { rows -> rows.map { it.toDomain() } }
    }

    /** The most recently active agent in [sessionId], if any. */
    fun observeActive(sessionId: String): Flow<AgentActivity?> = engine.fromAccount(null) { store ->
        store.activeAgents(sessionId).map { rows -> rows.firstOrNull()?.toDomain() }
    }.distinctUntilChanged()

    companion object {
        /** The agent type the engine records for an agent first seen through a phase change. */
        const val UNKNOWN_AGENT_TYPE = "unknown"
    }
}
