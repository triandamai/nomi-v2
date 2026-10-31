package ai.nomi.network

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

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
    val timezone: String = "UTC",
    @SerialName("home_layout") val homeLayout: String? = null,
    @SerialName("drawer_pins") val drawerPins: List<String>? = null,
    @SerialName("drawer_recents") val drawerRecents: Boolean = true,
    @SerialName("drawer_recents_limit") val drawerRecentsLimit: Int = 10,
)

/** `UpdatePreferencesRequest` in routes/profile.rs. */
@Serializable
data class UpdatePreferencesDto(
    val theme: String? = null,
    @SerialName("accent_color") val accentColor: String? = null,
    val timezone: String? = null,
    @SerialName("home_layout") val homeLayout: String? = null,
    @SerialName("drawer_pins") val drawerPins: List<String>? = null,
    @SerialName("drawer_recents") val drawerRecents: Boolean? = null,
    @SerialName("drawer_recents_limit") val drawerRecentsLimit: Int? = null,
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
data class OutItemDto(
    val kind: String,
    val agent: String,
    val title: String,
    val detail: String,
    @SerialName("session_id") val sessionId: String? = null,
    val at: String,
)

@Serializable
data class TodayItemDto(
    val id: String,
    @SerialName("run_at") val runAt: String,
    val label: String,
    val agent: String,
    val recurrence: String? = null,
)

@Serializable
data class PlanItemDto(
    val kind: String,
    val title: String,
    val agent: String,
    val done: Long,
    val total: Long,
    @SerialName("session_id") val sessionId: String,
    @SerialName("updated_at") val updatedAt: String,
)

@Serializable
data class HomeSummaryDto(
    val since: String = "",
    val timezone: String = "UTC",
    @SerialName("while_you_were_out") val whileYouWereOut: List<OutItemDto> = emptyList(),
    val today: List<TodayItemDto> = emptyList(),
    val plans: List<PlanItemDto> = emptyList(),
    @SerialName("while_you_were_out_total") val whileYouWereOutTotal: Int = 0,
    @SerialName("today_total") val todayTotal: Int = 0,
    @SerialName("plans_total") val plansTotal: Int = 0,
)

/** `/api/profile`, `/api/agents`, `/api/preferences` and `/api/home`: who the person is, their crew, appearance, and status. */
interface HomeApi {
    suspend fun profile(): ProfileDto
    suspend fun crew(): List<CrewMemberDto>
    suspend fun preferences(): PreferencesDto = PreferencesDto()
    suspend fun updatePreferences(request: UpdatePreferencesDto): PreferencesDto = PreferencesDto(
        theme = request.theme ?: "system",
        accentColor = request.accentColor ?: "canopy",
    )
    suspend fun summary(): HomeSummaryDto = HomeSummaryDto()
}

