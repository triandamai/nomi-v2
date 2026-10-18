package ai.nomi.data

import ai.nomi.database.NomiDatabase
import ai.nomi.database.entity.AgentActivityEntity
import ai.nomi.realtime.RealtimeEvent
import ai.nomi.realtime.Topic
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.map

/**
 * Live crew activity, written from chat-topic agent events and read from Room. Only chats whose topic the hub is
 * subscribed to (an open chat screen) feed it: the user topic carries no agent phases.
 */
class ActivityRepository(private val db: NomiDatabase, private val clock: EpochClock) {
    private val dao get() = db.agentActivityDao()

    /** Agents not in the idle `waiting` phase, most recently changed first, across every chat. */
    fun observeActive(): Flow<List<AgentActivity>> = dao.observeActive().map { rows -> rows.map { it.toDomain() } }

    /** The most recently active agent in [sessionId], if any. */
    fun observeActive(sessionId: String): Flow<AgentActivity?> =
        observeActive().map { all -> all.firstOrNull { it.sessionId == sessionId } }.distinctUntilChanged()

    /** Forgets every agent row of [sessionId] (after a resync; live agents re-announce through phase events). */
    suspend fun clearForSession(sessionId: String) = dao.clearForSession(sessionId)

    /** Applies an agent lifecycle event; other events are ignored. */
    suspend fun onEvent(event: RealtimeEvent) {
        when (event) {
            is RealtimeEvent.AgentSessionStarted -> dao.upsert(
                AgentActivityEntity(event.agentSessionId, event.sessionId, event.agentType, STARTED_PHASE, null, clock()),
            )
            is RealtimeEvent.AgentPhaseChanged -> {
                val existing = dao.get(event.agentSessionId)
                val sessionId = existing?.sessionId ?: (event.topic as? Topic.Chat)?.sessionId ?: return
                dao.upsert(
                    existing?.copy(phase = event.phase, detail = event.detail, updatedAt = clock())
                        // Started before this client was listening: the type is unknown until the next start.
                        ?: AgentActivityEntity(event.agentSessionId, sessionId, UNKNOWN_AGENT_TYPE, event.phase, event.detail, clock()),
                )
            }
            is RealtimeEvent.AgentSessionEnded -> dao.delete(event.agentSessionId)
            else -> Unit
        }
    }

    companion object {
        /** `AgentSessionStarted` carries no phase; a freshly spawned agent is working. */
        const val STARTED_PHASE = "thinking"

        /** Agent type for an agent first seen through a phase change. */
        const val UNKNOWN_AGENT_TYPE = "unknown"
    }
}
