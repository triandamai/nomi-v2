package ai.nomi.realtime

import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.TestScope
import kotlinx.coroutines.test.advanceTimeBy
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import kotlinx.serialization.json.jsonObject
import kotlin.random.Random
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertIs
import kotlin.test.assertTrue

/*
 * The hub runs on `backgroundScope` (its ping loop never ends, so a foreground scope would never go idle).
 * `advanceUntilIdle()` deliberately ignores background work in kotlinx-coroutines-test, so these tests settle
 * the hub with `runCurrent()` (runs everything due now, background included) and move time with `advanceTimeBy`.
 */
@OptIn(ExperimentalCoroutinesApi::class)
class RealtimeHubTest {
    private val chat = Topic.Chat("s1")

    private fun eventFrame(topic: String, json: String) =
        ServerFrame.Event(topic, WireJson.parseToJsonElement(json).jsonObject)

    private fun TestScope.connectedHub(backoff: Backoff = Backoff()): Pair<DefaultRealtimeHub, FakeTransport> {
        val t = FakeTransport()
        val hub = DefaultRealtimeHub(t, refreshToken = { true }, scope = backgroundScope, backoff = backoff)
        hub.start()
        t.push(ServerFrame.Ready("u1"))
        runCurrent()
        return hub to t
    }

    @Test fun subscribes_once_for_two_listeners() = runTest {
        val (hub, t) = connectedHub()
        hub.addListener(chat) {}; hub.addListener(chat) {}
        runCurrent()
        assertEquals(listOf(ClientFrame.Subscribe("chat:s1")), t.sent.filterIsInstance<ClientFrame.Subscribe>())
    }

    @Test fun unsubscribe_only_when_last_listener_removed() = runTest {
        val (hub, t) = connectedHub()
        val a = hub.addListener(chat) {}; val b = hub.addListener(chat) {}
        runCurrent(); a.remove(); runCurrent()
        assertTrue(t.sent.none { it is ClientFrame.Unsubscribe })
        b.remove(); runCurrent()
        assertEquals(ClientFrame.Unsubscribe("chat:s1"), t.sent.last())
    }

    @Test fun throwing_listener_does_not_break_others() = runTest {
        val (hub, t) = connectedHub()
        val got = mutableListOf<RealtimeEvent>()
        hub.addListener(chat) { error("boom") }
        hub.addListener(chat) { got += it }
        runCurrent()
        t.push(eventFrame("chat:s1", """{"kind":"SessionRenamed","title":"x"}"""))
        t.push(eventFrame("chat:s1", """{"kind":"SessionRenamed","title":"y"}"""))
        runCurrent()
        assertEquals(listOf("x", "y"), got.map { (it as RealtimeEvent.SessionRenamed).title })
    }

    @Test fun reconnect_resubscribes_and_emits_resynced() = runTest {
        val (hub, t) = connectedHub()
        val got = mutableListOf<RealtimeEvent>()
        hub.addListener(chat) { got += it }
        runCurrent(); t.sent.clear()
        t.dropConnection(); advanceTimeBy(Backoff.MAX_DELAY_MS + 1); t.acceptNext(userId = "u1"); runCurrent()
        assertEquals(listOf(ClientFrame.Subscribe("chat:s1")), t.sent.filterIsInstance<ClientFrame.Subscribe>())
        assertTrue(got.any { it is RealtimeEvent.Resynced })
    }

    @Test fun unauthorized_upgrade_refreshes_token_once_then_stops() = runTest {
        var refreshes = 0
        val t = FakeTransport(failWith401Times = Int.MAX_VALUE)
        val hub = DefaultRealtimeHub(t, refreshToken = { refreshes++; true }, scope = backgroundScope)
        hub.start(); runCurrent()
        assertEquals(1, refreshes)
        assertEquals(ConnectionState.Idle, hub.state.value)
    }

    @Test fun stale_socket_is_recycled() = runTest {
        val (hub, t) = connectedHub()
        advanceTimeBy(DefaultRealtimeHub.STALE_AFTER_MS + 1); runCurrent()
        assertTrue(t.closedCount >= 1)
        assertTrue(hub.state.value is ConnectionState.Waiting || hub.state.value is ConnectionState.Connecting)
    }

    @Test fun flow_adapter_removes_listener_on_cancel() = runTest {
        val (hub, t) = connectedHub()
        val job = launch { hub.events(chat).collect {} }
        runCurrent(); job.cancel(); runCurrent()
        assertEquals(ClientFrame.Unsubscribe("chat:s1"), t.sent.last())
    }

