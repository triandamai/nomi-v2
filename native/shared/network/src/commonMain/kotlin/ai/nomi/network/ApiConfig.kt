package ai.nomi.network

private const val INSECURE_DEFAULT_PREFIX = "http://api-nomi.trian.space"
private const val SECURE_DEFAULT_PREFIX = "https://api-nomi.trian.space"

/**
 * Where the Nomi backend lives; the Rust engine builds every URL under [baseUrl]. The hosted API is always reached
 * over https, even when configured with http.
 */
class ApiConfig(rawBaseUrl: String) {
    val baseUrl: String = normalize(rawBaseUrl)

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
