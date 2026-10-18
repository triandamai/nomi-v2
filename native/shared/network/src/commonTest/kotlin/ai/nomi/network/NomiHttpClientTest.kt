package ai.nomi.network

import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.client.engine.mock.toByteArray
import io.ktor.client.request.get
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpMethod
import io.ktor.http.HttpStatusCode
import io.ktor.http.headersOf
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNull

class NomiHttpClientTest {
    private val config = ApiConfig("https://nomi.example")
    private val json = headersOf(HttpHeaders.ContentType, "application/json")

    @Test
    fun unauthorizedTriggersExactlyOneRefreshThenRetriesWithNewToken() = runTest {
        val store = FakeTokenStore(Tokens("old-access", "refresh-1"))
        val seen = mutableListOf<String>()
        var refreshBody = ""
        val engine = MockEngine { request ->
            seen += "${request.method.value} ${request.url.encodedPath} ${request.headers[HttpHeaders.Authorization]}"
            when (request.url.encodedPath) {
                "/api/auth/refresh" -> {
                    refreshBody = request.body.toByteArray().decodeToString()
                    respond("""{"access_token":"new-access"}""", HttpStatusCode.OK, json)
                }
                else -> if (request.headers[HttpHeaders.Authorization] == "Bearer new-access") {
                    respond("{}", HttpStatusCode.OK, json)
                } else {
                    respond("", HttpStatusCode.Unauthorized)
                }
            }
        }
        val client = nomiHttpClient(engine, config, store) { error("must not sign out") }

        val response = client.get("${config.baseUrl}/api/home")

        assertEquals(HttpStatusCode.OK, response.status)
        assertEquals(1, seen.count { it.contains("/api/auth/refresh") })
        assertEquals("""{"refresh_token":"refresh-1"}""", refreshBody)
        assertEquals("GET /api/home Bearer new-access", seen.last())
        assertEquals(Tokens("new-access", "refresh-1"), store.read())
    }

    @Test
    fun refreshRejectedClearsStoreAndSignsOutOnce() = runTest {
        val store = FakeTokenStore(Tokens("old-access", "dead-refresh"))
        var signedOut = 0
        val engine = MockEngine { request ->
            when (request.url.encodedPath) {
                "/api/auth/refresh" -> respond("", HttpStatusCode.Unauthorized)
                else -> respond("", HttpStatusCode.Unauthorized)
            }
        }
        val client = nomiHttpClient(engine, config, store) { signedOut++ }

        val response = client.get("${config.baseUrl}/api/home")

        assertEquals(HttpStatusCode.Unauthorized, response.status)
        assertEquals(1, signedOut)
        assertNull(store.read())
        assertEquals(1, store.clearCount)
    }

    @Test
    fun aRejectedRefreshForAnOlderAccountLeavesTheNewerSessionAlone() = runTest {
        val store = FakeTokenStore(Tokens("a-access", "a-refresh"))
        var signedOut = 0
        val engine = MockEngine { request ->
            when (request.url.encodedPath) {
                "/api/auth/refresh" -> {
                    // B signs in while A's refresh is on the wire.
                    store.write(Tokens("b-access", "b-refresh"))
                    respond("", HttpStatusCode.Unauthorized)
                }
                else -> respond("", HttpStatusCode.Unauthorized)
            }
        }
        val client = nomiHttpClient(engine, config, store) { signedOut++ }

        client.get("${config.baseUrl}/api/home")

        assertEquals(Tokens("b-access", "b-refresh"), store.read())
        assertEquals(0, store.clearCount)
        assertEquals(0, signedOut)
    }

    @Test
    fun aSuccessfulRefreshForAnOlderAccountDoesNotOverwriteTheNewerSession() = runTest {
        val store = FakeTokenStore(Tokens("a-access", "a-refresh"))
        val engine = MockEngine { request ->
            when (request.url.encodedPath) {
                "/api/auth/refresh" -> {
                    store.write(Tokens("b-access", "b-refresh"))
                    respond("""{"access_token":"a-new"}""", HttpStatusCode.OK, json)
                }
                else -> respond("", HttpStatusCode.Unauthorized)
            }
        }
        val client = nomiHttpClient(engine, config, store) { error("must not sign out") }

        client.get("${config.baseUrl}/api/home")

        assertEquals(Tokens("b-access", "b-refresh"), store.read())
    }

