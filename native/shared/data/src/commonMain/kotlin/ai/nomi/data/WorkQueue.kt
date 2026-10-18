package ai.nomi.data

import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.launch

/**
 * Runs suspending work one item at a time, in submission order, on [scope]. Hub listeners are plain callbacks, so
 * each event is queued here: deltas must be appended in order, and a completed turn must not clear a draft before
 * the deltas queued ahead of it have been written. A failing item is logged and does not stop the queue.
 */
internal class WorkQueue(scope: CoroutineScope, private val log: (String) -> Unit) {
    private val work = Channel<suspend () -> Unit>(Channel.UNLIMITED)

    private val consumer: Job = scope.launch {
        for (item in work) {
            // Throwable, not Exception: an Error from one item (e.g. TODO()) must not stop every later one.
            try {
                item()
            } catch (e: CancellationException) {
                throw e
            } catch (e: Throwable) {
                log("data: queued work failed: $e")
            }
        }
    }

    /** False once its scope was cancelled (the session ended): nothing submitted will run any more. */
    val isActive: Boolean get() = consumer.isActive

    fun submit(item: suspend () -> Unit) {
        work.trySend(item)
    }

    /** Work already queued still runs; later [submit]s are dropped. Idempotent. */
    fun close() {
        work.close()
    }
}
