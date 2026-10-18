package ai.nomi.data

import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.async
import kotlinx.coroutines.cancelAndJoin
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.ensureActive
import kotlinx.coroutines.job
import kotlin.concurrent.Volatile

/** Thrown by [SessionScope.run] when the signed-in session ended before (or while) the work ran. */
class SessionEndedException : IllegalStateException("the signed-in session has ended")

/**
 * The coroutine scope of one signed-in session, a child of the app scope [parent]. Everything that writes the local
 * cache on behalf of an account runs in it: the user-topic queue, chat binding queues, and repository calls made from
 * screens (through [run]). Sign-out [close]s it — cancelling and joining all of that — before the cache is wiped, so
 * nothing from the previous account can land in Room afterwards.
 *
 * [open] and [close] are called by `SessionGate` (serialized by its lock); [scope] and [run] from anywhere.
 */
class SessionScope(private val parent: CoroutineScope) {
    @Volatile
    private var current: CoroutineScope? = null

    /** The open session's scope; between sessions a cancelled one, so whatever is launched in it never runs. */
    val scope: CoroutineScope get() = current ?: ended

    /** Idempotent. */
    fun open() {
        if (current != null) return
        current = CoroutineScope(parent.coroutineContext + SupervisorJob(parent.coroutineContext[Job]))
    }

    /** Cancels the session's work and waits for it to finish. Idempotent. */
    suspend fun close() {
        val closing = current ?: return
        current = null
        closing.coroutineContext.job.cancelAndJoin()
    }

    /**
     * Runs [block] as part of the current session (not of the caller), so sign-out cancels it before the wipe.
     * Throws [SessionEndedException] when there is no session or it ends meanwhile; the caller's own cancellation is
     * rethrown as is.
     */
    suspend fun <T> run(block: suspend () -> T): T {
        val work = scope.async { block() }
        return try {
            work.await()
        } catch (e: CancellationException) {
            currentCoroutineContext().ensureActive()
            throw SessionEndedException()
        }
    }

    private companion object {
        val ended = CoroutineScope(Job().apply { cancel() })
    }
}
