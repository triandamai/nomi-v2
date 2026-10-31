package ai.nomi.sdk

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flow

// Room-style flows over AccountStore's observations: the current answer first, then a new one after
// each change to it. Each collector gets its own observation; a flow completes when the account
// store closes (sign-out, account switch, shutdown).

/** The chat list, newest first. */
fun AccountStore.sessions(filter: SessionFilter): Flow<List<Session>> =
    observe({ observeSessions(filter) }, { next() }, { cancel() })

/** The newest [limit] messages of a chat, oldest first. Raise [limit] to load older ones. */
fun AccountStore.messages(sessionId: String, limit: UInt): Flow<List<Message>> =
    observe({ observeMessages(sessionId, limit) }, { next() }, { cancel() })

/** The chat's streaming draft, or null while there is none. */
fun AccountStore.draft(sessionId: String): Flow<Draft?> =
    observe({ observeDraft(sessionId) }, { next()?.let { Present(it.draft) } }, { cancel() }).unwrapPresent()

fun AccountStore.notifications(limit: UInt): Flow<List<NotificationItem>> =
    observe({ observeNotifications(limit) }, { next() }, { cancel() })

fun AccountStore.unreadNotificationCount(): Flow<UInt> =
    observe({ observeUnreadNotificationCount() }, { next() }, { cancel() })

/** Agents doing something, in every chat or only in [sessionId]. */
fun AccountStore.activeAgents(sessionId: String? = null): Flow<List<AgentActivity>> =
    observe({ observeActiveAgents(sessionId) }, { next() }, { cancel() })

/** Pulls [next] until it returns null; cancels and frees the observation when collection stops. */
private fun <O : Disposable, T : Any> observe(
    open: () -> O,
    next: suspend O.() -> T?,
    cancel: O.() -> Unit,
): Flow<T> = flow {
    val observation = open()
    try {
        while (true) {
            emit(observation.next() ?: break)
        }
    } finally {
        observation.cancel()
        observation.destroy()
    }
}

/** A nullable answer boxed so it can travel through [observe], where null means "ended". */
private class Present<T>(val value: T)

private fun <T> Flow<Present<T>>.unwrapPresent(): Flow<T> = flow { collect { emit(it.value) } }
