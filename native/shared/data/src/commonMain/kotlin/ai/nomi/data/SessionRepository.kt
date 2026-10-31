package ai.nomi.data

import ai.nomi.sdk.SessionFilter
import ai.nomi.sdk.sessions
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map

/** The most chats the list shows; the engine caps every query here. */
private const val SESSION_LIMIT = 500u

/** The chat list, kept by the Rust engine: [refresh] and realtime events write it, the UI observes it. */
class SessionRepository(private val engine: NomiEngine) {
    fun observeAll(): Flow<List<Session>> = engine.fromAccount(emptyList()) { store ->
        store.sessions(SessionFilter(search = null, unreadOnly = false, limit = SESSION_LIMIT))
            .map { rows -> rows.map { it.toDomain() } }
    }

    /** Replaces the list with the server's, keeping each chat's unread flag. */
    suspend fun refresh() = translated { engine.get().refreshSessions() }

    suspend fun markRead(sessionId: String) = translated { engine.account().markSessionRead(sessionId) }

    /** Creates an empty chat server-side; it appears in the list once its first message is sent. */
    suspend fun create(): String = translated { engine.get().createSession() }
}
