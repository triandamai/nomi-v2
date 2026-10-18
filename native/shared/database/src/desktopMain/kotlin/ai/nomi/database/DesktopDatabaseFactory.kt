package ai.nomi.database

import androidx.room.Room
import java.io.File

/** [dataDir] is the app data dir (e.g. the network module's `desktopDataDir()`); it is created if missing. */
class DesktopDatabaseFactory(private val dataDir: File) : DatabaseFactory {
    override fun create(): NomiDatabase {
        dataDir.mkdirs()
        return Room.databaseBuilder<NomiDatabase>(name = File(dataDir, DATABASE_FILE_NAME).absolutePath).configureNomi()
    }
}
