@file:OptIn(ExperimentalForeignApi::class)

package ai.nomi.database

import androidx.room.Room
import kotlinx.cinterop.ExperimentalForeignApi
import platform.Foundation.NSApplicationSupportDirectory
import platform.Foundation.NSFileManager
import platform.Foundation.NSURL
import platform.Foundation.NSUserDomainMask

/** Stores the database in the app's Application Support directory (backed up, not user-visible). */
class IosDatabaseFactory : DatabaseFactory {
    override fun create(): NomiDatabase {
        val dir: NSURL = requireNotNull(
            NSFileManager.defaultManager.URLForDirectory(
                directory = NSApplicationSupportDirectory,
                inDomain = NSUserDomainMask,
                appropriateForURL = null,
                create = true,
                error = null,
            ),
        ) { "Application Support directory unavailable" }
        return Room.databaseBuilder<NomiDatabase>(name = requireNotNull(dir.path) + "/" + DATABASE_FILE_NAME).configureNomi()
    }
}
