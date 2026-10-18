package ai.nomi.data

import ai.nomi.database.NomiDatabase
import ai.nomi.database.entity.NotificationEntity
import ai.nomi.network.NotificationsApi
import ai.nomi.realtime.RealtimeEvent
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map

/** The inbox: REST and `NotificationCreated` events write Room, the UI observes Room. */
class NotificationRepository(
    private val api: NotificationsApi,
    private val db: NomiDatabase,
    private val clock: EpochClock,
    private val session: SessionScope,
) {
    private val dao get() = db.notificationDao()

    fun observeAll(): Flow<List<NotificationItem>> = dao.observeAll().map { rows -> rows.map { it.toDomain() } }

    fun observeUnreadCount(): Flow<Int> = dao.observeUnreadCount()

    /** Refetches the newest page of the inbox. */
    suspend fun refresh() = session.run {
        val now = clock()
        api.list().items.forEach { dto -> dao.upsert(dto.toEntity(dao.get(dto.id), now)) }
    }

    /** The event carries no timestamp, so arrival time stands in for `created_at` until the next refresh. */
    suspend fun onCreated(event: RealtimeEvent.NotificationCreated) = session.run {
        val existing = dao.get(event.notificationId)
        dao.upsert(
            NotificationEntity(
                id = event.notificationId,
                kind = event.category,
                title = event.title,
                body = event.body,
                link = event.link,
                createdAt = existing?.createdAt ?: clock(),
                readAt = existing?.readAt,
            ),
        )
    }

    suspend fun markRead(id: String) = session.run {
        api.markRead(id)
        val existing = dao.get(id) ?: return@run
        if (existing.readAt == null) dao.upsert(existing.copy(readAt = clock()))
    }

    suspend fun markAllRead() = session.run {
        api.markAllRead()
        dao.markAllRead(clock())
    }
}
