package ai.nomi.realtime

import kotlin.random.Random

/**
 * Reconnect delays: exponential `BASE_DELAY_MS · 2^attempt`, capped at [MAX_DELAY_MS], with full jitter
 * (uniform in `0..ceiling`). Stateless: the hub owns the attempt counter and resets it once a connection is stable.
 * [random] is injectable so tests are deterministic.
 */
class Backoff(private val random: Random = Random.Default) {
    fun delayFor(attempt: Int): Long {
        val exponent = attempt.coerceIn(FIRST_ATTEMPT, MAX_EXPONENT)
        val ceiling = (BASE_DELAY_MS shl exponent).coerceAtMost(MAX_DELAY_MS)
        return (NO_DELAY_MS..ceiling).random(random)
    }

    companion object {
        const val BASE_DELAY_MS: Long = 500L
        const val MAX_DELAY_MS: Long = 30_000L

        private const val FIRST_ATTEMPT = 0
        private const val NO_DELAY_MS = 0L

        /** `BASE_DELAY_MS shl 6` already exceeds the cap; clamping here also keeps the shift from overflowing. */
        private const val MAX_EXPONENT = 6
    }
}
