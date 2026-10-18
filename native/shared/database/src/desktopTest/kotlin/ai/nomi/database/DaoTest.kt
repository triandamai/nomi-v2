package ai.nomi.database

import ai.nomi.database.entity.AgentActivityEntity
import ai.nomi.database.entity.MessageEntity
import ai.nomi.database.entity.NotificationEntity
import ai.nomi.database.entity.SessionEntity
import ai.nomi.database.entity.StreamingDraftEntity
import androidx.room.Room
import androidx.sqlite.driver.bundled.BundledSQLiteDriver
import app.cash.turbine.test
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.runTest
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class DaoTest {
    private lateinit var db: NomiDatabase

    @BeforeTest
    fun open() {
        db = Room.inMemoryDatabaseBuilder<NomiDatabase>().setDriver(BundledSQLiteDriver()).build()
    }

    @AfterTest
    fun close() = db.close()

    private fun session(id: String, updatedAt: Long = 1L, unread: Boolean = true) =
        SessionEntity(id, "title $id", "preview", updatedAt, unread)

    private fun message(id: String, sessionId: String, createdAt: Long) =
        MessageEntity(id, sessionId, "assistant", "planner", "Planner", "hi", "[]", createdAt, "done")

    @Test
    fun sessionUpsertEmitsOrderedByRecency() = runTest {
        db.sessionDao().observeAll().test {
            assertEquals(emptyList(), awaitItem())
            db.sessionDao().upsert(session("a", updatedAt = 1))
            assertEquals(listOf("a"), awaitItem().map { it.id })
            db.sessionDao().upsert(session("b", updatedAt = 2))
            assertEquals(listOf("b", "a"), awaitItem().map { it.id })
        }
    }

    @Test
    fun markReadClearsUnread() = runTest {
        db.sessionDao().upsert(session("a", unread = true))
        db.sessionDao().markRead("a")
        assertEquals(false, db.sessionDao().observeAll().first().single().unread)
    }

    @Test
    fun messagesObservedPerSessionInOrderAndDeletable() = runTest {
        db.messageDao().upsert(listOf(message("m2", "s1", 2), message("m1", "s1", 1), message("x", "s2", 1)))
        db.messageDao().observe("s1").test {
            assertEquals(listOf("m1", "m2"), awaitItem().map { it.id })
            db.messageDao().delete("m1")
            assertEquals(listOf("m2"), awaitItem().map { it.id })
        }
    }

    @Test
    fun appendDeltaConcatenatesWithinATurn() = runTest {
        val dao = db.streamingDraftDao()
        dao.appendDelta("s1", "job1", "Hel", 1L)
        dao.appendDelta("s1", "job1", "lo", 2L)
        assertEquals(2L, dao.observe("s1").first()?.updatedAt)
        assertEquals("Hello", dao.observe("s1").first()?.text)
    }

    @Test
    fun appendDeltaReplacesDraftOfADifferentTurn() = runTest {
        val dao = db.streamingDraftDao()
        dao.appendDelta("s1", "job1", "old", 1L)
        dao.appendDelta("s1", "job2", "new", 2L)
        val draft = dao.observe("s1").first()
        assertEquals("new", draft?.text)
        assertEquals("job2", draft?.turnJobId)
    }

    @Test
    fun clearRemovesDraft() = runTest {
        val dao = db.streamingDraftDao()
        dao.appendDelta("s1", "job1", "x", 1L)
        dao.clear("s1")
        assertNull(dao.observe("s1").first())
    }

    @Test
    fun unreadCountTracksUpserts() = runTest {
        val dao = db.notificationDao()
        dao.observeUnreadCount().test {
            assertEquals(0, awaitItem())
            dao.upsert(NotificationEntity("n1", "k", "t", "b", null, 1, null))
            assertEquals(1, awaitItem())
            dao.upsert(NotificationEntity("n1", "k", "t", "b", null, 1, 5))
            assertEquals(0, awaitItem())
        }
    }

    @Test
    fun markReadLeavesOtherSessionsUntouched() = runTest {
        db.sessionDao().upsert(session("a"))
        db.sessionDao().upsert(session("b", updatedAt = 2))
        db.sessionDao().markRead("a")
        assertEquals(mapOf("a" to false, "b" to true), db.sessionDao().observeAll().first().associate { it.id to it.unread })
    }

    @Test
    fun observeActiveExcludesWaitingAndDeletedAgents() = runTest {
        val dao = db.agentActivityDao()
        dao.upsert(AgentActivityEntity("a1", "s1", "planner", "thinking", "d", 1))
        dao.upsert(AgentActivityEntity("a2", "s1", "coder", "waiting", "d", 1))
        dao.upsert(AgentActivityEntity("a3", "s2", "coder", "calling_tool", "d", 1))
        assertEquals(setOf("a1", "a3"), dao.observeActive().first().map { it.agentSessionId }.toSet())
        dao.delete("a1")
        assertEquals(listOf("a3"), dao.observeActive().first().map { it.agentSessionId })
        dao.clearForSession("s2")
        assertEquals(emptyList(), dao.observeActive().first())
    }

    @Test
    fun clearAllEmptiesEveryTable() = runTest {
        db.sessionDao().upsert(session("a"))
        db.messageDao().upsert(listOf(message("m1", "a", 1)))
        db.streamingDraftDao().appendDelta("a", "t1", "hi", 1)
        db.notificationDao().upsert(NotificationEntity("n1", "k", "t", "b", null, 1, null))
        db.agentActivityDao().upsert(AgentActivityEntity("x", "a", "planner", "thinking", null, 1))

        db.clearAll()

        assertEquals(emptyList(), db.sessionDao().observeAll().first())
        assertEquals(emptyList(), db.messageDao().observe("a").first())
        assertNull(db.streamingDraftDao().get("a"))
        assertEquals(emptyList(), db.notificationDao().observeAll().first())
        assertEquals(emptyList(), db.agentActivityDao().observeActive().first())
    }
}
