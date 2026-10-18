package ai.nomi.network

import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.request.get
import io.ktor.http.isSuccess
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

private const val PROFILE_PATH = "/api/profile"
private const val CREW_PATH = "/api/agents"

/** `ProfileResponse` in routes/profile.rs. */
@Serializable
data class ProfileDto(
    @SerialName("display_name") val displayName: String? = null,
    val username: String? = null,
    val email: String,
    @SerialName("avatar_url") val avatarUrl: String? = null,
)

/**
 * One crew member (`CrewMember` in routes/agents.rs). [state] is "working", "waiting" (on the person), "done" (in the
 * last day) or "idle", with [status] the line to go with it. [shape]/[tone]/[motion] are set for dynamic agents.
 */
@Serializable
data class CrewMemberDto(
    @SerialName("agent_type") val agentType: String,
    val name: String,
    val role: String,
    @SerialName("is_dynamic") val isDynamic: Boolean = false,
    val shape: String? = null,
    val tone: String? = null,
    val motion: String? = null,
    val state: String,
    val status: String,
)

@Serializable
internal data class CrewResponse(val agents: List<CrewMemberDto>)

/** `/api/profile` and `/api/agents`: who the person is and their crew, for Home. */
interface HomeApi {
    suspend fun profile(): ProfileDto
    suspend fun crew(): List<CrewMemberDto>
}

class HttpHomeApi(private val client: HttpClient, private val config: ApiConfig) : HomeApi {
    override suspend fun profile(): ProfileDto = get(PROFILE_PATH, "profile")

    override suspend fun crew(): List<CrewMemberDto> = get<CrewResponse>(CREW_PATH, "crew").agents

    private suspend inline fun <reified T> get(path: String, what: String): T {
        val response = client.get(config.url(path))
        if (!response.status.isSuccess()) throw ApiException(response.status.value, "$what failed (${response.status.value})")
        return response.body()
    }
}
