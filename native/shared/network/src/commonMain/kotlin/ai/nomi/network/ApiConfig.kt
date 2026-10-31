package ai.nomi.network

private const val INSECURE_DEFAULT_PREFIX = "http://api-nomi.trian.space"
private const val SECURE_DEFAULT_PREFIX = "https://api-nomi.trian.space"
/** The hosted API sits at `api-<web host>`; the web app (and its share pages) at the host without it. */
private const val API_HOST_PREFIX = "api-"
private const val SCHEME_SEPARATOR = "://"

/**
 * Where the Nomi backend lives; the Rust engine builds every URL under [baseUrl]. The hosted API is always reached
 * over https, even when configured with http.
 */
class ApiConfig(rawBaseUrl: String) {
    val baseUrl: String = normalize(rawBaseUrl)

    /** The web app, for links people open in a browser (shared replies): the API's host without `api-`. */
    val webUrl: String = webOf(baseUrl)

    override fun equals(other: Any?): Boolean = other is ApiConfig && other.baseUrl == baseUrl

    override fun hashCode(): Int = baseUrl.hashCode()

    override fun toString(): String = "ApiConfig(baseUrl=$baseUrl)"

    companion object {
        fun webOf(apiUrl: String): String {
            val scheme = apiUrl.substringBefore(SCHEME_SEPARATOR, missingDelimiterValue = "")
            val rest = apiUrl.substringAfter(SCHEME_SEPARATOR).trimEnd('/')
            if (scheme.isEmpty() || !rest.startsWith(API_HOST_PREFIX)) return apiUrl.trimEnd('/')
            return scheme + SCHEME_SEPARATOR + rest.removePrefix(API_HOST_PREFIX)
        }

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
