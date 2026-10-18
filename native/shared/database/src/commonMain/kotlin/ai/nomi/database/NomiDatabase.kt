package ai.nomi.database

import ai.nomi.database.dao.AgentActivityDao
import ai.nomi.database.dao.MessageDao
import ai.nomi.database.dao.NotificationDao
import ai.nomi.database.dao.SessionDao
import ai.nomi.database.dao.StreamingDraftDao
import ai.nomi.database.entity.AgentActivityEntity
import ai.nomi.database.entity.MessageEntity
import ai.nomi.database.entity.NotificationEntity
import ai.nomi.database.entity.SessionEntity
import ai.nomi.database.entity.StreamingDraftEntity
import androidx.room.ConstructedBy
import androidx.room.Database
import androidx.room.RoomDatabase
import androidx.room.RoomDatabaseConstructor

internal const val DATABASE_VERSION = 1

@Database(
    entities = [
        SessionEntity::class,
        MessageEntity::class,
        StreamingDraftEntity::class,
        NotificationEntity::class,
        AgentActivityEntity::class,
    ],
    version = DATABASE_VERSION,
    exportSchema = true,
)
@ConstructedBy(NomiDatabaseConstructor::class)
abstract class NomiDatabase : RoomDatabase() {
    abstract fun sessionDao(): SessionDao
    abstract fun messageDao(): MessageDao
    abstract fun streamingDraftDao(): StreamingDraftDao
    abstract fun notificationDao(): NotificationDao
    abstract fun agentActivityDao(): AgentActivityDao
}

// Room's KSP processor generates the `actual` for every target.
@Suppress("KotlinNoActualForExpect")
expect object NomiDatabaseConstructor : RoomDatabaseConstructor<NomiDatabase> {
    override fun initialize(): NomiDatabase
}
