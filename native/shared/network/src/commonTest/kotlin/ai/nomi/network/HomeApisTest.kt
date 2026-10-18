package ai.nomi.network

import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.http.headersOf
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith

class HomeApisTest {
    private val config = ApiConfig("https://nomi.example")
    private val json = headersOf(HttpHeaders.ContentType, "application/json")

    private fun api(body: String, status: HttpStatusCode = HttpStatusCode.OK, paths: MutableList<String>? = null): HomeApi {
        val engine = MockEngine { request ->
            paths?.add(request.url.encodedPath)
            respond(body, status, json)
        }
        return HttpHomeApi(nomiHttpClient(engine, config, FakeTokenStore(Tokens("a", "r"))) {}, config)
    }

    @Test
    fun profileReadsTheDisplayName() = runTest {
        val paths = mutableListOf<String>()
        val profile = api("""{"display_name":"Ana Lima","username":null,"email":"ana@nomi.test","avatar_url":null}""", paths = paths).profile()
        assertEquals("Ana Lima", profile.displayName)
        assertEquals(listOf("/api/profile"), paths)
    }

    @Test
    fun crewReadsTheAgentsWithTheirState() = runTest {
        val crew = api(
            """{"agents":[{"agent_type":"chitchat","name":"Nomi","role":"Talks with you","is_dynamic":false,"shape":null,"tone":null,"motion":null,"state":"working","status":"Planning your Saturday"}]}""",
        ).crew()
        assertEquals(listOf(CrewMemberDto("chitchat", "Nomi", "Talks with you", false, null, null, null, "working", "Planning your Saturday")), crew)
    }

    @Test
    fun preferencesReadsAndUpdatesThemeAndAccent() = runTest {
        val paths = mutableListOf<String>()
        val api = api("""{"theme":"dark","accent_color":"coral-reef","timezone":"UTC","language":"en","has_stored_timezone":true,"animated_backdrop":true,"drawer_recents":true,"drawer_recents_limit":10}""", paths = paths)
        val loaded = api.preferences()
        assertEquals(PreferencesDto(theme = "dark", accentColor = "coral-reef"), loaded)
        val updated = api.updatePreferences(UpdatePreferencesDto(theme = "dark"))
        assertEquals("coral-reef", updated.accentColor)
        assertEquals(listOf("/api/preferences", "/api/preferences"), paths)
    }

    @Test
    fun failuresThrow() = runTest {
        assertFailsWith<ApiException> { api("""{"error":"x"}""", HttpStatusCode.InternalServerError).crew() }
    }
}
