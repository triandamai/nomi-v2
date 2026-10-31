package ai.nomi.sdk

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertIs
import kotlin.test.assertNotNull
import kotlin.test.assertTrue
import kotlin.time.Duration.Companion.seconds
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.CoroutineStart
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.withContext
import kotlinx.coroutines.withTimeout

private const val USER_ID = "4d2a9c1b-user"
private const val CHAT = "c1"
private const val PAGE = 50u
private val TEST_TIMEOUT = 5.seconds

/**
 * Sync through the Kotlin API against a backend that can't be reached: what the device does on
 * its own (the outbox, local writes, typed events, errors). Delivery, retries and realtime are
 * covered by the Rust tests against a fake backend.
 */
class SyncTest {
    private fun nativeTest(
        signedIn: Boolean = true,
        body: suspend CoroutineScope.(NativeNomiClient, AccountStore) -> Unit,
    ) = runTest {
        if (!hostCanLoadSdk) return@runTest
        withContext(Dispatchers.Default) {
            withTimeout(TEST_TIMEOUT) {
                val stored = Credentials(accessToken = accessToken(USER_ID), refreshToken = "refresh").takeIf { signedIn }
                val client = NativeNomiClient(defaultSdkConfig(UNREACHABLE_BASE_URL, newDataDir()), MemoryCredentialStore(stored))
                try {
                    body(client, client.openAccount(USER_ID))
                } finally {
                    client.close()
                }
            }
        }
    }

    @Test
    fun aMessageShowsAtOnceAndWaitsInTheOutbox() = nativeTest { client, store ->
        val localId = client.sendMessage(CHAT, "Hello")

        val shown = store.messages(CHAT, PAGE).first { it.isNotEmpty() }.single()
        assertEquals(localId, shown.id)
        assertEquals("sending", shown.status)
        val entry = store.outbox().first { it.isNotEmpty() }.single()
        assertEquals(OutboxState.PENDING, entry.state)
        assertEquals(OutboxOperation.SendMessage(CHAT, "Hello", localId), entry.operation)
        assertEquals("Hello", store.sessions(SessionFilter(null, false, PAGE)).first().single().lastMessagePreview)

        assertTrue(client.discardOutbox(entry.key))
        store.outbox().first { it.isEmpty() }
        store.messages(CHAT, PAGE).first { it.isEmpty() }
    }

    @Test
    fun rejectsTextTheBackendWouldRefuse() = nativeTest { client, store ->
        assertFailsWith<NomiException.InvalidArgument> { client.sendMessage(CHAT, "   ") }
        assertTrue(store.outboxEntries().isEmpty())
    }

    @Test
    fun markingReadIsLocalAtOnce() = nativeTest(signedIn = false) { client, store ->
        store.upsertNotifications(
            listOf(NotificationItem("n1", "deploy", "T", "B", null, createdAt = 1, readAt = null)),
        )
        client.markNotificationRead("n1")
        assertEquals(0u, store.unreadNotificationCount().first())
        client.markAllNotificationsRead()
        // The single read is folded into the read-all.
        assertEquals(
            listOf<OutboxOperation>(OutboxOperation.MarkAllNotificationsRead),
            store.outboxEntries().map { it.operation },
        )
    }

    @Test
    fun syncingAChatEndsWhenTheAccountCloses() = nativeTest { client, _ ->
        val syncing = launch(start = CoroutineStart.UNDISPATCHED) { client.syncChat(CHAT).collect {} }
        client.closeAccount()
        syncing.join()
        assertFailsWith<NomiException.AccountClosed> { client.refreshSessions() }
        assertFailsWith<NomiException.AccountClosed> { client.syncChat(CHAT).collect {} }
    }

    @Test
    fun fetchesNeedTheBackend() = nativeTest { client, _ ->
        assertFailsWith<NomiException.Network> { client.refreshSessions() }
        assertFailsWith<NomiException.Network> { client.loadOlderMessages(CHAT) }
        assertTrue(client.hasOlderMessages(CHAT))
    }

    @Test
    fun decodesRealtimeEventsTyped() {
        if (!hostCanLoadSdk) return
        val event = decodeRealtimeEvent("chat:$CHAT", """{"kind":"TurnFailed","turn_job_id":"t1","error":"boom"}""")
        assertIs<RealtimeEvent.TurnFailed>(event)
        assertEquals(CHAT, event.sessionId)
        assertEquals("boom", event.error)
        val unknown = decodeRealtimeEvent("user", """{"kind":"Novel"}""")
        assertIs<RealtimeEvent.Unknown>(unknown)
        assertNotNull(unknown.kind)
    }
}