    @Test
    fun clearedStoreIsNotServedFromTheAuthPluginCache() = runTest {
        val store = FakeTokenStore(Tokens("cached", "r"))
        val headers = mutableListOf<String?>()
        val engine = MockEngine { request ->
            headers += request.headers[HttpHeaders.Authorization]
            respond("{}", HttpStatusCode.OK, json)
        }
        val client = nomiHttpClient(engine, config, store) {}
        client.get("${config.baseUrl}/api/home")
        store.clear()
        client.clearCachedTokens()
        client.get("${config.baseUrl}/api/home")

        assertEquals(listOf<String?>("Bearer cached", null), headers)
    }

    @Test
    fun bearerIsOnlySentToTheApiHost() = runTest {
        val store = FakeTokenStore(Tokens("secret", "r"))
        val seen = mutableMapOf<String, String?>()
        val engine = MockEngine { request ->
            seen[request.url.host] = request.headers[HttpHeaders.Authorization]
            respond("{}", HttpStatusCode.OK, json)
        }
        val client = nomiHttpClient(engine, config, store) {}
        client.get("${config.baseUrl}/api/home")
        client.get("https://other.example/x")
        assertEquals("Bearer secret", seen["nomi.example"])
        assertNull(seen["other.example"])
    }

    @Test
    fun authPathsUnderABasePathPrefixDoNotTriggerRefresh() = runTest {
        val prefixed = ApiConfig("https://nomi.example/nomi")
        val store = FakeTokenStore(Tokens("a", "r"))
        var refreshes = 0
        val engine = MockEngine { request ->
            if (request.url.encodedPath.endsWith("/auth/refresh")) refreshes++
            respond("", HttpStatusCode.Unauthorized)
        }
        val client = nomiHttpClient(engine, prefixed, store) { error("must not sign out") }
        AuthApi(client, prefixed, store).also {
            runCatching { it.login("a@b.c", "bad") }
        }
        assertEquals(0, refreshes)
        assertEquals(Tokens("a", "r"), store.read())
    }

    @Test
    fun logoutSurvivesNetworkFailureAndStillClears() = runTest {
        val store = FakeTokenStore(Tokens("a", "r"))
        val engine = MockEngine { throw RuntimeException("offline") }
        AuthApi(nomiHttpClient(engine, config, store) {}, config, store).logout()
        assertNull(store.read())
    }

    @Test
    fun loginParsesTokensAndEmailCodeChallenge() = runTest {
        val tokenEngine = MockEngine { request ->
            assertEquals(HttpMethod.Post, request.method)
            assertEquals("/api/auth/login", request.url.encodedPath)
            assertEquals("""{"email":"a@b.c","password":"pw"}""", request.body.toByteArray().decodeToString())
            respond("""{"access_token":"a","refresh_token":"r"}""", HttpStatusCode.OK, json)
        }
        val store = FakeTokenStore()
        val tokenApi = AuthApi(nomiHttpClient(tokenEngine, config, store) {}, config, store)
        assertEquals(SignInResult.SignedIn(Tokens("a", "r")), tokenApi.login("a@b.c", "pw"))
        assertEquals(Tokens("a", "r"), store.read())

        val challengeEngine = MockEngine {
            respond(
                """{"verification":{"challenge_id":"c1","purpose":"login","email":"a***@b.c","expires_in":600,"resend_in":30,"sends_left":4}}""",
                HttpStatusCode.OK,
                json,
            )
        }
        val challengeStore = FakeTokenStore()
        val challengeApi = AuthApi(nomiHttpClient(challengeEngine, config, challengeStore) {}, config, challengeStore)
        val result = challengeApi.login("a@b.c", "pw")
        assertEquals(
            SignInResult.EmailCode(VerificationInfo("c1", "login", "a***@b.c", 600, 30, 4)),
            result,
        )
        assertNull(challengeStore.read())
    }

    @Test
    fun logoutSendsRefreshTokenAndClearsLocalState() = runTest {
        val store = FakeTokenStore(Tokens("a", "r"))
        var body = ""
        val engine = MockEngine { request ->
            body = request.body.toByteArray().decodeToString()
            respond("", HttpStatusCode.NoContent)
        }
        val client = nomiHttpClient(engine, config, store) {}
        AuthApi(client, config, store).logout()
        assertEquals("""{"refresh_token":"r"}""", body)
        assertNull(store.read())
    }

    @Test
    fun webSocketUrlMapsSchemes() {
        assertEquals("wss://nomi.example/api/realtime/ws", ApiConfig("https://nomi.example").toWebSocketUrl("/api/realtime/ws"))
        assertEquals("ws://localhost:3000/x", ApiConfig("http://localhost:3000").toWebSocketUrl("/x"))
    }
}
