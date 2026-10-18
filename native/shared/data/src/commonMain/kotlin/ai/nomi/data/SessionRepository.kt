package ai.nomi.data

import ai.nomi.database.NomiDatabase
import ai.nomi.database.entity.SessionEntity
import ai.nomi.network.SessionsApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map

/** The chat list: REST and `ChatReply` events write Room, the UI observes Room. */
class SessionRepository(
    private val api: SessionsApi,
    private val db: NomiDatabase,
    private val clock: EpochClock,
) {
    private val dao get() = db.sessionDao()

    fun observeAll(): Flow<List<Session>> = dao.observeAll().map { rows -> rows.map { it.toDomain() } }

    /**
     * Refetches the list, keeping each row's local unread flag. Chats deleted elsewhere are not removed yet (the
     * server list is the person's whole history, so a full replace would be safe once SessionDao can delete).
     */
    suspend fun refresh() {
        api.list().forEach { dto -> dao.upsert(dto.toEntity(dao.get(dto.id))) }
    }

    /** A new reply landed in [sessionId]; [unread] is false when that chat is open on screen. */
    suspend fun onReply(sessionId: String, preview: String, unread: Boolean) {
        val now = clock()
        val existing = dao.get(sessionId)
        dao.upsert(
            existing?.copy(lastMessagePreview = preview, updatedAt = now, unread = unread)
                ?: SessionEntity(sessionId, title = "", lastMessagePreview = preview, updatedAt = now, unread = unread),
        )
    }

    suspend fun markRead(sessionId: String) = dao.markRead(sessionId)

    /** Creates an empty chat server-side; it appears in the list once its first message is sent. */
    suspend fun create(): String = api.create()
}
