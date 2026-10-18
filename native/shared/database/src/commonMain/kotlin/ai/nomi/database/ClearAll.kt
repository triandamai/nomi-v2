package ai.nomi.database

import androidx.room.immediateTransaction
import androidx.room.useWriterConnection

/**
 * Empties every table in one write transaction (sign-out), so no row of the previous account survives a crash
 * halfway. Room's `clearAllTables()` is Android-only and blocking; the DAO deletes also notify open observers.
 */
suspend fun NomiDatabase.clearAll() {
    useWriterConnection { transactor ->
        transactor.immediateTransaction {
            agentActivityDao().clearAll()
            streamingDraftDao().clearAll()
            messageDao().clearAll()
            notificationDao().clearAll()
            sessionDao().clearAll()
        }
    }
}
