package ai.nomi.realtime

/** A realtime subscription scope, spelled on the wire as `user`, `chat:<id>` or `project:<id>`. */
sealed interface Topic {
    val wire: String

    data object User : Topic {
        override val wire: String = WIRE_USER
    }

    data class Chat(val sessionId: String) : Topic {
        override val wire: String get() = "$PREFIX_CHAT$sessionId"
    }

    data class Project(val projectId: String) : Topic {
        override val wire: String get() = "$PREFIX_PROJECT$projectId"
    }

    companion object {
        private const val WIRE_USER = "user"
        private const val PREFIX_CHAT = "chat:"
        private const val PREFIX_PROJECT = "project:"

        fun parse(wire: String): Topic? = when {
            wire == WIRE_USER -> User
            wire.startsWith(PREFIX_CHAT) -> wire.removePrefix(PREFIX_CHAT).takeIf { it.isNotEmpty() }?.let(::Chat)
            wire.startsWith(PREFIX_PROJECT) -> wire.removePrefix(PREFIX_PROJECT).takeIf { it.isNotEmpty() }?.let(::Project)
            else -> null
        }
    }
}
