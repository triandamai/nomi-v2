package ai.nomi.network

import kotlin.test.Test
import kotlin.test.assertEquals

class ApiConfigTest {
    @Test
    fun theHostedApiIsAlwaysReachedOverHttps() {
        assertEquals("https://api-nomi.trian.space", ApiConfig("http://api-nomi.trian.space").baseUrl)
        assertEquals("https://api-nomi.trian.space/", ApiConfig(" http://api-nomi.trian.space/ ").baseUrl)
    }

    @Test
    fun otherBackendsAreKeptAsGiven() {
        assertEquals("http://localhost:8080", ApiConfig("http://localhost:8080").baseUrl)
        assertEquals("https://nomi.example/base", ApiConfig("https://nomi.example/base").baseUrl)
    }
}
