package ai.nomi.realtime

/** A realtime subscription scope, spelled on the wire as `user`, `chat:<id>` or `project:<id>`. */
sealed interface Topic {
    val wire: String

    data object User : Topic {
        override val wire: String = WIRE_USER
    }

    /** Ids are UUIDs, so they compare case-insensitively: the id is kept lowercased, as the gateway spells it. */
    class Chat(sessionId: String) : Topic {
        val sessionId: String = sessionId.lowercase()
        override val wire: String get() = "$PREFIX_CHAT$sessionId"
        override fun equals(other: Any?): Boolean = other is Chat && other.sessionId == sessionId
        override fun hashCode(): Int = sessionId.hashCode()
        override fun toString(): String = "Chat(sessionId=$sessionId)"
    }

    /** See [Chat]: the id is kept lowercased. */
    class Project(projectId: String) : Topic {
        val projectId: String = projectId.lowercase()
        override val wire: String get() = "$PREFIX_PROJECT$projectId"
        override fun equals(other: Any?): Boolean = other is Project && other.projectId == projectId
        override fun hashCode(): Int = projectId.hashCode()
        override fun toString(): String = "Project(projectId=$projectId)"
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
