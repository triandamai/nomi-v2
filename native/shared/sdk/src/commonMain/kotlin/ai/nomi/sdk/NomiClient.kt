package ai.nomi.sdk

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.StateFlow

/**
 * The app's handle on the Rust engine: one per process, owned by the app, never by a screen.
 *
 * The models ([SdkConfig], [SdkEvent], [SdkState], [Diagnostics], [NomiException]) are the SDK's
 * public types, generated from `rust/crates/nomi-sdk`.
 */
interface NomiClient {
    /**
     * Every [SdkEvent] sent while collecting. Each collector gets its own stream, which completes
     * once the client is closed.
     */
    val events: Flow<SdkEvent>

    /** Where the engine is in its lifecycle; updated as [start] and [close] return. */
    val state: StateFlow<SdkState>

    fun diagnostics(): Diagnostics

    /** Starts background work. A no-op while running; throws [NomiException.ShutDown] once closed. */
    suspend fun start()

    /** Stops background work for good and completes every [events] stream. A no-op once closed. */
    suspend fun close()
}
