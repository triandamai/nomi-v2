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

    /** Who is signed in; set from the stored session at construction, then by sign-in and sign-out. */
    val user: StateFlow<SignedInUser?>

    /** The realtime socket's state; it runs while the client is started and someone is signed in. */
    val connection: StateFlow<ConnectionState>

    /** The open account's store; null until [openAccount], and after it is closed or removed. */
    val account: StateFlow<AccountStore?>

    fun diagnostics(): Diagnostics

    /** Starts background work. A no-op while running; throws [NomiException.ShutDown] once closed. */
    suspend fun start()

    /**
     * Signs in with email and password. The backend may ask for an emailed code instead
     * ([SignInResult.EmailCodeRequired]); finish with [verifyEmailCode]. Throws
     * [NomiException.Unauthorized] for wrong credentials, [NomiException.Network] when offline.
     */
    suspend fun signIn(email: String, password: String): SignInResult

    /** Throws [NomiException.EmailCode] when the code is wrong, expired or used up. */
    suspend fun verifyEmailCode(challengeId: String, code: String): SignedInUser

    suspend fun resendEmailCode(challengeId: String): VerificationInfo

    /** False when there is no session or the backend refused it (the session has then ended). */
    suspend fun refreshSession(): Boolean

    /**
     * Forgets the session, closes the socket and the open account (its data stays until
     * [removeAccount]), and revokes the session on the backend, best effort.
     */
    suspend fun signOut()

    suspend fun fetchProfile(): Profile

    /** Skips any backoff wait and reconnects the socket now (e.g. back online). */
    fun reconnect()

    /**
     * Events for one realtime topic (`chat:<id>`, `project:<id>`, `user`): [SdkEvent.Realtime],
     * [SdkEvent.TopicResynced] and [SdkEvent.TopicAccessLost]. The topic is watched while collected.
     * Throws [NomiException.InvalidArgument] for a malformed topic.
     */
    fun topicEvents(topic: String): Flow<SdkEvent>

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
