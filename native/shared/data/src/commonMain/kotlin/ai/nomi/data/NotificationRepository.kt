package ai.nomi.data

import ai.nomi.sdk.notifications
import ai.nomi.sdk.unreadNotificationCount
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map

/** The most notifications the inbox shows; the engine caps every query here. */
private const val NOTIFICATION_LIMIT = 500u
private const val NONE_UNREAD = 0

/** The inbox, kept by the Rust engine: [refresh] and realtime events write it, the UI observes it. */
class NotificationRepository(private val engine: NomiEngine) {
    fun observeAll(): Flow<List<NotificationItem>> = engine.fromAccount(emptyList()) { store ->
        store.notifications(NOTIFICATION_LIMIT).map { rows -> rows.map { it.toDomain() } }
    }

    fun observeUnreadCount(): Flow<Int> = engine.fromAccount(NONE_UNREAD) { store ->
        store.unreadNotificationCount().map { it.toInt() }
    }

    /** Refetches the newest page of the inbox. */
    suspend fun refresh() = translated { engine.get().refreshNotifications() }

    /** Read here at once; the engine tells the backend, retrying until it has it. */
    suspend fun markRead(id: String) = translated { engine.get().markNotificationRead(id) }

    suspend fun markAllRead() = translated { engine.get().markAllNotificationsRead() }
}
