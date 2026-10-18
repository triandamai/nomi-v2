package ai.nomi.database

import android.content.Context
import androidx.room.Room

class AndroidDatabaseFactory(context: Context) : DatabaseFactory {
    private val appContext = context.applicationContext

    override fun create(): NomiDatabase =
        Room.databaseBuilder<NomiDatabase>(
            context = appContext,
            name = appContext.getDatabasePath(DATABASE_FILE_NAME).absolutePath,
        ).configureNomi()
}
