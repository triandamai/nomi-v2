package ai.nomi.network

import kotlinx.coroutines.test.runTest
import java.nio.file.Files
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertTrue

private class FakeBackend : SecretBackend {
    val data = mutableMapOf<String, String>()
    var failing = false
    override fun get(service: String, account: String): String? {
        if (failing) error("backend down")
        return data[account]
    }
    override fun set(service: String, account: String, secret: String) {
        if (failing) error("backend down")
        data[account] = secret
    }
    override fun delete(service: String, account: String) {
        if (failing) error("backend down")
        data.remove(account)
    }
}

class DesktopTokenStoreTest {
    private val tokens = Tokens("a", "r")

    @Test
    fun notFoundOnFirstReadKeepsTheKeyringInUse() = runTest {
        val dir = Files.createTempDirectory("nomi-desktop").toFile()
        val backend = FakeBackend()
        val store = DesktopTokenStore(dir) { backend }
        assertNull(store.read())
        store.write(tokens)
        assertEquals(1, backend.data.size)
        assertFalse(FileTokenStore(dir).file.exists())
        assertEquals(tokens, store.read())
        store.clear()
        assertNull(store.read())
        dir.deleteRecursively()
    }

    @Test
    fun backendFailureFallsBackToFileAndDropsStaleKeyringEntry() = runTest {
        val dir = Files.createTempDirectory("nomi-desktop").toFile()
        val backend = FakeBackend()
        val store = DesktopTokenStore(dir) { backend }
        store.write(Tokens("old", "old"))
        backend.failing = true
        store.write(tokens)
        assertTrue(FileTokenStore(dir).file.exists())
        assertEquals(tokens, store.read())
        backend.failing = false
        // The stale entry could not be deleted while the backend was down; the next successful write replaces it.
        store.write(Tokens("new", "new"))
        assertFalse(FileTokenStore(dir).file.exists())
        assertEquals(Tokens("new", "new"), DesktopTokenStore(dir) { backend }.read())
        dir.deleteRecursively()
    }

    @Test
    fun noBackendUsesTheFile() = runTest {
        val dir = Files.createTempDirectory("nomi-desktop").toFile()
        val store = DesktopTokenStore(dir) { null }
        store.write(tokens)
        assertEquals(tokens, store.read())
        dir.deleteRecursively()
    }
}
