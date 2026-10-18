package ai.nomi.network

import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.client.engine.mock.respondError
import io.ktor.client.engine.mock.toByteArray
import io.ktor.client.request.get
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.http.headersOf
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertFalse
import kotlin.test.assertNull
import kotlin.test.assertTrue

class AuthRefreshTest {
    private val config = ApiConfig("https://nomi.example")
    private val json = headersOf(HttpHeaders.ContentType, "application/json")

    @Test
    fun refreshStoresTheNewAccessTokenAndKeepsTheRefreshToken() = runTest {
        val store = FakeTokenStore(Tokens("old", "refresh-1"))
        var body = ""
        val engine = MockEngine { request ->
            assertEquals("/api/auth/refresh", request.url.encodedPath)
            body = request.body.toByteArray().decodeToString()
            respond("""{"access_token":"new"}""", HttpStatusCode.OK, json)
        }
        val api = AuthApi(nomiHttpClient(engine, config, store) {}, config, store)

        assertTrue(api.refresh())
        assertEquals("""{"refresh_token":"refresh-1"}""", body)
        assertEquals(Tokens("new", "refresh-1"), store.read())
    }

    @Test
    fun refreshWithoutASessionIsFalseAndSendsNothing() = runTest {
        val store = FakeTokenStore()
        val engine = MockEngine { error("no request expected") }
        val api = AuthApi(nomiHttpClient(engine, config, store) {}, config, store)

        assertFalse(api.refresh())
    }

    @Test
    fun rejectedRefreshClearsTheSession() = runTest {
        val store = FakeTokenStore(Tokens("old", "dead"))
        val engine = MockEngine { respond("", HttpStatusCode.Unauthorized) }
        val api = AuthApi(nomiHttpClient(engine, config, store) {}, config, store)

        assertFalse(api.refresh())
        assertNull(store.read())
    }

    @Test
    fun serverErrorIsNotAVerdictOnTheSession() = runTest {
        val store = FakeTokenStore(Tokens("old", "r"))
        val engine = MockEngine { respondError(HttpStatusCode.ServiceUnavailable) }
        val api = AuthApi(nomiHttpClient(engine, config, store) {}, config, store)

        assertFailsWith<ApiException> { api.refresh() }
        assertEquals(Tokens("old", "r"), store.read())
    }

    @Test
    fun requestsAfterARefreshCarryTheNewTokenNotTheCachedOne() = runTest {
        val store = FakeTokenStore(Tokens("old", "r"))
        val headers = mutableListOf<String?>()
        val engine = MockEngine { request ->
            if (request.url.encodedPath == "/api/auth/refresh") {
                respond("""{"access_token":"new"}""", HttpStatusCode.OK, json)
            } else {
                headers += request.headers[HttpHeaders.Authorization]
                respond("{}", HttpStatusCode.OK, json)
            }
        }
        val client = nomiHttpClient(engine, config, store) {}
        client.get("${config.baseUrl}/api/home")
        AuthApi(client, config, store).refresh()
        client.get("${config.baseUrl}/api/home")

        assertEquals<List<String?>>(listOf("Bearer old", "Bearer new"), headers)
    }

    @Test
    fun revokeSendsTheGivenTokenAndLeavesANewerSessionAlone() = runTest {
        val store = FakeTokenStore(Tokens("new-access", "new-refresh"))
        var body = ""
        val engine = MockEngine { request ->
            assertEquals("/api/auth/logout", request.url.encodedPath)
            body = request.body.toByteArray().decodeToString()
            respond("", HttpStatusCode.NoContent)
        }
        AuthApi(nomiHttpClient(engine, config, store) {}, config, store).revoke("old-refresh")

        assertEquals("""{"refresh_token":"old-refresh"}""", body)
        assertEquals(Tokens("new-access", "new-refresh"), store.read())
    }

    @Test
    fun forgetLocalSessionReturnsWhatWasStoredAndClearsIt() = runTest {
        val store = FakeTokenStore(Tokens("a", "r"))
        val api = AuthApi(nomiHttpClient(MockEngine { error("no request expected") }, config, store) {}, config, store)

        assertEquals(Tokens("a", "r"), api.forgetLocalSession())
        assertNull(store.read())
    }
}
