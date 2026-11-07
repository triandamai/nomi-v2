package ai.nomi.network

import kotlin.test.Test
import kotlin.test.assertEquals

class ApiConfigTest {
    @Test
    fun theHostedApiIsAlwaysReachedOverHttps() {
        val host = HOSTED_API_URL.substringAfter("://").trimEnd('/')
        assertEquals("https://$host", ApiConfig("http://$host").baseUrl)
        assertEquals("https://$host/", ApiConfig(" http://$host/ ").baseUrl)
        assertEquals("https://$host", ApiConfig(HOSTED_API_URL).baseUrl)
    }

    @Test
    fun otherBackendsAreKeptAsGiven() {
        assertEquals("http://localhost:8080", ApiConfig("http://localhost:8080").baseUrl)
        assertEquals("https://nomi.example/base", ApiConfig("https://nomi.example/base").baseUrl)
    }

    @Test
    fun theWebAppIsTheApisHostWithoutItsPrefix() {
        assertEquals("https://nomi.trian.space", ApiConfig("https://api-nomi.trian.space").webUrl)
        assertEquals("http://localhost:8080", ApiConfig("http://localhost:8080/").webUrl)
    }
}
