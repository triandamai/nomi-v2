package ai.nomi.data

import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.launch

/**
 * Runs suspending work one item at a time, in submission order, on [scope]. Hub listeners are plain callbacks, so
 * each event is queued here: deltas must be appended in order, and a completed turn must not clear a draft before
 * the deltas queued ahead of it have been written. A failing item is logged and does not stop the queue.
 */
internal class WorkQueue(scope: CoroutineScope, private val log: (String) -> Unit) {
    private val work = Channel<suspend () -> Unit>(Channel.UNLIMITED)

    init {
        scope.launch {
            for (item in work) {
                try {
                    item()
                } catch (e: CancellationException) {
                    throw e
                } catch (e: Exception) {
                    log("data: queued work failed: $e")
                }
            }
        }
    }

    fun submit(item: suspend () -> Unit) {
        work.trySend(item)
    }

    /** Work already queued still runs; later [submit]s are dropped. Idempotent. */
    fun close() {
        work.close()
    }
}
