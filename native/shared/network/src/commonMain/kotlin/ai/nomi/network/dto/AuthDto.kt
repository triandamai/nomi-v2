package ai.nomi.network.dto

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

@Serializable
internal data class LoginRequest(val email: String, val password: String)

/** Body of `/api/auth/refresh` and `/api/auth/logout`. */
@Serializable
internal data class RefreshRequest(@SerialName("refresh_token") val refreshToken: String)

@Serializable
internal data class AccessTokenResponse(@SerialName("access_token") val accessToken: String)

@Serializable
internal data class TokenPairResponse(
    @SerialName("access_token") val accessToken: String,
    @SerialName("refresh_token") val refreshToken: String,
)

/** Body of `/api/auth/verify`. */
@Serializable
internal data class VerifyRequest(@SerialName("challenge_id") val challengeId: String, val code: String)

/** Body of `/api/auth/resend`. */
@Serializable
internal data class ChallengeRequest(@SerialName("challenge_id") val challengeId: String)
