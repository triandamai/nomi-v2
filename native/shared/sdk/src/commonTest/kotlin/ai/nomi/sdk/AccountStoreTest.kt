package ai.nomi.sdk

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertNull
import kotlin.time.Duration.Companion.seconds
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.CoroutineStart
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.produceIn
import kotlinx.coroutines.flow.toList
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.withContext
import kotlinx.coroutines.withTimeout

private const val BASE_URL = "https://api-nomi.trian.space"
private const val ACCOUNT = "account-a"
private const val OTHER_ACCOUNT = "account-b"
private const val ALL = 100u
private val TEST_TIMEOUT = 5.seconds

/** The Rust storage engine through the Kotlin API; runs where the host can load the library. */
class AccountStoreTest {
    private fun nativeTest(body: suspend CoroutineScope.(NativeNomiClient) -> Unit) = runTest {
        if (!hostCanLoadSdk) return@runTest
        withContext(Dispatchers.Default) {
            withTimeout(TEST_TIMEOUT) {
                val client = NativeNomiClient(defaultSdkConfig(BASE_URL, newDataDir()), MemoryCredentialStore())
                try {
                    body(client)
                } finally {
                    client.close()
                }
            }
        }
    }

    private fun session(id: String, updatedAt: Long, unread: Boolean = false) =
        Session(id = id, title = "Chat $id", lastMessagePreview = "", updatedAt = updatedAt, unread = unread)

    private fun message(id: String, sessionId: String, createdAt: Long) = Message(
        id = id, sessionId = sessionId, role = "user", agentType = null, agentDisplayName = null,
        content = "text $id", contentJson = null, createdAt = createdAt, status = "sent",
    )

    private val everything = SessionFilter(search = null, unreadOnly = false, limit = ALL)

    @Test
    fun sessionsFlowStartsWithTheSnapshotThenFollowsWrites() = nativeTest { client ->
        val store = client.openAccount(ACCOUNT)
        val lists = store.sessions(everything).produceIn(this)
        // The snapshot is read once the observation is running: write only after it arrives.
        assertEquals(emptyList(), lists.receive())
        store.upsertSessions(listOf(session("a", 1), session("b", 2)))
        assertEquals(listOf("b", "a"), lists.receive().map { it.id })
        lists.cancel()
    }

    @Test
    fun pagesThroughMessages() = nativeTest { client ->
        val store = client.openAccount(ACCOUNT)
        store.upsertMessages((1..5).map { message("m$it", "s", it.toLong()) })

        val newest = store.messagesPage("s", null, 3u)
        assertEquals(listOf("m3", "m4", "m5"), newest.messages.map { it.id })
        val older = store.messagesPage("s", newest.older, 3u)
        assertEquals(listOf("m1", "m2"), older.messages.map { it.id })
        assertNull(older.older)
        assertEquals(listOf("m4", "m5"), store.messages("s", 2u).first().map { it.id })
    }

    @Test
    fun draftFlowTellsNoDraftApartFromTheEnd() = nativeTest { client ->
        val store = client.openAccount(ACCOUNT)
        val drafts = store.draft("s").produceIn(this)
        assertNull(drafts.receive())
        store.appendDraft("s", "turn", "Hello", 1)
        assertEquals("Hello", drafts.receive()?.text)
        drafts.cancel()
    }

    @Test
    fun switchingAccountsClosesThePreviousStoreAndKeepsDataApart() = nativeTest { client ->
        val first = client.openAccount(ACCOUNT)
        first.upsertSessions(listOf(session("a", 1)))
        val firstSessions = async(start = CoroutineStart.UNDISPATCHED) { first.sessions(everything).toList() }

        val second = client.openAccount(OTHER_ACCOUNT)
        assertEquals(OTHER_ACCOUNT, client.account.value?.accountId())
        assertEquals(emptyList(), second.sessions(everything).first())
        assertFailsWith<NomiException.AccountClosed> { first.getSession("a") }
        // The previous store's flow completes (with its snapshot, if it got one before the switch).
        firstSessions.await()

        assertEquals("a", client.openAccount(ACCOUNT).getSession("a")?.id)
    }

    @Test
    fun removingTheAccountDeletesItsData() = nativeTest { client ->
        client.openAccount(ACCOUNT).upsertSessions(listOf(session("a", 1)))
        client.removeAccount(ACCOUNT)
        assertNull(client.account.value)
        assertNull(client.openAccount(ACCOUNT).getSession("a"))
    }

    @Test
    fun reportsTheDatabase() = nativeTest { client ->
        val status = client.openAccount(ACCOUNT).databaseStatus()
        assertEquals(ACCOUNT, status.accountId)
        assertEquals(ACCOUNT, client.diagnostics().accountId)
        assertEquals(true, status.path.endsWith("accounts/$ACCOUNT/nomi.db"))
    }

    @Test
    fun rejectsAnAccountIdThatIsAPath() = nativeTest { client ->
        assertFailsWith<NomiException.InvalidArgument> { client.openAccount("../escape") }
    }
}
