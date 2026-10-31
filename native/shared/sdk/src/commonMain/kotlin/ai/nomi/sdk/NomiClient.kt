package ai.nomi.sdk

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.StateFlow

/**
 * The app's handle on the Rust engine: one per process, owned by the app, never by a screen.
 *
 * The models ([SdkConfig], [SdkEvent], [SdkState], [Diagnostics], [NomiException], [AccountStore]
 * and the stored models) are the SDK's public types, generated from `rust/crates/nomi-sdk`.
 */
interface NomiClient {
    /**
     * Every [SdkEvent] sent while collecting. Each collector gets its own stream, which completes
     * once the client is closed.
     */
    val events: Flow<SdkEvent>

    /** Where the engine is in its lifecycle; updated as [start] and [close] return. */
    val state: StateFlow<SdkState>

    /** Who is signed in; set from the stored session at construction, then by sign-in and sign-out. */
    val user: StateFlow<SignedInUser?>

    /** The realtime socket's state; it runs while the client is started and someone is signed in. */
    val connection: StateFlow<ConnectionState>

    /** The open account's store; null until [openAccount], and after it is closed or removed. */
    val account: StateFlow<AccountStore?>

    fun diagnostics(): Diagnostics

    /** Starts background work. A no-op while running; throws [NomiException.ShutDown] once closed. */
    suspend fun start()

    /**
     * Signs in with email and password. The backend may ask for an emailed code instead
     * ([SignInResult.EmailCodeRequired]); finish with [verifyEmailCode]. Throws
     * [NomiException.Unauthorized] for wrong credentials, [NomiException.Network] when offline.
     */
    suspend fun signIn(email: String, password: String): SignInResult

    /** Throws [NomiException.EmailCode] when the code is wrong, expired or used up. */
    suspend fun verifyEmailCode(challengeId: String, code: String): SignedInUser

    suspend fun resendEmailCode(challengeId: String): VerificationInfo

    /** False when there is no session or the backend refused it (the session has then ended). */
    suspend fun refreshSession(): Boolean

    /**
     * Forgets the session, closes the socket and the open account (its data stays until
     * [removeAccount]), and revokes the session on the backend, best effort.
     */
    suspend fun signOut()

    suspend fun fetchProfile(): Profile

    /** Skips any backoff wait and reconnects the socket now (e.g. back online). */
    fun reconnect()

    /**
     * Events for one realtime topic (`chat:<id>`, `project:<id>`, `user`): [SdkEvent.Realtime],
     * [SdkEvent.TopicResynced] and [SdkEvent.TopicAccessLost]. The topic is watched while collected.
     * Throws [NomiException.InvalidArgument] for a malformed topic.
     */
    fun topicEvents(topic: String): Flow<SdkEvent>

    /**
     * The local store for [accountId] (the signed-in user's id), opening and migrating its database
     * if needed. Opening another account closes the current one: its store throws
     * [NomiException.AccountClosed] and its flows complete.
     */
    suspend fun openAccount(accountId: String): AccountStore

    /** Closes the open account, keeping its data. */
    suspend fun closeAccount()

    /** Deletes everything stored for [accountId] (sign-out), closing it first if it is open. */
    suspend fun removeAccount(accountId: String)

    /** Deletes every account's local data. */
    suspend fun resetStorage()

    // Sync: the open account's data, kept in step with the backend. Every call throws
    // [NomiException.AccountClosed] while no account is open.

    /** Replaces the chat list with the server's. */
    suspend fun refreshSessions()

    /** Creates an empty chat and returns its id; the server lists it once a message is sent. */
    suspend fun createSession(): String

    /**
     * Keeps a chat in sync while collected (its screen is open): watches its topic, applies its
     * events, and loads its newest page. Completes when the account closes or switches.
     */
    fun syncChat(sessionId: String): Flow<Nothing>

    /** Loads a chat's newest page. */
    suspend fun refreshMessages(sessionId: String)

    /** Loads the page before the oldest stored message; returns whether older ones still exist. */
    suspend fun loadOlderMessages(sessionId: String): Boolean

    /** Whether older messages may exist on the server; true until a load says otherwise. */
    suspend fun hasOlderMessages(sessionId: String): Boolean

    /**
     * Queues the person's message and returns its local id. It shows at once with status
     * `sending`, goes out in order with the chat's other queued messages, is retried until the
     * backend has it (across restarts), and is then replaced by the server's copy. Throws
     * [NomiException.InvalidArgument] for empty or over-long text.
     */
    suspend fun sendMessage(sessionId: String, text: String): String

