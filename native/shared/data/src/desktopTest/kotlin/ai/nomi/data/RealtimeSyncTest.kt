package ai.nomi.data

import ai.nomi.database.NomiDatabase
import ai.nomi.database.entity.SessionEntity
import ai.nomi.network.NotificationDto
import ai.nomi.network.NotificationPageDto
import ai.nomi.network.SessionDto
import ai.nomi.realtime.RealtimeEvent
import ai.nomi.realtime.Topic
import app.cash.turbine.test
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.runTest
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals

class RealtimeSyncTest {
    private lateinit var db: NomiDatabase
    private val hub = FakeRealtimeHub()
    private val clock = FakeClock()
    private val sessionsApi = FakeSessionsApi()
    private val notificationsApi = FakeNotificationsApi()
    private val foreground = ForegroundChat()

    @BeforeTest
    fun open() {
        db = inMemoryDb()
    }

    @AfterTest
    fun close() = db.close()

    private lateinit var activity: ActivityRepository

    private fun TestScope.sync(): RealtimeSync {
        activity = ActivityRepository(db, clock)
        val session = backgroundScope.openSession()
        return RealtimeSync(
            hub,
            SessionRepository(sessionsApi, db, clock, session),
            NotificationRepository(notificationsApi, db, clock, session),
            activity,
            foreground,
            session,
        )
    }

    @Test
    fun chat_reply_for_open_chat_is_not_unread() = runTest {
        val sync = sync()
        db.sessionDao().upsert(SessionEntity("open", "Open", "old", 1, unread = false))
        db.sessionDao().upsert(SessionEntity("other", "Other", "old", 1, unread = false))
        sync.start()
        foreground.enter("open")

        hub.emit(RealtimeEvent.ChatReply(Topic.User, "open", "m1", "seen it"))
        hub.emit(RealtimeEvent.ChatReply(Topic.User, "other", "m2", "news"))

        val rows = db.sessionDao().observeAll().first { all -> all.all { it.lastMessagePreview != "old" } }
            .associateBy { it.id }
        assertEquals(false, rows.getValue("open").unread)
        assertEquals("seen it", rows.getValue("open").lastMessagePreview)
        assertEquals(true, rows.getValue("other").unread)
        assertEquals(clock.now, rows.getValue("other").updatedAt)
    }

    @Test
    fun notification_created_increments_unread() = runTest {
        val sync = sync()
        sync.start()
        sync.companionSignals.test {
            hub.emit(RealtimeEvent.NotificationCreated(Topic.User, "n1", "reply", "Title", "Body", "/chat/s1"))
            assertEquals(CompanionSignal.Notification("Title", "Body", "/chat/s1"), awaitItem())
        }
        db.notificationDao().observeUnreadCount().first { it == 1 }
        val stored = db.notificationDao().get("n1")
        assertEquals("reply", stored?.kind)
        assertEquals(clock.now, stored?.createdAt)
    }

    @Test
    fun reply_in_background_chat_signals_new_reply() = runTest {
        val sync = sync()
        sync.start()
        foreground.enter("open")
        sync.companionSignals.test {
            hub.emit(RealtimeEvent.ChatReply(Topic.User, "open", "m1", "visible already"))
            hub.emit(RealtimeEvent.ChatReply(Topic.User, "bg", "m2", "psst"))
            assertEquals(CompanionSignal.NewReply("bg", "psst", agentType = null), awaitItem())
        }
    }

    @Test
    fun agent_activity_drives_working_and_idle() = runTest {
        val sync = sync()
        sync.start()
        assertEquals(CompanionSignal.Idle, sync.companionState.value)
        activity.onEvent(RealtimeEvent.AgentSessionStarted(Topic.Chat("s1"), "a1", "s1", "coder", "Coder", "web", "you"))
        sync.companionState.first { it == CompanionSignal.Working("coder", ActivityRepository.STARTED_PHASE) }
        activity.onEvent(RealtimeEvent.AgentPhaseChanged(Topic.Chat("s1"), "a1", "writing", null))
        sync.companionState.first { it == CompanionSignal.Working("coder", "writing") }
        activity.onEvent(RealtimeEvent.AgentSessionEnded(Topic.Chat("s1"), "a1", "s1", "done"))
        sync.companionState.first { it == CompanionSignal.Idle }
    }

    @Test
    fun late_collector_sees_current_working_state() = runTest {
        val sync = sync()
        sync.start()
        activity.onEvent(RealtimeEvent.AgentSessionStarted(Topic.Chat("s1"), "a1", "s1", "coder", "Coder", "web", "you"))
        sync.companionState.first { it is CompanionSignal.Working }
        // A companion that starts collecting only now still gets the current state, not nothing.
        assertEquals(CompanionSignal.Working("coder", ActivityRepository.STARTED_PHASE), sync.companionState.first())
    }

    @Test
    fun resynced_refetches_sessions_and_notifications_keeping_unread() = runTest {
        val sync = sync()
        db.sessionDao().upsert(SessionEntity("s1", "Old", "p", 1, unread = true))
        sessionsApi.sessions = listOf(
            SessionDto(id = "s1", title = "New title", lastMessage = null, agentActive = false, updatedAt = "2026-10-18T10:00:00Z"),
        )
        notificationsApi.page = NotificationPageDto(
            items = listOf(NotificationDto("n1", "reply", "T", "B", null, read = false, createdAt = "2026-10-18T10:00:00Z")),
            unread = 1,
        )
        sync.start()
        hub.emit(RealtimeEvent.Resynced(Topic.User))

        val session = db.sessionDao().observeAll().first { it.singleOrNull()?.title == "New title" }.single()
        assertEquals(true, session.unread)
        db.notificationDao().observeUnreadCount().first { it == 1 }
        assertEquals(1, sessionsApi.listCalls)
        assertEquals(1, notificationsApi.listCalls)
    }

    @Test
    fun start_is_idempotent_and_stop_removes_the_listener() = runTest {
        val sync = sync()
        sync.start()
        sync.start()
        assertEquals(1, hub.listenerCount(Topic.User))
        sync.stop()
        assertEquals(0, hub.listenerCount(Topic.User))
    }
}
