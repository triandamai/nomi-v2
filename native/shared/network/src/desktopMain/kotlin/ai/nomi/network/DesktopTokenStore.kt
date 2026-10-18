package ai.nomi.network

import com.github.javakeyring.Keyring
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext
import kotlinx.serialization.json.Json

private const val KEYRING_ACCOUNT = "session"

/** OS keyring (Keychain / Credential Manager / libsecret) with a 0600 file fallback; never logs secrets. */
class DesktopTokenStore(dataDir: java.io.File = desktopDataDir()) : TokenStore {
    private val fallback = FileTokenStore(dataDir)
    private val lock = Mutex()
    private var keyring: Keyring? = null
    private var keyringTried = false

    private fun keyringOrNull(): Keyring? {
        if (!keyringTried) {
            keyringTried = true
            keyring = runCatching { Keyring.create() }.getOrElse {
                System.err.println("Nomi: no OS keyring (${it::class.simpleName}); storing tokens in a private file")
                null
            }
        }
        return keyring
    }

    private suspend fun <T> viaKeyring(block: (Keyring) -> T): Result<T>? = withContext(Dispatchers.IO) {
        val ring = keyringOrNull() ?: return@withContext null
        runCatching { block(ring) }.also { result ->
            result.exceptionOrNull()?.let {
                System.err.println("Nomi: keyring unavailable (${it::class.simpleName}); using private file")
                keyring = null
            }
        }.takeIf { it.isSuccess }
    }

    override suspend fun read(): Tokens? = lock.withLock {
        val stored = viaKeyring { it.getPassword(TOKEN_SERVICE, KEYRING_ACCOUNT) }
        if (stored != null) runCatching { Json.decodeFromString(Tokens.serializer(), stored.getOrThrow()) }.getOrNull()
        else fallback.read()
    }

    override suspend fun write(tokens: Tokens) = lock.withLock {
        val json = Json.encodeToString(Tokens.serializer(), tokens)
        if (viaKeyring { it.setPassword(TOKEN_SERVICE, KEYRING_ACCOUNT, json) } == null) fallback.write(tokens)
    }

    override suspend fun clear() = lock.withLock {
        viaKeyring { it.deletePassword(TOKEN_SERVICE, KEYRING_ACCOUNT) }
        fallback.clear()
    }
}
