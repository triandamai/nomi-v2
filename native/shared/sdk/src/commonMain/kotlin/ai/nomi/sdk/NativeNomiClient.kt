package ai.nomi.sdk

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.CoroutineStart
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.launch

/**
 * [NomiClient] backed by the Rust engine through UniFFI.
 *
 * Constructing it initializes the engine: it throws [NomiException.InvalidConfig] for a config the
 * engine rejects and [NomiException.Storage] when the data directory can't be created.
 * [credentials] is the platform's secure store for the session's tokens.
 */
class NativeNomiClient(config: SdkConfig, credentials: CredentialStore) : NomiClient {
    private val sdk = NomiSdk(config, credentials)

    /** Keeps [user] and [connection] in step with the engine's events until [close]. */
    private val scope = CoroutineScope(SupervisorJob() + Dispatchers.Default)

    private val mutableState = MutableStateFlow(sdk.state())
    private val mutableAccount = MutableStateFlow(sdk.currentAccount())
    private val mutableUser = MutableStateFlow(sdk.signedInUser())
    private val mutableConnection = MutableStateFlow(sdk.connectionState())

    override val state: StateFlow<SdkState> = mutableState.asStateFlow()

    override val account: StateFlow<AccountStore?> = mutableAccount.asStateFlow()

    override val user: StateFlow<SignedInUser?> = mutableUser.asStateFlow()

    override val connection: StateFlow<ConnectionState> = mutableConnection.asStateFlow()

    override val events: Flow<SdkEvent> = flow {
        val subscription = sdk.subscribe()
        try {
            while (true) {
                emit(subscription.nextEvent() ?: break)
            }
        } finally {
            subscription.cancel()
            subscription.destroy()
        }
    }

    init {
        // Undispatched: subscribed before the constructor returns, so no event is missed.
        scope.launch(start = CoroutineStart.UNDISPATCHED) {
            events.collect { event ->
                when (event) {
                    is SdkEvent.SignedIn, is SdkEvent.SignedOut -> mutableUser.value = sdk.signedInUser()
                    is SdkEvent.ConnectionChanged -> mutableConnection.value = event.state
                    is SdkEvent.AccountChanged -> mutableAccount.value = sdk.currentAccount()
                    else -> Unit
                }
            }
        }
    }

    override fun diagnostics(): Diagnostics = sdk.diagnostics()

    override suspend fun start() {
        try {
            sdk.start()
        } finally {
            mutableState.value = sdk.state()
        }
    }

    override suspend fun signIn(email: String, password: String): SignInResult =
        userChange { sdk.signIn(email, password) }

    override suspend fun verifyEmailCode(challengeId: String, code: String): SignedInUser =
        userChange { sdk.verifyEmailCode(challengeId, code) }

    override suspend fun resendEmailCode(challengeId: String): VerificationInfo = sdk.resendEmailCode(challengeId)

    override suspend fun refreshSession(): Boolean = userChange { sdk.refreshSession() }

    override suspend fun signOut() = userChange { accountChange { sdk.signOut() } }

    override suspend fun fetchProfile(): Profile = sdk.fetchProfile()

    override fun reconnect() = sdk.reconnect()

    override fun topicEvents(topic: String): Flow<SdkEvent> {
        // Validated eagerly, so a malformed topic fails at the call, not at collection.
        val wire = realtimeTopic(topic)
        return flow {
            val watch = sdk.watchTopic(wire)
            try {
                events.filter { it.topicOrNull() == wire }.collect { emit(it) }
            } finally {
                watch.release()
                watch.destroy()
            }
        }
    }

    override suspend fun openAccount(accountId: String): AccountStore = accountChange { sdk.openAccount(accountId) }

    override suspend fun closeAccount() = accountChange { sdk.closeAccount() }

    override suspend fun removeAccount(accountId: String) = accountChange { sdk.removeAccount(accountId) }

    override suspend fun resetStorage() = accountChange { sdk.resetStorage() }

    override suspend fun refreshSessions() = sdk.refreshSessions()

    override suspend fun createSession(): String = sdk.createSession()

    override fun syncChat(sessionId: String): Flow<Nothing> = flow {
        val binding = sdk.bindChat(sessionId)
        try {
            val accountId = account.value?.accountId()
            account.first { it?.accountId() != accountId }
        } finally {
            binding.release()
            binding.destroy()
        }
    }

    override suspend fun refreshMessages(sessionId: String) = sdk.refreshMessages(sessionId)

    override suspend fun loadOlderMessages(sessionId: String): Boolean = sdk.loadOlderMessages(sessionId)

    override suspend fun hasOlderMessages(sessionId: String): Boolean = sdk.hasOlderMessages(sessionId)

    override suspend fun sendMessage(sessionId: String, text: String): String = sdk.sendMessage(sessionId, text)

    override suspend fun resolveApproval(sessionId: String, messageId: String, approve: Boolean, remember: Boolean) =
        sdk.resolveApproval(sessionId, messageId, approve, remember)

    override suspend fun refreshNotifications() = sdk.refreshNotifications()

    override suspend fun loadMoreNotifications(): Boolean = sdk.loadMoreNotifications()

    override suspend fun markNotificationRead(notificationId: String) = sdk.markNotificationRead(notificationId)

    override suspend fun markAllNotificationsRead() = sdk.markAllNotificationsRead()

    override suspend fun retryOutbox(key: String): Boolean = sdk.retryOutbox(key)

    override suspend fun discardOutbox(key: String): Boolean = sdk.discardOutbox(key)

    override suspend fun syncNow() = sdk.syncNow()

    override fun setForegroundChat(sessionId: String?) = sdk.setForegroundChat(sessionId)

    override suspend fun close() {
        try {
            sdk.shutdown()
        } finally {
            mutableState.value = sdk.state()
            mutableAccount.value = sdk.currentAccount()
            mutableConnection.value = sdk.connectionState()
            scope.cancel()
        }
    }

    private suspend fun <T> accountChange(change: suspend () -> T): T = try {
        change()
    } finally {
        mutableAccount.value = sdk.currentAccount()
    }

    private suspend fun <T> userChange(change: suspend () -> T): T = try {
        change()
    } finally {
        mutableUser.value = sdk.signedInUser()
    }
}

/** The realtime topic an event is about, for the events that have one. */
private fun SdkEvent.topicOrNull(): String? = when (this) {
    is SdkEvent.Realtime -> topic
    is SdkEvent.TopicResynced -> topic
    is SdkEvent.TopicAccessLost -> topic
    else -> null
}
