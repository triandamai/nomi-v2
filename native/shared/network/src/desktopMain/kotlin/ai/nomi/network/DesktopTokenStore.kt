package ai.nomi.network

import com.github.javakeyring.Keyring
import com.github.javakeyring.PasswordAccessException
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext
import kotlinx.serialization.json.Json
import java.io.File

private const val KEYRING_ACCOUNT = "session"
private const val PROBE_ACCOUNT = "availability-probe"
private const val PROBE_VALUE = "ok"

/**
 * A secret slot in the OS keyring. [get] returns null when nothing is stored; any exception from any call means
 * the backend itself failed (not "not found").
 */
internal interface SecretBackend {
    fun get(service: String, account: String): String?
    fun set(service: String, account: String, secret: String)
    fun delete(service: String, account: String)
}

/** java-keyring reports a missing entry as [PasswordAccessException], same as a real failure, so map it here. */
private class JavaKeyringBackend(private val keyring: Keyring) : SecretBackend {
    override fun get(service: String, account: String): String? =
        try { keyring.getPassword(service, account) } catch (_: PasswordAccessException) { null }

    override fun set(service: String, account: String, secret: String) = keyring.setPassword(service, account, secret)

    override fun delete(service: String, account: String) {
        try { keyring.deletePassword(service, account) } catch (_: PasswordAccessException) { /* already gone */ }
    }
}

/** Decides keyring availability once: it must be creatable and survive a write/read/delete round trip. */
private fun openJavaKeyring(): SecretBackend? = try {
    val backend = JavaKeyringBackend(Keyring.create())
    backend.set(TOKEN_SERVICE, PROBE_ACCOUNT, PROBE_VALUE)
    check(backend.get(TOKEN_SERVICE, PROBE_ACCOUNT) == PROBE_VALUE)
    backend.delete(TOKEN_SERVICE, PROBE_ACCOUNT)
    backend
} catch (e: Exception) {
    System.err.println("Nomi: no usable OS keyring (${e::class.simpleName}); storing tokens in a private file")
    null
}

/** OS keyring (Keychain / Credential Manager / libsecret) with a 0600 file fallback; never logs secrets. */
class DesktopTokenStore internal constructor(
    dataDir: File,
    private val backendProvider: () -> SecretBackend?,
) : TokenStore {
    constructor(dataDir: File = desktopDataDir()) : this(dataDir, ::openJavaKeyring)

    private val fallback = FileTokenStore(dataDir)
    private val lock = Mutex()
    private val backend: SecretBackend? by lazy(backendProvider)

    override suspend fun read(): Tokens? = lock.withLock {
        withContext(Dispatchers.IO) {
            val stored = backend?.let { b ->
                try { b.get(TOKEN_SERVICE, KEYRING_ACCOUNT) } catch (e: Exception) { warn(e); null }
            }
            stored?.let { runCatching { Json.decodeFromString(Tokens.serializer(), it) }.getOrNull() }
        } ?: fallback.read()
    }

    override suspend fun write(tokens: Tokens) = lock.withLock {
        val saved = withContext(Dispatchers.IO) {
            val b = backend ?: return@withContext false
            try {
                b.set(TOKEN_SERVICE, KEYRING_ACCOUNT, Json.encodeToString(Tokens.serializer(), tokens))
                true
            } catch (e: Exception) {
                warn(e)
                // Never leave an older session behind for the next launch to read first.
                runCatching { b.delete(TOKEN_SERVICE, KEYRING_ACCOUNT) }
                false
            }
        }
        if (saved) fallback.clear() else fallback.write(tokens)
    }

    override suspend fun clear() = lock.withLock {
        withContext(Dispatchers.IO) {
            backend?.let { b -> try { b.delete(TOKEN_SERVICE, KEYRING_ACCOUNT) } catch (e: Exception) { warn(e) } }
        }
        fallback.clear()
    }

    private fun warn(e: Exception) = System.err.println("Nomi: keyring call failed (${e::class.simpleName}); using private file")
}