    @Test fun denied_emits_access_lost() = runTest {
        val (hub, t) = connectedHub()
        val got = mutableListOf<RealtimeEvent>()
        hub.addListener(chat) { got += it }; runCurrent()
        t.push(ServerFrame.Denied("chat:s1")); runCurrent()
        assertEquals(RealtimeEvent.AccessLost(chat), got.single())
    }

    // ---- Beyond the brief: the remaining behaviour lines ----

    @Test fun user_topic_is_never_subscribed_or_unsubscribed() = runTest {
        val (hub, t) = connectedHub()
        val h = hub.addListener(Topic.User) {}
        runCurrent(); h.remove(); runCurrent()
        assertTrue(t.sent.none { it is ClientFrame.Subscribe || it is ClientFrame.Unsubscribe })
    }

    @Test fun events_reach_only_their_topic_in_order() = runTest {
        val (hub, t) = connectedHub()
        val chatGot = mutableListOf<RealtimeEvent>()
        val userGot = mutableListOf<RealtimeEvent>()
        hub.addListener(chat) { chatGot += it }
        hub.addListener(Topic.User) { userGot += it }
        runCurrent()
        t.push(eventFrame("chat:s1", """{"kind":"MessageCreated","message_id":"m1"}"""))
        t.push(eventFrame("chat:other", """{"kind":"MessageCreated","message_id":"m2"}"""))
        t.push(eventFrame("chat:s1", """{"kind":"MessageCreated","message_id":"m3"}"""))
        runCurrent()
        assertEquals(listOf("m1", "m3"), chatGot.map { (it as RealtimeEvent.MessageCreated).messageId })
        assertTrue(userGot.isEmpty())
    }

    @Test fun connected_state_carries_user_id() = runTest {
        val (hub, _) = connectedHub()
        assertEquals(ConnectionState.Connected("u1"), hub.state.value)
    }

    @Test fun reconnect_emits_resynced_to_user_topic_too() = runTest {
        val (hub, t) = connectedHub()
        val got = mutableListOf<RealtimeEvent>()
        hub.addListener(Topic.User) { got += it }
        runCurrent()
        t.dropConnection(); advanceTimeBy(Backoff.MAX_DELAY_MS + 1); t.acceptNext("u1"); runCurrent()
        assertEquals(listOf<RealtimeEvent>(RealtimeEvent.Resynced(Topic.User)), got)
    }

    @Test fun socket_close_moves_to_waiting() = runTest {
        val (hub, t) = connectedHub()
        t.dropConnection(); runCurrent()
        val waiting = assertIs<ConnectionState.Waiting>(hub.state.value)
        assertEquals(1, waiting.attempt)
        assertTrue(waiting.retryInMs in 0..Backoff.BASE_DELAY_MS)
    }

    @Test fun denied_topic_is_not_resubscribed_until_a_listener_is_added_again() = runTest {
        val (hub, t) = connectedHub()
        hub.addListener(chat) {}; runCurrent()
        t.push(ServerFrame.Denied("chat:s1")); runCurrent()
        t.sent.clear()
        t.dropConnection(); advanceTimeBy(Backoff.MAX_DELAY_MS + 1); t.acceptNext("u1"); runCurrent()
        assertTrue(t.sent.none { it is ClientFrame.Subscribe })
        hub.addListener(chat) {}; runCurrent()
        assertEquals(listOf(ClientFrame.Subscribe("chat:s1")), t.sent.filterIsInstance<ClientFrame.Subscribe>())
    }

    @Test fun refresh_rejected_goes_idle_without_retrying() = runTest {
        val t = FakeTransport(failWith401Times = 1)
        val hub = DefaultRealtimeHub(t, refreshToken = { false }, scope = backgroundScope)
        hub.start(); runCurrent()
        assertEquals(ConnectionState.Idle, hub.state.value)
        advanceTimeBy(Backoff.MAX_DELAY_MS + 1)
        assertEquals(1, t.connectCount)
    }

    @Test fun refresh_accepted_retries_immediately() = runTest {
        val t = FakeTransport(failWith401Times = 1)
        val hub = DefaultRealtimeHub(t, refreshToken = { true }, scope = backgroundScope)
        hub.start(); t.acceptNext("u1"); runCurrent()
        assertEquals(2, t.connectCount)
        assertEquals(ConnectionState.Connected("u1"), hub.state.value)
    }

    @Test fun client_pings_on_interval() = runTest {
        val (_, t) = connectedHub()
        advanceTimeBy(DefaultRealtimeHub.PING_INTERVAL_MS + 1)
        assertEquals(listOf<ClientFrame>(ClientFrame.Ping), t.sent)
    }

