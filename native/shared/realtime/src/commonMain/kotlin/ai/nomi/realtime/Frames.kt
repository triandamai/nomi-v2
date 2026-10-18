@file:OptIn(ExperimentalSerializationApi::class)

package ai.nomi.realtime

import kotlinx.serialization.ExperimentalSerializationApi
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonClassDiscriminator
import kotlinx.serialization.json.JsonObject

/** The one Json configuration for everything on the realtime wire. */
val WireJson: Json = Json {
    ignoreUnknownKeys = true
}

/** Frames the app sends to the gateway (Rust `ClientFrame`, tag `op`). */
@Serializable
@JsonClassDiscriminator("op")
sealed interface ClientFrame {
    @Serializable
    @SerialName("subscribe")
    data class Subscribe(val topic: String) : ClientFrame

    @Serializable
    @SerialName("unsubscribe")
    data class Unsubscribe(val topic: String) : ClientFrame

    @Serializable
    @SerialName("ping")
    data object Ping : ClientFrame
}

/** Frames the gateway sends to the app (Rust `ServerFrame`, tag `type`). */
@Serializable
@JsonClassDiscriminator("type")
sealed interface ServerFrame {
    @Serializable
    @SerialName("ready")
    data class Ready(@SerialName("user_id") val userId: String) : ServerFrame

    @Serializable
    @SerialName("subscribed")
    data class Subscribed(val topic: String) : ServerFrame

    @Serializable
    @SerialName("unsubscribed")
    data class Unsubscribed(val topic: String) : ServerFrame

    @Serializable
    @SerialName("denied")
    data class Denied(val topic: String) : ServerFrame

    @Serializable
    @SerialName("event")
    data class Event(val topic: String, val payload: JsonObject) : ServerFrame

    @Serializable
    @SerialName("error")
    data class Error(val message: String) : ServerFrame

    @Serializable
    @SerialName("pong")
    data object Pong : ServerFrame
}
