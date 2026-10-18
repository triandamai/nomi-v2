package ai.nomi.network

import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.request.get
import io.ktor.client.request.put
import io.ktor.client.request.setBody
import io.ktor.http.ContentType
import io.ktor.http.contentType
import io.ktor.http.isSuccess
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

private const val PROFILE_PATH = "/api/profile"
private const val CREW_PATH = "/api/agents"
private const val PREFERENCES_PATH = "/api/preferences"

/** `ProfileResponse` in routes/profile.rs. */
@Serializable
data class ProfileDto(
    @SerialName("display_name") val displayName: String? = null,
    val username: String? = null,
    val email: String,
    @SerialName("avatar_url") val avatarUrl: String? = null,
)

/** Subset of `PreferencesResponse` in routes/profile.rs used by the native shell. */
@Serializable
data class PreferencesDto(
    val theme: String = "system",
    @SerialName("accent_color") val accentColor: String = "canopy",
)

/** `UpdatePreferencesRequest` in routes/profile.rs. */
@Serializable
data class UpdatePreferencesDto(
    val theme: String? = null,
    @SerialName("accent_color") val accentColor: String? = null,
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

/** `/api/profile`, `/api/agents` and `/api/preferences`: who the person is, their crew, and their appearance choices. */
interface HomeApi {
    suspend fun profile(): ProfileDto
    suspend fun crew(): List<CrewMemberDto>
    suspend fun preferences(): PreferencesDto = PreferencesDto()
    suspend fun updatePreferences(request: UpdatePreferencesDto): PreferencesDto = PreferencesDto(
        theme = request.theme ?: "system",
        accentColor = request.accentColor ?: "canopy",
    )
}

class HttpHomeApi(private val client: HttpClient, private val config: ApiConfig) : HomeApi {
    override suspend fun profile(): ProfileDto = get(PROFILE_PATH, "profile")

    override suspend fun crew(): List<CrewMemberDto> = get<CrewResponse>(CREW_PATH, "crew").agents

    override suspend fun preferences(): PreferencesDto = get(PREFERENCES_PATH, "preferences")

    override suspend fun updatePreferences(request: UpdatePreferencesDto): PreferencesDto {
        val response = client.put(config.url(PREFERENCES_PATH)) {
            contentType(ContentType.Application.Json)
            setBody(request)
        }
        if (!response.status.isSuccess()) {
            throw ApiException(response.status.value, "update preferences failed (${response.status.value})")
        }
        return response.body()
    }

    private suspend inline fun <reified T> get(path: String, what: String): T {
        val response = client.get(config.url(path))
        if (!response.status.isSuccess()) throw ApiException(response.status.value, "$what failed (${response.status.value})")
        return response.body()
    }
}
