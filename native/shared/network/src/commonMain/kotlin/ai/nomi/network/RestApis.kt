package ai.nomi.network

import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.request.get
import io.ktor.client.request.parameter
import io.ktor.client.request.post
import io.ktor.client.request.setBody
import io.ktor.client.statement.HttpResponse
import io.ktor.http.ContentType
import io.ktor.http.contentType
import io.ktor.http.encodeURLPathPart
import io.ktor.http.isSuccess

/** Messages per page; matches the web client's `MESSAGE_PAGE_SIZE` (the server clamps to 1..100). */
const val MESSAGE_PAGE_SIZE = 50

private const val SESSIONS_PATH = "/api/sessions"
private const val NOTIFICATIONS_PATH = "/api/notifications"
private const val PARAM_LIMIT = "limit"
private const val PARAM_BEFORE = "before"
private const val PARAM_FILTER = "filter"
private const val PARAM_PAGE = "page"

private fun messagesPath(sessionId: String) = "$SESSIONS_PATH/${sessionId.encodeURLPathPart()}/messages"

/** Throws [ApiException] for any non-2xx status so callers never parse an error body as data. */
private suspend inline fun <reified T> HttpResponse.bodyOrThrow(what: String): T {
    ensureSuccess(what)
    return body()
}

private fun HttpResponse.ensureSuccess(what: String) {
    if (!status.isSuccess()) throw ApiException(status.value, "$what failed (${status.value})")
}

/** `/api/sessions`. */
interface SessionsApi {
    /** The person's chats, latest first. A chat with no message yet is not listed (unless it's a project chat). */
    suspend fun list(): List<SessionDto>

    /** Creates an empty chat and returns its id. */
    suspend fun create(): String
}

/** `/api/sessions/:id/messages[/:message_id]`. */
interface MessagesApi {
    /** A page ending just before [before] (or at the newest), oldest first. */
    suspend fun page(sessionId: String, before: String? = null, limit: Int = MESSAGE_PAGE_SIZE): MessagePageDto

    /** One message (the endpoint the web's `fetchRenderedMessage` uses). */
    suspend fun get(sessionId: String, messageId: String): MessageDto

    /** Posts the person's message; the reply arrives over realtime. */
    suspend fun send(sessionId: String, text: String): SendMessageResponseDto
}

/** `/api/notifications`. */
interface NotificationsApi {
    /** [filter] is `all` (server default), `unread`, `read` or `promo`; [page] starts at 1. */
    suspend fun list(filter: String? = null, page: Int? = null): NotificationPageDto

    suspend fun unread(): UnreadCountDto

    suspend fun markRead(id: String)

    suspend fun markAllRead()
}

class HttpSessionsApi(private val client: HttpClient, private val config: ApiConfig) : SessionsApi {
    override suspend fun list(): List<SessionDto> =
        client.get(config.url(SESSIONS_PATH)).bodyOrThrow<ListSessionsResponse>("list sessions").sessions

    override suspend fun create(): String =
        client.post(config.url(SESSIONS_PATH)).bodyOrThrow<CreateSessionResponse>("create session").sessionId
}

class HttpMessagesApi(private val client: HttpClient, private val config: ApiConfig) : MessagesApi {
    override suspend fun page(sessionId: String, before: String?, limit: Int): MessagePageDto =
        client.get(config.url(messagesPath(sessionId))) {
            parameter(PARAM_LIMIT, limit)
            if (before != null) parameter(PARAM_BEFORE, before)
        }.bodyOrThrow("list messages")

    override suspend fun get(sessionId: String, messageId: String): MessageDto =
        client.get(config.url("${messagesPath(sessionId)}/${messageId.encodeURLPathPart()}")).bodyOrThrow("fetch message")

    override suspend fun send(sessionId: String, text: String): SendMessageResponseDto =
        client.post(config.url(messagesPath(sessionId))) {
            contentType(ContentType.Application.Json)
            setBody(SendMessageRequest(text))
        }.bodyOrThrow("send message")
}

class HttpNotificationsApi(private val client: HttpClient, private val config: ApiConfig) : NotificationsApi {
    override suspend fun list(filter: String?, page: Int?): NotificationPageDto =
        client.get(config.url(NOTIFICATIONS_PATH)) {
            if (filter != null) parameter(PARAM_FILTER, filter)
            if (page != null) parameter(PARAM_PAGE, page)
        }.bodyOrThrow("list notifications")

    override suspend fun unread(): UnreadCountDto =
        client.get(config.url("$NOTIFICATIONS_PATH/unread")).bodyOrThrow("unread notifications")

    override suspend fun markRead(id: String) {
        client.post(config.url("$NOTIFICATIONS_PATH/${id.encodeURLPathPart()}/read")).ensureSuccess("mark notification read")
    }

    override suspend fun markAllRead() {
        client.post(config.url("$NOTIFICATIONS_PATH/read-all")).ensureSuccess("mark all notifications read")
    }
}
