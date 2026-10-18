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
    suspend fun appendDelta(sessionId: String, turnJobId: String, text: String, updatedAt: Long = 0L) {
        val existing = get(sessionId)
        val base = if (existing != null && existing.turnJobId == turnJobId) existing.text else ""
        upsert(
            StreamingDraftEntity(
                sessionId = sessionId,
                turnJobId = turnJobId,
                text = base + text,
                phase = existing?.takeIf { it.turnJobId == turnJobId }?.phase,
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
    @Query("SELECT * FROM agent_activity WHERE phase NOT IN ('idle', 'done') ORDER BY updatedAt DESC")
    fun observeActive(): Flow<List<AgentActivityEntity>>

    @Insert(onConflict = OnConflictStrategy.REPLACE)
    suspend fun upsert(activity: AgentActivityEntity)
}