    @Test fun frames_keep_a_socket_fresh() = runTest {
        val (hub, t) = connectedHub()
        advanceTimeBy(DefaultRealtimeHub.STALE_AFTER_MS - 1); t.push(ServerFrame.Pong); runCurrent()
        advanceTimeBy(DefaultRealtimeHub.STALE_AFTER_MS - 1)
        assertEquals(0, t.closedCount)
        assertEquals(ConnectionState.Connected("u1"), hub.state.value)
    }

    @Test fun backoff_grows_and_resets_after_a_stable_connection() = runTest {
        val (hub, t) = connectedHub()
        t.dropConnection(); runCurrent()
        assertEquals(1, assertIs<ConnectionState.Waiting>(hub.state.value).attempt)
        advanceTimeBy(Backoff.MAX_DELAY_MS + 1); t.acceptNext("u1"); runCurrent()
        t.dropConnection(); runCurrent()
        assertEquals(2, assertIs<ConnectionState.Waiting>(hub.state.value).attempt)
        advanceTimeBy(Backoff.MAX_DELAY_MS + 1); t.acceptNext("u1"); runCurrent()
        advanceTimeBy(DefaultRealtimeHub.STABLE_AFTER_MS + 1)
        t.dropConnection(); runCurrent()
        assertEquals(1, assertIs<ConnectionState.Waiting>(hub.state.value).attempt)
    }

    @Test fun stop_closes_socket_and_goes_idle() = runTest {
        val (hub, t) = connectedHub()
        hub.stop(); runCurrent()
        assertEquals(ConnectionState.Idle, hub.state.value)
        assertEquals(1, t.closedCount)
        advanceTimeBy(Backoff.MAX_DELAY_MS + 1)
        assertEquals(1, t.connectCount)
    }

    @Test fun start_is_idempotent() = runTest {
        val (hub, t) = connectedHub()
        hub.start(); runCurrent()
        assertEquals(1, t.connectCount)
    }

    @Test fun listeners_survive_stop_and_resubscribe_on_next_start() = runTest {
        val (hub, t) = connectedHub()
        val got = mutableListOf<RealtimeEvent>()
        hub.addListener(chat) { got += it }; runCurrent()
        hub.stop(); runCurrent(); t.sent.clear()
        hub.start(); t.acceptNext("u1"); runCurrent()
        assertEquals(listOf(ClientFrame.Subscribe("chat:s1")), t.sent.filterIsInstance<ClientFrame.Subscribe>())
        assertEquals(listOf<RealtimeEvent>(RealtimeEvent.Resynced(chat)), got)
    }

    @Test fun listener_removed_before_the_hub_ran_never_subscribes() = runTest {
        val (hub, t) = connectedHub()
        hub.addListener(chat) {}.remove(); runCurrent()
        assertTrue(t.sent.isEmpty())
    }
}

class BackoffTest {
    @Test fun full_jitter_stays_under_the_exponential_ceiling() {
        val backoff = Backoff(Random(SEED))
        repeat(SAMPLES) {
            assertTrue(backoff.delayFor(0) in 0..Backoff.BASE_DELAY_MS)
            assertTrue(backoff.delayFor(3) in 0..Backoff.BASE_DELAY_MS * 8)
            assertTrue(backoff.delayFor(40) in 0..Backoff.MAX_DELAY_MS)
        }
    }

    @Test fun injected_random_makes_delays_deterministic() {
        val a = Backoff(Random(SEED)); val b = Backoff(Random(SEED))
        assertEquals((0 until SAMPLES).map { a.delayFor(it) }, (0 until SAMPLES).map { b.delayFor(it) })
    }

    @Test fun ceiling_reaches_the_cap() {
        val alwaysMax = object : Random() {
            override fun nextBits(bitCount: Int): Int = 0
            override fun nextLong(from: Long, until: Long): Long = until - 1
        }
        val backoff = Backoff(alwaysMax)
        assertEquals(Backoff.BASE_DELAY_MS, backoff.delayFor(0))
        assertEquals(Backoff.BASE_DELAY_MS * 4, backoff.delayFor(2))
        assertEquals(Backoff.MAX_DELAY_MS, backoff.delayFor(10))
        assertEquals(Backoff.MAX_DELAY_MS, backoff.delayFor(Int.MAX_VALUE))
    }

    private companion object {
        const val SEED = 42
        const val SAMPLES = 50
    }
}
