package ai.nomi.sdk

import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertNotNull
import kotlin.test.assertTrue
import kotlin.time.Duration.Companion.seconds
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.CoroutineStart
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.filterNot
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.toList
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.withContext
import kotlinx.coroutines.withTimeout

private const val BASE_URL = "https://api-nomi.trian.space"
private val EVENT_TIMEOUT = 5.seconds

/**
 * Drives the real Rust library,, so it only runs where the host can load it (see [hostCanLoadSdk]).
 * Collectors start `UNDISPATCHED`: they subscribe before the test moves on, so no event is missed.
 */
class NativeNomiClientTest {
    private fun client(): NativeNomiClient = NativeNomiClient(defaultSdkConfig(BASE_URL, newDataDir()), MemoryCredentialStore())

    /** Native calls resume from Rust threads; real time, not runTest's virtual clock, bounds them. */
    private fun nativeTest(body: suspend CoroutineScope.() -> Unit) = runTest {
        if (hostCanLoadSdk) withContext(Dispatchers.Default) { withTimeout(EVENT_TIMEOUT) { body() } }
    }

    @Test
    fun walksThroughTheLifecycle() = nativeTest {
        val client = client()
        assertEquals(SdkState.INITIALIZED, client.state.value)
        assertEquals(sdkVersion(), client.diagnostics().sdkVersion)

        client.start()
        assertEquals(SdkState.RUNNING, client.state.value)
        assertNotNull(client.diagnostics().startedAtMs)

        client.close()
        assertEquals(SdkState.SHUT_DOWN, client.state.value)
    }

    @Test
    fun eventsCarryStateChangesAndCompleteOnClose() = nativeTest {
        val client = client()
        val lifecycle = async(start = CoroutineStart.UNDISPATCHED) {
            client.events.filterNot { it is SdkEvent.Log }.toList()
        }

        client.start()
        client.close()

        assertEquals(
            listOf(SdkEvent.StateChanged(SdkState.RUNNING), SdkEvent.StateChanged(SdkState.SHUT_DOWN)),
            lifecycle.await(),
        )
    }

    @Test
    fun forwardsTheEnginesLogs() = nativeTest {
        val client = client()
        val started = async(start = CoroutineStart.UNDISPATCHED) {
            client.events.filter { it is SdkEvent.Log && it.record.message == "started" }.first()
        }
        client.start()
        val record = (started.await() as SdkEvent.Log).record
        assertEquals(LogLevel.INFO, record.level)
        assertTrue(record.target.startsWith("nomi_sdk"))
        client.close()
    }

    @Test
    fun cancellingACollectorReleasesItsStream() = nativeTest {
        val client = client()
        client.start()
        val collector = async(start = CoroutineStart.UNDISPATCHED) { client.events.toList() }
        collector.cancel()
        collector.join()
        client.close()
    }

    @Test
    fun startAfterCloseFails() = nativeTest {
        val client = client()
        client.close()
        assertFailsWith<NomiException.ShutDown> { client.start() }
        assertEquals(SdkState.SHUT_DOWN, client.state.value)
    }

    @Test
    fun rejectsAnInvalidConfig() = nativeTest {
        val error = assertFailsWith<NomiException.InvalidConfig> {
            NativeNomiClient(defaultSdkConfig("not a url", newDataDir()), MemoryCredentialStore())
        }
        assertTrue(error.reason.contains("base_url"))
    }
}
