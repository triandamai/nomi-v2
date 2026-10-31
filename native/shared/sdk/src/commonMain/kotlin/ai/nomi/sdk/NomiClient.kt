package ai.nomi.sdk

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.StateFlow

/**
 * The app's handle on the Rust engine: one per process, owned by the app, never by a screen.
 *
 * The models ([SdkConfig], [SdkEvent], [SdkState], [Diagnostics], [NomiException], [AccountStore]
 * and the stored models) are the SDK's public types, generated from `rust/crates/nomi-sdk`.
 */
interface NomiClient {
    /**
     * Every [SdkEvent] sent while collecting. Each collector gets its own stream, which completes
     * once the client is closed.
     */
    val events: Flow<SdkEvent>

    /** Where the engine is in its lifecycle; updated as [start] and [close] return. */
    val state: StateFlow<SdkState>

    /** The open account's store; null until [openAccount], and after it is closed or removed. */
    val account: StateFlow<AccountStore?>

    fun diagnostics(): Diagnostics

    /** Starts background work. A no-op while running; throws [NomiException.ShutDown] once closed. */
    suspend fun start()

    /**
     * The local store for [accountId] (the signed-in user's id), opening and migrating its database
     * if needed. Opening another account closes the current one: its store throws
     * [NomiException.AccountClosed] and its flows complete.
     */
    suspend fun openAccount(accountId: String): AccountStore

    /** Closes the open account, keeping its data. */
    suspend fun closeAccount()

    /** Deletes everything stored for [accountId] (sign-out), closing it first if it is open. */
    suspend fun removeAccount(accountId: String)

    /** Deletes every account's local data. */
    suspend fun resetStorage()

    /** Stops background work for good and completes every [events] stream. A no-op once closed. */
    suspend fun close()
}
