package ai.nomi.data

import ai.nomi.database.NomiDatabase
import ai.nomi.database.entity.SessionEntity
import ai.nomi.network.SessionDto
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.runTest
import kotlin.test.AfterTest
import kotlin.test.BeforeTest
import kotlin.test.Test
import kotlin.test.assertEquals

class SessionRepositoryTest {
    private lateinit var db: NomiDatabase
    private val api = FakeSessionsApi()
    private val clock = FakeClock()

    @BeforeTest
    fun open() {
        db = inMemoryDb()
    }

    @AfterTest
    fun close() = db.close()

    @Test
    fun refresh_removes_chats_the_server_no_longer_lists_and_keeps_unread() = runTest {
        val repo = SessionRepository(api, db, clock, backgroundScope.openSession())
        db.sessionDao().upsert(SessionEntity("kept", "Kept", "p", 1, unread = true))
        db.sessionDao().upsert(SessionEntity("deleted-elsewhere", "Gone", "p", 1, unread = false))
        api.sessions = listOf(SessionDto(id = "kept", title = "Kept", lastMessage = null, agentActive = false, updatedAt = "2026-10-18T10:00:00Z"))

        repo.refresh()

        val rows = db.sessionDao().observeAll().first()
        assertEquals(listOf("kept"), rows.map { it.id })
        assertEquals(true, rows.single().unread)
    }
}
