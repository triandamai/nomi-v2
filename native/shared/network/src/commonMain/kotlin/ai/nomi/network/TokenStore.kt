package ai.nomi.network

import kotlinx.serialization.Serializable

@Serializable
data class Tokens(val access: String, val refresh: String)

/** Secure, platform-specific persistence of the signed-in session. Never log what it holds. */
interface TokenStore {
    suspend fun read(): Tokens?
    suspend fun write(tokens: Tokens)
    suspend fun clear()
}

internal const val TOKEN_SERVICE = "ai.nomi.app.tokens"
