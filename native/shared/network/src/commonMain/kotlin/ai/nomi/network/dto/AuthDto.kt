package ai.nomi.network.dto

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

/** Body of `/api/auth/refresh`, which the client's own 401 handling calls. */
@Serializable
internal data class RefreshRequest(@SerialName("refresh_token") val refreshToken: String)

@Serializable
internal data class AccessTokenResponse(@SerialName("access_token") val accessToken: String)
