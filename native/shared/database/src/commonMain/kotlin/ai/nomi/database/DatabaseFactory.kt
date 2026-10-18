package ai.nomi.database

import androidx.room.RoomDatabase
import androidx.sqlite.driver.bundled.BundledSQLiteDriver
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.IO

internal const val DATABASE_FILE_NAME = "nomi.db"

/** Builds the on-disk [NomiDatabase] for a platform. */
interface DatabaseFactory {
    fun create(): NomiDatabase
}

/**
 * Shared Room configuration. The database is only a cache of server data, so a schema bump simply drops
 * and rebuilds every table (`fallbackToDestructiveMigration`) instead of shipping migrations; repositories
 * repopulate it from REST/realtime. Revisit before storing anything that exists only locally.
 */
internal fun RoomDatabase.Builder<NomiDatabase>.configureNomi(): NomiDatabase =
    setDriver(BundledSQLiteDriver())
        .setQueryCoroutineContext(Dispatchers.IO)
        .fallbackToDestructiveMigration(dropAllTables = true)
        .build()
