package ai.nomi.database.dao

import ai.nomi.database.entity.AgentActivityEntity
import ai.nomi.database.entity.MessageEntity
import ai.nomi.database.entity.NotificationEntity
import ai.nomi.database.entity.SessionEntity
import ai.nomi.database.entity.StreamingDraftEntity
import androidx.room.Dao
import androidx.room.Insert
import androidx.room.OnConflictStrategy
import androidx.room.Query
import androidx.room.Transaction
import kotlinx.coroutines.flow.Flow

/** Phase the backend reports for an agent that is alive but not doing anything. */
internal const val IDLE_PHASE = "waiting"

@Dao
interface SessionDao {
    @Query("SELECT * FROM sessions ORDER BY updatedAt DESC")
    fun observeAll(): Flow<List<SessionEntity>>

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun upsert(session: SessionEntity)

    @Query("UPDATE sessions SET unread = 0 WHERE id = :id")
    suspend fun markRead(id: String)
}

@Dao
interface MessageDao {
    @Query("SELECT * FROM messages WHERE sessionId = :sessionId ORDER BY createdAt ASC, id ASC")
    fun observe(sessionId: String): Flow<List<MessageEntity>>

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun upsert(messages: List<MessageEntity>)

    @Query("DELETE FROM messages WHERE id = :id")
    suspend fun delete(id: String)
}

@Dao
interface StreamingDraftDao {
    @Query("SELECT * FROM streaming_drafts WHERE sessionId = :sessionId")
    fun observe(sessionId: String): Flow<StreamingDraftEntity?>

    @Query("SELECT * FROM streaming_drafts WHERE sessionId = :sessionId")
    suspend fun get(sessionId: String): StreamingDraftEntity?

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun upsert(draft: StreamingDraftEntity)

    /** Appends [text] to the session's draft; a draft from a different turn is replaced, not extended. */
    @Transaction
    suspend fun appendDelta(sessionId: String, turnJobId: String, text: String, updatedAt: Long) {
        val existing = get(sessionId)
        val sameTurn = existing?.takeIf { it.turnJobId == turnJobId }
        val base = sameTurn?.text ?: ""
        upsert(
            StreamingDraftEntity(
                sessionId = sessionId,
                turnJobId = turnJobId,
                text = base + text,
                // A new turn starts a fresh draft, so the previous turn's phase must not leak into it.
                phase = sameTurn?.phase,
                updatedAt = updatedAt,
            ),
        )
    }

    @Query("DELETE FROM streaming_drafts WHERE sessionId = :sessionId")
    suspend fun clear(sessionId: String)
}

@Dao
interface NotificationDao {
    @Query("SELECT COUNT(*) FROM notifications WHERE readAt IS NULL")
    fun observeUnreadCount(): Flow<Int>

    @Query("SELECT * FROM notifications ORDER BY createdAt DESC")
    fun observeAll(): Flow<List<NotificationEntity>>

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun upsert(notification: NotificationEntity)
}

@Dao
interface AgentActivityDao {
    @Query("SELECT * FROM agent_activity WHERE phase != '$IDLE_PHASE' ORDER BY updatedAt DESC")
    fun observeActive(): Flow<List<AgentActivityEntity>>

    @Query("DELETE FROM agent_activity WHERE agentSessionId = :agentSessionId")
    suspend fun delete(agentSessionId: String)

    @Query("DELETE FROM agent_activity WHERE sessionId = :sessionId")
    suspend fun clearForSession(sessionId: String)

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun upsert(activity: AgentActivityEntity)
}
