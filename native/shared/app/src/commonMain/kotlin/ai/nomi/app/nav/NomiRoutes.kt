package ai.nomi.app.nav

import androidx.navigation3.runtime.NavKey
import kotlinx.serialization.Serializable

private const val SCHEME_SEPARATOR = "://"
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
         * [Notifications], any other path → [Home]. Accepts a bare path or a full URL; null for an empty link.
         */
        fun fromLink(link: String): NomiRoute? {
            val trimmed = link.trim()
            if (trimmed.isEmpty()) return null
            val path = if (SCHEME_SEPARATOR in trimmed) {
                trimmed.substringAfter(SCHEME_SEPARATOR).substringAfter(PATH_SEPARATOR, missingDelimiterValue = "")
            } else {
                trimmed
            }
            val segments = path.substringBefore('?').substringBefore('#').split(PATH_SEPARATOR).filter { it.isNotEmpty() }
            return when {
                segments.size == CHAT_PATH_SEGMENTS && segments.first() == SEGMENT_CHAT -> Chat(segments[CHAT_ID_SEGMENT])
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
