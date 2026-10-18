package ai.nomi.network

import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.client.engine.mock.toByteArray
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpMethod
import io.ktor.http.HttpStatusCode
import io.ktor.http.headersOf
import kotlinx.coroutines.test.runTest
import kotlinx.serialization.json.JsonArray
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertNull
import kotlin.test.assertTrue

class RestApisTest {
    private val config = ApiConfig("https://nomi.example")
    private val json = headersOf(HttpHeaders.ContentType, "application/json")

    private fun client(engine: MockEngine) = nomiHttpClient(engine, config, FakeTokenStore(Tokens("a", "r"))) {}

    private val messageJson = """
        {"id":"m1","sender":"assistant","content":"hi","content_blocks":[{"kind":"reasoning","text":"x"}],
         "created_at":"2026-10-18T10:00:00.123456Z","my_feedback":null,"agent_display_name":"Planner",
         "memory_count":0,"steps":[],"sources":[],"unexpected":true}
    """.trimIndent()

    @Test
    fun listSessionsParsesTheServerShape() = runTest {
        val engine = MockEngine { request ->
            assertEquals(HttpMethod.Get, request.method)
            assertEquals("/api/sessions", request.url.encodedPath)
            respond(
                """{"sessions":[{"id":"s1","channel":"web","chat_type":"dm","chat_id":"c","title":null,
                   "last_message":{"content":"hey","created_at":"2026-10-18T10:00:00Z"},"agent_active":true,
                   "updated_at":"2026-10-18T10:00:00Z","project_id":null}]}""",
                HttpStatusCode.OK,
                json,
            )
        }
        val sessions = HttpSessionsApi(client(engine), config).list()
        val only = sessions.single()
        assertEquals("s1", only.id)
        assertNull(only.title)
        assertEquals("hey", only.lastMessage?.content)
        assertTrue(only.agentActive)
    }

    @Test
    fun createSessionReturnsTheNewId() = runTest {
        val engine = MockEngine { request ->
            assertEquals(HttpMethod.Post, request.method)
            assertEquals("/api/sessions", request.url.encodedPath)
            respond("""{"session_id":"new"}""", HttpStatusCode.Created, json)
        }
        assertEquals("new", HttpSessionsApi(client(engine), config).create())
    }

    @Test
    fun messagePageSendsLimitAndBeforeAndParses() = runTest {
        val engine = MockEngine { request ->
            assertEquals("/api/sessions/s1/messages", request.url.encodedPath)
            assertEquals("50", request.url.parameters["limit"])
            assertEquals("m9", request.url.parameters["before"])
            respond("""{"messages":[$messageJson],"has_more":true}""", HttpStatusCode.OK, json)
        }
        val page = HttpMessagesApi(client(engine), config).page("s1", before = "m9")
        assertTrue(page.hasMore)
        val message = page.messages.single()
        assertEquals("assistant", message.sender)
        assertEquals("Planner", message.agentDisplayName)
        assertTrue(message.contentBlocks is JsonArray)
    }

    @Test
    fun resolveApprovalPutsTheDecision() = runTest {
        val engine = MockEngine { request ->
            assertEquals(HttpMethod.Put, request.method)
            assertEquals("/api/sessions/s1/messages/m2/approval", request.url.encodedPath)
            assertEquals("""{"decision":"deny","remember":true}""", request.body.toByteArray().decodeToString())
            respond("", HttpStatusCode.Accepted)
        }
        HttpMessagesApi(client(engine), config).resolveApproval("s1", "m2", approve = false, remember = true)
    }

    @Test
    fun resolveApprovalNoLongerPendingThrowsTheStatus() = runTest {
        val engine = MockEngine { respond("this action is no longer pending", HttpStatusCode.Conflict) }
        val error = assertFailsWith<ApiException> {
            HttpMessagesApi(client(engine), config).resolveApproval("s1", "m2", approve = true, remember = false)
        }
        assertEquals(HttpStatusCode.Conflict.value, error.status)
    }

    @Test
    fun latestPageOmitsBefore() = runTest {
        val engine = MockEngine { request ->
            assertNull(request.url.parameters["before"])
            respond("""{"messages":[],"has_more":false}""", HttpStatusCode.OK, json)
        }
        assertEquals(emptyList(), HttpMessagesApi(client(engine), config).page("s1").messages)
    }

    @Test
    fun getMessageHitsTheRenderedMessageEndpoint() = runTest {
        val engine = MockEngine { request ->
            assertEquals("/api/sessions/s1/messages/m1", request.url.encodedPath)
            respond(messageJson, HttpStatusCode.OK, json)
        }
        assertEquals("m1", HttpMessagesApi(client(engine), config).get("s1", "m1").id)
    }

    @Test
    fun sendPostsTextAndReturnsTheStoredUserMessage() = runTest {
        var body = ""
        val engine = MockEngine { request ->
            assertEquals(HttpMethod.Post, request.method)
            assertEquals("/api/sessions/s1/messages", request.url.encodedPath)
            body = request.body.toByteArray().decodeToString()
            respond("""{"user_message":$messageJson}""", HttpStatusCode.OK, json)
        }
        val response = HttpMessagesApi(client(engine), config).send("s1", "hello")
        assertEquals("""{"text":"hello"}""", body)
        assertEquals("m1", response.userMessage.id)
        assertNull(response.supervisorReply)
    }

    @Test
    fun nonSuccessBecomesApiException() = runTest {
        val engine = MockEngine { respond("message not found", HttpStatusCode.NotFound) }
        val error = assertFailsWith<ApiException> { HttpMessagesApi(client(engine), config).get("s1", "gone") }
        assertEquals(HttpStatusCode.NotFound.value, error.status)
    }

    @Test
    fun notificationsListUnreadAndMarkRead() = runTest {
        val seen = mutableListOf<String>()
        val engine = MockEngine { request ->
            seen += "${request.method.value} ${request.url.encodedPath}${request.url.encodedQuery.let { if (it.isEmpty()) "" else "?$it" }}"
            when (request.url.encodedPath) {
                "/api/notifications" -> respond(
                    """{"items":[{"id":"n1","kind":"reply","title":"T","body":"B","link":"/chat/s1","read":false,
                       "created_at":"2026-10-18T10:00:00Z"}],"unread":1,"page":1,"pages":1,"page_size":20,"total":1,
                       "counts":{"all":1,"unread":1,"read":0,"promo":0}}""",
                    HttpStatusCode.OK,
                    json,
                )
                "/api/notifications/unread" -> respond("""{"unread":3,"latest":{"id":"n1","title":"T","link":null}}""", HttpStatusCode.OK, json)
                else -> respond("", HttpStatusCode.NoContent)
            }
        }
        val api = HttpNotificationsApi(client(engine), config)
        val page = api.list(page = 2)
        assertEquals("reply", page.items.single().kind)
        assertEquals(1L, page.unread)
        assertEquals(3L, api.unread().unread)
        api.markRead("n1")
        api.markAllRead()
        assertEquals(
            listOf(
                "GET /api/notifications?page=2",
                "GET /api/notifications/unread",
                "POST /api/notifications/n1/read",
                "POST /api/notifications/read-all",
            ),
            seen,
        )
    }
}
