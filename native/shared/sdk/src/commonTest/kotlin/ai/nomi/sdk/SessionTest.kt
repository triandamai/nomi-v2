package ai.nomi.sdk

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertNull
import kotlin.time.Duration.Companion.seconds
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.withContext
import kotlinx.coroutines.withTimeout

private const val USER_ID = "6f1c2b9e-user"
private val TEST_TIMEOUT = 5.seconds

/**
 * Session and realtime through the Kotlin API, against a backend that can't be reached: what can
 * be checked without a server (the credential-store callback, errors, the socket's retry state).
 * The protocol itself is covered by the Rust tests against a fake backend.
 */
class SessionTest {
    private fun nativeTest(
        stored: Credentials? = null,
        body: suspend CoroutineScope.(NativeNomiClient, MemoryCredentialStore) -> Unit,
    ) = runTest {
        if (!hostCanLoadSdk) return@runTest
        withContext(Dispatchers.Default) {
            withTimeout(TEST_TIMEOUT) {
                val store = MemoryCredentialStore(stored)
                val client = NativeNomiClient(defaultSdkConfig(UNREACHABLE_BASE_URL, newDataDir()), store)
                try {
                    body(client, store)
                } finally {
                    client.close()
                }
            }
        }
    }

    private val session = Credentials(accessToken = accessToken(USER_ID), refreshToken = "refresh")

    @Test
    fun readsTheStoredSessionThroughTheCallback() = nativeTest(stored = session) { client, _ ->
        assertEquals(USER_ID, client.user.value?.userId)
    }

    @Test
    fun startsSignedOutWithoutAStoredSession() = nativeTest { client, _ ->
        assertNull(client.user.value)
        assertFailsWith<NomiException.NotSignedIn> { client.fetchProfile() }
        assertEquals(false, client.refreshSession())
    }

    @Test
    fun anUnreachableBackendIsANetworkError() = nativeTest { client, store ->
        assertFailsWith<NomiException.Network> { client.signIn("a@nomi.test", "pw") }
        assertNull(store.stored)
    }

    @Test
    fun signOutForgetsTheSessionEvenOffline() = nativeTest(stored = session) { client, store ->
        client.signOut()
        assertNull(store.stored)
        assertEquals(1, store.clears)
        assertNull(client.user.value)
    }

    @Test
    fun theSocketRetriesWhileTheBackendIsDown() = nativeTest(stored = session) { client, _ ->
        assertEquals(ConnectionState.Idle, client.connection.value)
        client.start()
        val waiting = client.connection.first { it is ConnectionState.Waiting }
        assertEquals(1u, (waiting as ConnectionState.Waiting).attempt)
        client.close()
        assertEquals(ConnectionState.Idle, client.connection.value)
    }

    @Test
    fun rejectsAMalformedTopic() = nativeTest { client, _ ->
        assertEquals("chat:abc", realtimeTopic("chat:ABC"))
        assertFailsWith<NomiException.InvalidArgument> { client.topicEvents("session:1") }
    }
}
