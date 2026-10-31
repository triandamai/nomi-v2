package ai.nomi.data

import ai.nomi.network.AttachmentsApi
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock

/** Previews kept in memory: enough for the images on a few screens of chat. */
private const val PREVIEWS_KEPT = 48

/**
 * Attached images' small previews, fetched once each and kept while the app runs. A preview that can't be fetched
 * (offline, deleted, not an image) is null, and asked for again next time.
 */
class AttachmentPreviews(private val api: AttachmentsApi, private val log: (String) -> Unit = {}) {
    private val lock = Mutex()
    private val kept = LinkedHashMap<String, ByteArray>()

    suspend fun preview(id: String): ByteArray? {
        lock.withLock { kept[id] }?.let { return it }
        val bytes = try {
            api.preview(id)
        } catch (cancelled: CancellationException) {
            throw cancelled
        } catch (error: Exception) {
            log("attachments: no preview for $id: ${error.message}")
            return null
        }
        lock.withLock {
            kept[id] = bytes
            while (kept.size > PREVIEWS_KEPT) kept.remove(kept.keys.first())
        }
        return bytes
    }
}
