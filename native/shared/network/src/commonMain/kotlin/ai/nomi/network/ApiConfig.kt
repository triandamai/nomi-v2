package ai.nomi.network

private const val HTTPS_SCHEME = "https://"
private const val HTTP_SCHEME = "http://"
private const val WSS_SCHEME = "wss://"
private const val WS_SCHEME = "ws://"
private const val INSECURE_DEFAULT_PREFIX = "http://api-nomi.trian.space"
private const val SECURE_DEFAULT_PREFIX = "https://api-nomi.trian.space"

/** Where the Nomi backend lives. [baseUrl] has no trailing slash requirement; one is tolerated. */
class ApiConfig(rawBaseUrl: String) {
    val baseUrl: String = normalize(rawBaseUrl)
    private val root: String get() = baseUrl.trimEnd('/')

    fun url(path: String): String = root + path

    fun toWebSocketUrl(path: String): String = when {
        root.startsWith(HTTPS_SCHEME) -> WSS_SCHEME + root.removePrefix(HTTPS_SCHEME)
        root.startsWith(HTTP_SCHEME) -> WS_SCHEME + root.removePrefix(HTTP_SCHEME)
        else -> root
    } + path

    override fun equals(other: Any?): Boolean = other is ApiConfig && other.baseUrl == baseUrl

    override fun hashCode(): Int = baseUrl.hashCode()

    override fun toString(): String = "ApiConfig(baseUrl=$baseUrl)"

    companion object {
        fun normalize(url: String): String {
            val trimmed = url.trim()
            return if (trimmed.startsWith(INSECURE_DEFAULT_PREFIX)) {
                SECURE_DEFAULT_PREFIX + trimmed.removePrefix(INSECURE_DEFAULT_PREFIX)
            } else {
                trimmed
            }
        }
    }
}

