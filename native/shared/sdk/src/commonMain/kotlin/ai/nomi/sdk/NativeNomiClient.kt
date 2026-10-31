package ai.nomi.sdk

import kotlinx.coroutines.CoroutineExceptionHandler
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

    /** Keeps [user] and [connection] in step with the engine's events until [close]; a failure there is logged, not fatal. */
    private val scope = CoroutineScope(
        SupervisorJob() + Dispatchers.Default + CoroutineExceptionHandler { _, failure ->
            println("Nomi: engine events failed: ${failure.stackTraceToString()}")
        },
    )

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

    override suspend fun rateMessage(sessionId: String, messageId: String, rating: String?, reasons: List<String>, note: String?, includeChat: Boolean) =
        sdk.rateMessage(sessionId, messageId, rating, reasons, note, includeChat)

    override suspend fun deleteMessage(sessionId: String, messageId: String) = sdk.deleteMessage(sessionId, messageId)

    override suspend fun regenerateMessage(sessionId: String, messageId: String, style: String) =
        sdk.regenerateMessage(sessionId, messageId, style)

    override suspend fun shareReply(sessionId: String, messageId: String): ShareLink = sdk.shareReply(sessionId, messageId)

    override suspend fun refreshNotifications() = sdk.refreshNotifications()

    override suspend fun loadMoreNotifications(): Boolean = sdk.loadMoreNotifications()

    override suspend fun markNotificationRead(notificationId: String) = sdk.markNotificationRead(notificationId)

    override suspend fun markAllNotificationsRead() = sdk.markAllNotificationsRead()

    override suspend fun retryOutbox(key: String): Boolean = sdk.retryOutbox(key)

    override suspend fun discardOutbox(key: String): Boolean = sdk.discardOutbox(key)

    override suspend fun syncNow() = sdk.syncNow()

    override fun setForegroundChat(sessionId: String?) = sdk.setForegroundChat(sessionId)

    override suspend fun crew(): List<CrewMember> = sdk.crew()

    override suspend fun preferences(): Preferences = sdk.preferences()

    override suspend fun updatePreferences(update: PreferencesUpdate): Preferences = sdk.updatePreferences(update)

    override suspend fun homeSummary(): HomeSummary = sdk.homeSummary()

    override suspend fun moneySummary(month: String?): MoneySummary = sdk.moneySummary(month)

    override suspend fun addTransaction(transaction: NewTransaction) = sdk.addTransaction(transaction)

    override suspend fun setBudget(category: String, limitCents: Long) = sdk.setBudget(category, limitCents)

    override suspend fun reminders(): RemindersOverview = sdk.reminders()

    override suspend fun createReminder(reminder: NewReminder): Reminder = sdk.createReminder(reminder)

    override suspend fun actOnReminder(id: String, action: ReminderAction) = sdk.actOnReminder(id, action)

    override suspend fun cancelScheduledTask(id: String) = sdk.cancelScheduledTask(id)

    override suspend fun memories(): List<Memory> = sdk.memories()

    override suspend fun editMemory(id: String, content: String, kind: String?) = sdk.editMemory(id, content, kind)

    override suspend fun confirmMemory(id: String) = sdk.confirmMemory(id)

    override suspend fun deleteMemory(id: String) = sdk.deleteMemory(id)

    override suspend fun connections(): ConnectionsOverview = sdk.connections()

    override suspend fun usageBrief(): UsageBrief = sdk.usageBrief()

    override suspend fun projects(orgId: String?): List<Project> = sdk.projects(orgId)

    override suspend fun projectForChat(sessionId: String): ProjectDetail = sdk.projectForChat(sessionId)

    override suspend fun projectFile(projectId: String, path: String): ByteArray = sdk.projectFile(projectId, path)

    override suspend fun projectDeploy(projectId: String): DeployOverview = sdk.projectDeploy(projectId)

    override suspend fun publishProject(projectId: String) = sdk.publishProject(projectId)

    override suspend fun projectIssues(projectId: String, status: String): ProjectIssues = sdk.projectIssues(projectId, status)

    override suspend fun fileProjectIssue(projectId: String, issue: NewProjectIssue): ProjectIssue =
        sdk.fileProjectIssue(projectId, issue)

    override suspend fun setProjectIssueStatus(projectId: String, number: Int, status: String): ProjectIssue =
        sdk.setProjectIssueStatus(projectId, number, status)

    override suspend fun projectMembers(projectId: String): ProjectMembers = sdk.projectMembers(projectId)

    override suspend fun projectDomain(projectId: String): ProjectDomain = sdk.projectDomain(projectId)

    override suspend fun checkProjectDomain(projectId: String, subdomain: String): DomainCheck = sdk.checkProjectDomain(projectId, subdomain)

    override suspend fun changeProjectDomain(projectId: String, subdomain: String): DomainChange = sdk.changeProjectDomain(projectId, subdomain)

    override suspend fun projectUsage(projectId: String): List<ContainerUsage> = sdk.projectUsage(projectId)

    override suspend fun projectTraffic(projectId: String, range: String): ProjectTraffic = sdk.projectTraffic(projectId, range)

    override suspend fun projectLogs(projectId: String): ProjectLogs = sdk.projectLogs(projectId)

    override suspend fun takeProjectOffline(projectId: String) = sdk.takeProjectOffline(projectId)

    override suspend fun projectTables(projectId: String, source: DataSource): List<DatabaseTable> = sdk.projectTables(projectId, source)

    override suspend fun queryProjectData(projectId: String, sql: String, source: DataSource, relay: Boolean): QueryResult = sdk.queryProjectData(projectId, sql, source, relay)

    override suspend fun projectRelay(projectId: String, source: DataSource): RelayOverview = sdk.projectRelay(projectId, source)

    override suspend fun retryProjectRelay(projectId: String, source: DataSource) = sdk.retryProjectRelay(projectId, source)

    override suspend fun projectShowcase(projectId: String): ProjectShowcase = sdk.projectShowcase(projectId)

    override suspend fun setProjectShowcase(projectId: String, showcase: Boolean, blurb: String?): ProjectShowcase = sdk.setProjectShowcase(projectId, showcase, blurb)

    override suspend fun transferOrgs(projectId: String, query: String): List<TransferOrg> = sdk.transferOrgs(projectId, query)

    override suspend fun transferProject(projectId: String, orgId: String, confirmName: String) = sdk.transferProject(projectId, orgId, confirmName)

    override suspend fun deleteProject(projectId: String) = sdk.deleteProject(projectId)

    override suspend fun attachmentPreview(id: String): ByteArray = sdk.attachmentPreview(id)

    override suspend fun attachmentContent(id: String): ByteArray = sdk.attachmentContent(id)

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