    /** Approves or denies a pending approval card; [NomiException.Http] 409 when no longer pending. */
    suspend fun resolveApproval(sessionId: String, messageId: String, approve: Boolean, remember: Boolean)

    /** Refetches the inbox's first page. */
    suspend fun refreshNotifications()

    /** Loads the next inbox page; returns whether more pages exist. */
    suspend fun loadMoreNotifications(): Boolean

    /** Marks a notification read now, and on the backend through the outbox. */
    suspend fun markNotificationRead(notificationId: String)

    /** Marks every notification read now, and on the backend through the outbox. */
    suspend fun markAllNotificationsRead()

    /** Puts a failed outbox entry back in line; false when [key] names no failed entry. */
    suspend fun retryOutbox(key: String): Boolean

    /** Drops a pending or failed outbox entry and the message it showed; false when [key] names neither. */
    suspend fun discardOutbox(key: String): Boolean

    /** Refetches the chat list and the inbox and sends what the outbox holds (e.g. back in the foreground). */
    suspend fun syncNow()

    /** The chat on screen, or null: a reply there doesn't mark it unread. */
    fun setForegroundChat(sessionId: String?)

    // Account pages: online calls, nothing stored. Throws [NomiException.NotSignedIn] without a session.

    /** The person's crew and what each member is doing. */
    suspend fun crew(): List<CrewMember>

    /** Appearance and layout preferences. */
    suspend fun preferences(): Preferences

    /** Changes the preferences set in [update]; returns them all. */
    suspend fun updatePreferences(update: PreferencesUpdate): Preferences

    /** The home page's status: what happened while away, today, plans. */
    suspend fun homeSummary(): HomeSummary

    /** The money month starting in [month] (`YYYY-MM`), or the current one. */
    suspend fun moneySummary(month: String?): MoneySummary

    /** Adds a transaction; amounts are in cents. */
    suspend fun addTransaction(transaction: NewTransaction)

    /** Sets a category's monthly budget. */
    suspend fun setBudget(category: String, limitCents: Long)

    /** Upcoming and past reminders, and agent work scheduled for later. */
    suspend fun reminders(): RemindersOverview

    /** Creates a reminder. */
    suspend fun createReminder(reminder: NewReminder): Reminder

    /** Marks a reminder done, cancels it, or snoozes it. */
    suspend fun actOnReminder(id: String, action: ReminderAction)

    /** Cancels agent work scheduled for later. */
    suspend fun cancelScheduledTask(id: String)

    /** What Nomi remembers about the person. */
    suspend fun memories(): List<Memory>

    /** Rewrites a memory; [kind] changes its kind too. */
    suspend fun editMemory(id: String, content: String, kind: String?)

    /** Confirms a memory is still true. */
    suspend fun confirmMemory(id: String)

    /** Forgets a memory. */
    suspend fun deleteMemory(id: String)

    /** The person's connected services. */
    suspend fun connections(): ConnectionsOverview

    /** This month's allowance and how much is used. */
    suspend fun usageBrief(): UsageBrief

    /** Projects in [orgId], or in the organization picked for projects. */
    suspend fun projects(orgId: String?): List<Project>

    /** The project whose chat is [sessionId]; [NomiException.Http] 404 for an ordinary chat. */
    suspend fun projectForChat(sessionId: String): ProjectDetail

    /** A project file's content, as stored. */
    suspend fun projectFile(projectId: String, path: String): ByteArray

    /** Where the project is published, and its publishes. */
    suspend fun projectDeploy(projectId: String): DeployOverview

    /** Publishes the project to production. */
    suspend fun publishProject(projectId: String)

    /** The project's issues with [status] `open`, `closed` or `all`. */
    suspend fun projectIssues(projectId: String, status: String): ProjectIssues

    /** Files an issue on the project. */
    suspend fun fileProjectIssue(projectId: String, issue: NewProjectIssue): ProjectIssue

    /** Closes (`closed`) or reopens (`open`) a project issue. */
    suspend fun setProjectIssueStatus(projectId: String, number: Int, status: String): ProjectIssue

    /** Who works on the project. */
    suspend fun projectMembers(projectId: String): ProjectMembers

    /** A small preview of an attached image, as the encoded image. */
    suspend fun attachmentPreview(id: String): ByteArray

    /** An attached file's content as uploaded. */
    suspend fun attachmentContent(id: String): ByteArray

    /** Stops background work for good and completes every [events] stream. A no-op once closed. */
    suspend fun close()
}
