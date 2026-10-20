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
    fun preferencesReadsAndUpdatesThemeAndAccentAndHomeLayout() = runTest {
        val paths = mutableListOf<String>()
        val api = api("""{"theme":"dark","accent_color":"coral-reef","timezone":"UTC","language":"en","has_stored_timezone":true,"animated_backdrop":true,"drawer_recents":true,"drawer_recents_limit":10,"home_layout":"classic","drawer_pins":["money","reminders"]}""", paths = paths)
        val loaded = api.preferences()
        assertEquals(PreferencesDto(theme = "dark", accentColor = "coral-reef", homeLayout = "classic", drawerPins = listOf("money", "reminders"), drawerRecents = true, drawerRecentsLimit = 10), loaded)
        val updated = api.updatePreferences(UpdatePreferencesDto(theme = "dark", homeLayout = "classic"))
        assertEquals("coral-reef", updated.accentColor)
        assertEquals("classic", updated.homeLayout)
        assertEquals(listOf("/api/preferences", "/api/preferences"), paths)
    }

    @Test
    fun summaryReadsHomeStatusCards() = runTest {
        val paths = mutableListOf<String>()
        val summaryJson = """
            {
                "since": "2026-10-18T12:00:00Z",
                "timezone": "America/New_York",
                "while_you_were_out": [
                    {"kind": "finished", "agent": "coding", "title": "Build complete", "detail": "App compiled successfully", "session_id": "s-1", "at": "2026-10-18T13:00:00Z"}
                ],
                "today": [
                    {"id": "r-1", "run_at": "2026-10-18T14:00:00Z", "label": "Stretch at 4pm", "agent": "health", "recurrence": "daily"}
                ],
                "plans": [
                    {"kind": "todo", "title": "Weekend chores", "agent": "planning", "done": 2, "total": 5, "session_id": "s-2", "updated_at": "2026-10-18T13:30:00Z"}
                ],
                "while_you_were_out_total": 1,
                "today_total": 1,
                "plans_total": 1
            }
        """.trimIndent()
        val summary = api(summaryJson, paths = paths).summary()
        assertEquals("America/New_York", summary.timezone)
        assertEquals(1, summary.whileYouWereOut.size)
        assertEquals("Build complete", summary.whileYouWereOut.first().title)
        assertEquals("s-1", summary.whileYouWereOut.first().sessionId)
        assertEquals(1, summary.today.size)
        assertEquals("Stretch at 4pm", summary.today.first().label)
        assertEquals(1, summary.plans.size)
        assertEquals(2L, summary.plans.first().done)
        assertEquals(listOf("/api/home"), paths)
    }

    @Test
    fun usageApiReadsBrief() = runTest {
        val paths = mutableListOf<String>()
        val engine = MockEngine { request ->
            paths.add(request.url.encodedPath)
            respond("""{"plan":{"name":"Pro","monthly_tokens":1000000},"month":"2026-10","tokens_used":420000,"can_upgrade":true}""", HttpStatusCode.OK, json)
        }
        val client = nomiHttpClient(engine, config, FakeTokenStore(Tokens("a", "r"))) {}
        val usageApi = HttpUsageApi(client, config)
        val brief = usageApi.brief()
        assertEquals("Pro", brief.plan.name)
        assertEquals(1000000L, brief.plan.monthlyTokens)
        assertEquals(420000L, brief.tokensUsed)
        assertEquals(true, brief.canUpgrade)
        assertEquals(listOf("/api/usage/brief"), paths)
    }

    @Test
    fun projectsApiListsProjects() = runTest {
        val paths = mutableListOf<String>()
        val engine = MockEngine { request ->
            paths.add(request.url.encodedPath)
            respond("""{"org_id":"o1","projects":[{"id":"p1","session_id":"s1","name":"Alpha Web","description":"Website project","status":"building","created_at":"2026-10-01T00:00:00Z","updated_at":"2026-10-02T00:00:00Z","created_by":"Ana","open_issues":3}]}""", HttpStatusCode.OK, json)
        }
        val client = nomiHttpClient(engine, config, FakeTokenStore(Tokens("a", "r"))) {}
        val projectsApi = HttpProjectsApi(client, config)
        val list = projectsApi.list()
        assertEquals(1, list.size)
        assertEquals("Alpha Web", list.first().name)
        assertEquals("building", list.first().status)
        assertEquals(listOf("/api/projects"), paths)
    }

    @Test
    fun failuresThrow() = runTest {
        assertFailsWith<ApiException> { api("""{"error":"x"}""", HttpStatusCode.InternalServerError).crew() }
    }
}
