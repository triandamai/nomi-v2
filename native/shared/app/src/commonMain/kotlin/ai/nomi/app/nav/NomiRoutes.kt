package ai.nomi.app.nav

import androidx.navigation3.runtime.NavKey
import io.ktor.http.decodeURLPart
import kotlinx.serialization.Serializable

private const val SCHEME_SEPARATOR = "://"

/** Web links carry a host before the path; the app's own scheme (`nomi://chat/abc`) starts the path right away. */
private val WEB_SCHEMES = setOf("http", "https")
private const val PATH_SEPARATOR = '/'
private const val SEGMENT_CHAT = "chat"
private const val SEGMENT_NOTIFICATIONS = "notifications"
private const val CHAT_PATH_SEGMENTS = 2
private const val CHAT_ID_SEGMENT = 1

/** Every screen the shell can show. Home is always the root of a signed-in back stack. */
@Serializable
sealed interface NomiRoute : NavKey {
    companion object {
        /**
         * Where an in-app link goes (island click, bubble tap, push tap): `/chat/{id}` → [Chat], `/notifications` →
         * [Notifications], any other path → [Home]. Accepts a bare path, a web URL or a `nomi://` link; ids are
         * percent-decoded. Null for an empty link.
         */
        fun fromLink(link: String): NomiRoute? {
            val trimmed = link.trim()
            if (trimmed.isEmpty()) return null
            val path = if (SCHEME_SEPARATOR in trimmed) {
                val scheme = trimmed.substringBefore(SCHEME_SEPARATOR).lowercase()
                val rest = trimmed.substringAfter(SCHEME_SEPARATOR)
                if (scheme in WEB_SCHEMES) rest.substringAfter(PATH_SEPARATOR, missingDelimiterValue = "") else rest
            } else {
                trimmed
            }
            // Split before decoding, so an encoded '/' inside an id stays part of the id.
            val segments = path.substringBefore('?').substringBefore('#').split(PATH_SEPARATOR).filter { it.isNotEmpty() }
            return when {
                segments.size == CHAT_PATH_SEGMENTS && segments.first() == SEGMENT_CHAT ->
                    Chat(segments[CHAT_ID_SEGMENT].decodeURLPart())
                segments.firstOrNull() == SEGMENT_NOTIFICATIONS -> Notifications
                else -> Home
            }
        }
    }
}

@Serializable data object SignIn : NomiRoute

@Serializable data object Home : NomiRoute

@Serializable data object Chats : NomiRoute

@Serializable data class Chat(val sessionId: String) : NomiRoute

@Serializable data object Notifications : NomiRoute

@Serializable data object Settings : NomiRoute

/** The destinations of the navigation bar / rail, in order. */
val TopLevelRoutes: List<NomiRoute> = listOf(Home, Chats, Notifications, Settings)
