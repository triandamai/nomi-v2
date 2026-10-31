package ai.nomi.data

import ai.nomi.network.TokenStore
import ai.nomi.network.Tokens
import ai.nomi.sdk.AccountStore
import ai.nomi.sdk.CredentialStore
import ai.nomi.sdk.Credentials
import ai.nomi.sdk.NomiClient
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.emitAll
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext
import kotlin.concurrent.Volatile

/** Where the Rust engine talks to and keeps its data; each platform module provides it. */
data class SdkSetup(val baseUrl: String, val dataDir: String)

/**
 * The app's one Rust engine ([NomiClient]). It is created on first use, off the main thread: creating it reads the
 * stored session through the platform's secure store. Once created it follows [foreground], so a reply in the chat on
 * screen isn't marked unread.
 */
class NomiEngine(
    private val scope: CoroutineScope,
    private val foreground: ForegroundChat,
    private val create: () -> NomiClient,
) {
    private val lock = Mutex()

    @Volatile
    private var created: NomiClient? = null

    suspend fun get(): NomiClient = created ?: lock.withLock {
        created ?: withContext(Dispatchers.Default) { create() }.also { client ->
            created = client
            scope.launch { foreground.sessionId.collect { client.setForegroundChat(it) } }
        }
    }

    /**
     * [block]'s flow over the open account's store, restarted whenever the account changes; [empty] while none is
     * open (signed out, or not yet opened).
     */
    @OptIn(ExperimentalCoroutinesApi::class)
    fun <T> fromAccount(empty: T, block: (AccountStore) -> Flow<T>): Flow<T> = flow {
        emitAll(get().account.flatMapLatest { store -> store?.let(block) ?: flowOf(empty) })
    }

    /** The open account's store, or [SessionEndedException] when none is open. */
    suspend fun account(): AccountStore = get().account.value ?: throw SessionEndedException()
}

/**
 * The engine's [CredentialStore] over the app's [TokenStore] (Keystore, Keychain, the desktop store), where the
 * session has always been kept. The engine calls it off the main thread.
 *
 * A store that can't be read (locked keychain, corrupt entry) counts as signed out, as it always has; a failed write
 * is logged and leaves the session in memory only. Nothing is thrown back into the engine.
 */
class TokenStoreCredentials(
    private val tokens: TokenStore,
    private val log: (String) -> Unit = {},
) : CredentialStore {
    override fun load(): Credentials? = attempt("read", fallback = null) {
        tokens.read()?.let { Credentials(it.access, it.refresh) }
    }

    override fun save(credentials: Credentials) = attempt("write", fallback = Unit) {
        tokens.write(Tokens(credentials.accessToken, credentials.refreshToken))
    }

    override fun clear() = attempt("clear", fallback = Unit) { tokens.clear() }

    private fun <T> attempt(what: String, fallback: T, block: suspend () -> T): T = try {
        runBlocking { block() }
    } catch (e: Exception) {
        log("data: the token store failed to $what: $e")
        fallback
    }
}
