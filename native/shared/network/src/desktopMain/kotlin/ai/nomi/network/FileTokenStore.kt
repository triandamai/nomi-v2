package ai.nomi.network

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import kotlinx.serialization.json.Json
import java.io.File
import java.nio.file.Files
import java.nio.file.attribute.PosixFilePermission

private const val TOKEN_FILE_NAME = "tokens.json"
private val OWNER_ONLY = setOf(PosixFilePermission.OWNER_READ, PosixFilePermission.OWNER_WRITE)

/** Tokens in a 0600 file inside the app data dir: the fallback when no OS keyring is available. */
class FileTokenStore(private val dir: File) : TokenStore {
    val file: File get() = File(dir, TOKEN_FILE_NAME)

    override suspend fun read(): Tokens? = withContext(Dispatchers.IO) {
        runCatching { Json.decodeFromString(Tokens.serializer(), file.readText()) }.getOrNull()
    }

    override suspend fun write(tokens: Tokens) = withContext(Dispatchers.IO) {
        dir.mkdirs()
        val target = file
        if (!target.exists()) {
            runCatching { Files.createFile(target.toPath(), java.nio.file.attribute.PosixFilePermissions.asFileAttribute(OWNER_ONLY)) }
        }
        runCatching { Files.setPosixFilePermissions(target.toPath(), OWNER_ONLY) }
        target.writeText(Json.encodeToString(Tokens.serializer(), tokens))
    }

    override suspend fun clear() = withContext(Dispatchers.IO) {
        file.delete()
        Unit
    }
}

/** Per-OS app data directory: macOS Application Support, Windows %APPDATA%, Linux XDG data home. */
fun desktopDataDir(
    osName: String = System.getProperty("os.name"),
    env: Map<String, String> = System.getenv(),
    home: String = System.getProperty("user.home"),
): File = when {
    osName.startsWith("Mac", ignoreCase = true) -> File(home, "Library/Application Support/Nomi")
    osName.startsWith("Windows", ignoreCase = true) -> File(env["APPDATA"] ?: File(home, "AppData/Roaming").path, "Nomi")
    else -> File(env["XDG_DATA_HOME"]?.takeIf { it.isNotBlank() } ?: File(home, ".local/share").path, "nomi")
}
