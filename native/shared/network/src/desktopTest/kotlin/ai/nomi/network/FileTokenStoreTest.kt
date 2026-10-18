package ai.nomi.network

import kotlinx.coroutines.test.runTest
import java.nio.file.Files
import java.nio.file.attribute.PosixFilePermission
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class FileTokenStoreTest {
    @Test
    fun roundTripsWithOwnerOnlyPermissionsAndClears() = runTest {
        val dir = Files.createTempDirectory("nomi-tokens").toFile()
        val store = FileTokenStore(dir)
        assertNull(store.read())

        store.write(Tokens("a", "r"))
        assertEquals(Tokens("a", "r"), FileTokenStore(dir).read())

        val perms = Files.getPosixFilePermissions(store.file.toPath())
        assertEquals(setOf(PosixFilePermission.OWNER_READ, PosixFilePermission.OWNER_WRITE), perms)

        store.clear()
        assertNull(store.read())
        dir.deleteRecursively()
    }

    @Test
    fun corruptFileReadsAsSignedOut() = runTest {
        val dir = Files.createTempDirectory("nomi-tokens").toFile()
        val store = FileTokenStore(dir)
        store.file.writeText("not json")
        assertNull(store.read())
        dir.deleteRecursively()
    }

    @Test
    fun dataDirFollowsPlatformConventions() {
        assertEquals("/h/Library/Application Support/Nomi", desktopDataDir("Mac OS X", mapOf(), "/h").path)
        assertEquals("C:\\Roaming\\Nomi", desktopDataDir("Windows 11", mapOf("APPDATA" to "C:\\Roaming"), "C:\\u").name.let { "C:\\Roaming\\$it" })
        assertEquals("/x/nomi", desktopDataDir("Linux", mapOf("XDG_DATA_HOME" to "/x"), "/h").path)
        assertEquals("/h/.local/share/nomi", desktopDataDir("Linux", mapOf(), "/h").path)
    }
}
