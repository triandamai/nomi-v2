package ai.nomi.data

import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.asSharedFlow

/** Google's answer to "Continue with Google", as the web's callback hands it to the app. */
sealed interface GoogleReturn {
    data class Code(val code: String, val state: String) : GoogleReturn

    /** Cancelled on Google's page, or Google sign-in isn't available (`unavailable`). */
    data class Failed(val reason: String) : GoogleReturn
}

/**
 * The `nomi://auth/google?code=…&state=…` links the platforms receive (Android's intent, iOS's web-auth session),
 * on their way to the sign-in screen.
 */
object GoogleSignInReturns {
    const val LINK_PREFIX = "nomi://auth/google"
    private const val CANCELLED = "cancelled"
    private val returns = MutableSharedFlow<GoogleReturn>(extraBufferCapacity = 1)
    val flow: SharedFlow<GoogleReturn> = returns.asSharedFlow()

    /** Whether [link] is Google's answer (and not a link to a page of the app). */
    fun handles(link: String): Boolean = link.startsWith(LINK_PREFIX)

    /** Hands [link] on; false when it isn't Google's answer. */
    fun deliver(link: String): Boolean {
        val parsed = parse(link) ?: return false
        returns.tryEmit(parsed)
        return true
    }

    fun parse(link: String): GoogleReturn? {
        if (!handles(link)) return null
        val query = link.substringAfter('?', missingDelimiterValue = "")
        val params = query.split('&').filter { it.isNotEmpty() }.associate { part ->
            part.substringBefore('=') to decode(part.substringAfter('=', missingDelimiterValue = ""))
        }
        val code = params["code"]
        val state = params["state"]
        return if (code != null && state != null) GoogleReturn.Code(code, state) else GoogleReturn.Failed(params["error"] ?: CANCELLED)
    }

    private const val HEX = 16
    private const val ESCAPE_LENGTH = 3

    /** Percent-decoding of a query value (`+` is a space). */
    private fun decode(value: String): String {
        val bytes = ArrayList<Byte>(value.length)
        var i = 0
        while (i < value.length) {
            val c = value[i]
            when {
                c == '+' -> { bytes += ' '.code.toByte(); i++ }
                c == '%' && i + 2 <= value.lastIndex -> {
                    bytes += value.substring(i + 1, i + ESCAPE_LENGTH).toInt(HEX).toByte()
                    i += ESCAPE_LENGTH
                }
                else -> { c.toString().encodeToByteArray().forEach { bytes += it }; i++ }
            }
        }
        return bytes.toByteArray().decodeToString()
    }
}
