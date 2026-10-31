package ai.nomi.sdk

import kotlin.io.encoding.Base64
import kotlin.io.encoding.ExperimentalEncodingApi

/** A [CredentialStore] in memory, recording what the engine did with it. */
class MemoryCredentialStore(initial: Credentials? = null) : CredentialStore {
    var stored: Credentials? = initial
        private set
    var clears = 0
        private set

    override fun load(): Credentials? = stored

    override fun save(credentials: Credentials) {
        stored = credentials
    }

    override fun clear() {
        clears++
        stored = null
    }
}

/** An unsigned JWT-shaped access token naming [userId], as the backend's tokens do in `sub`. */
@OptIn(ExperimentalEncodingApi::class)
fun accessToken(userId: String): String {
    val base64 = Base64.UrlSafe.withPadding(Base64.PaddingOption.ABSENT)
    val header = base64.encode("""{"alg":"none"}""".encodeToByteArray())
    val payload = base64.encode("""{"sub":"$userId"}""".encodeToByteArray())
    return "$header.$payload.signature"
}

/** A port nothing listens on: connections are refused at once. */
const val UNREACHABLE_BASE_URL = "http://127.0.0.1:9"
