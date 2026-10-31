package ai.nomi.data

/** Something listening (e.g. a chat kept in sync while its screen is open); [remove] stops it. Idempotent. */
fun interface ListenerHandle {
    fun remove()
}
