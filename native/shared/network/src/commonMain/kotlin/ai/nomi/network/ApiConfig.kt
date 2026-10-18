package ai.nomi.network

private const val HTTPS_SCHEME = "https://"
private const val HTTP_SCHEME = "http://"
private const val WSS_SCHEME = "wss://"
private const val WS_SCHEME = "ws://"

/** Where the Nomi backend lives. [baseUrl] has no trailing slash requirement; one is tolerated. */
data class ApiConfig(val baseUrl: String) {
    private val root: String get() = baseUrl.trimEnd('/')

    fun url(path: String): String = root + path

    fun toWebSocketUrl(path: String): String = when {
        root.startsWith(HTTPS_SCHEME) -> WSS_SCHEME + root.removePrefix(HTTPS_SCHEME)
        root.startsWith(HTTP_SCHEME) -> WS_SCHEME + root.removePrefix(HTTP_SCHEME)
        else -> root
    } + path
}
