package ai.nomi.app.screens

import ai.nomi.app.screens.projects.Loadable
import ai.nomi.app.screens.projects.OpenFile
import ai.nomi.app.screens.projects.PROJECT_TAB_CHAT
import ai.nomi.app.screens.projects.PROJECT_TAB_CODE
import ai.nomi.app.screens.projects.PROJECT_TAB_PREVIEW
import ai.nomi.app.screens.projects.ProjectWorkspaceContent
import ai.nomi.data.AgentActivity
import ai.nomi.data.ProjectDeployInfo
import ai.nomi.data.ProjectFileEntry
import ai.nomi.data.ProjectInfo
import ai.nomi.data.ProjectPublishInfo
import ai.nomi.data.ChatState
import ai.nomi.data.Draft
import ai.nomi.data.MESSAGE_STATUS_FAILED
import ai.nomi.data.MESSAGE_STATUS_SENDING
import ai.nomi.data.MESSAGE_STATUS_SENT
import ai.nomi.data.Message
import ai.nomi.data.NotificationItem
import ai.nomi.feature.chat.ChatComposerState
import ai.nomi.feature.chat.ChatContent
import ai.nomi.feature.chat.ReplyFeedbackState
import ai.nomi.feature.chats.ChatsContent
import ai.nomi.feature.notifications.NotificationsContent
import ai.nomi.feature.settings.SettingsContent
import ai.nomi.network.ProfileDto
import androidx.compose.ui.test.junit4.createComposeRule
import kotlinx.datetime.TimeZone
import org.junit.Rule
import kotlin.test.Test

private const val MINUTE = 60_000L

private fun msg(id: String, role: String, text: String, minutesAgo: Long, agent: String? = null, name: String? = null) =
    Message(id, "s1", role, agent, name, text, null, SNAPSHOT_NOW - minutesAgo * MINUTE, "done")

private val Conversation = ChatState(
    messages = listOf(
        msg("m1", "user", "Can you plan my Saturday? Farmers market in the morning, and a hike if the weather holds.", 12),
        msg("m2", "assistant", "Happy to. The market opens at 8 and is quietest before 9, so I'd go early. Rain is likely after 2pm, so the hike fits best around 11.", 11, "chitchat", "Nomi"),
        msg("m3", "assistant", "I've also put a reminder for sunscreen at 10:30.", 11, "chitchat", "Nomi"),
        msg("m4", "user", "Perfect. What did I spend at the market last month?", 2),
    ),
    draft = Draft("t1", "You spent €64 across three visits — mostly at the bakery stall", "writing_reply", SNAPSHOT_NOW),
    activeAgent = AgentActivity("a1", "s1", "money", "writing_reply", null, SNAPSHOT_NOW),
)

private val Inbox = listOf(
    NotificationItem("n1", "quota", "You've used 80% of this month's tokens", "Upgrade or wait until the 1st for more.", "/billing", SNAPSHOT_NOW - 30 * MINUTE, null),
    NotificationItem("n2", "deploy", "portfolio-site is live", "Published to portfolio.nomi.app.", null, SNAPSHOT_NOW - 5 * 60 * MINUTE, null),
    NotificationItem("n3", "account", "Welcome to Nomi", "Your crew is ready when you are.", null, SNAPSHOT_NOW - 3 * 24 * 60 * MINUTE, SNAPSHOT_NOW),
)

/** The person's messages in each outbox state: sent, failed, then one still sending. */
private val Outbox = ChatState(
    messages = listOf(
        msg("m1", "user", "Book a table for Friday at 7.", 20).copy(status = MESSAGE_STATUS_SENT),
        msg("m2", "assistant", "Done — a table for two at Lumen, Friday 7pm.", 19, "chitchat", "Nomi"),
        msg("local-a", "user", "Make it four people instead.", 3).copy(status = MESSAGE_STATUS_FAILED),
        msg("local-b", "user", "And ask for a window seat.", 1).copy(status = MESSAGE_STATUS_SENDING),
    ),
    draft = null,
    activeAgent = null,
)

/** A message with two photos and a document, as the composer sends them. */
private val WithFiles = ChatState(
    messages = listOf(
        msg("m1", "user", "Here's the menu draft and two photos of the counter.\n\n" +
            "<attachment id=\"f1\" name=\"counter-1.jpg\" kind=\"image\" mime=\"image/jpeg\" size=\"204800\"/>\n\n" +
            "<attachment id=\"f2\" name=\"counter-2.jpg\" kind=\"image\" mime=\"image/jpeg\" size=\"198000\"/>\n\n" +
            "<attachment id=\"f3\" name=\"menu-draft.pdf\" kind=\"pdf\" mime=\"application/pdf\" size=\"1258291\"/>", 6),
        msg("m2", "assistant", "Got them. The menu reads well; I'd move the pastries above the drinks.", 5, "chitchat", "Nomi"),
    ),
    draft = null,
    activeAgent = null,
)

/** A reply with thinking and steps before it, a code block, a like, and the dislike form open on another. */
private val Replies = ChatState(
    messages = listOf(
        msg("r1", "user", "How do I read the bakery's orders file in Kotlin?", 9),
        msg("r2", "assistant", "", 8, "coding", "Koda").copy(
            content = "Thinking about the file format",
            contentJson = """[{"kind":"reasoning","text":"The orders file is CSV, one order per line. Read it lazily."}]""",
        ),
        msg(
            "r3", "assistant",
            "Read it line by line, so a big file never sits in memory:\n\n```kotlin\n// One order per line\nfun orders(path: String) = File(path).useLines { lines ->\n    lines.drop(1).map { it.split(\",\") }.toList()\n}\n```\n\nThe first line is the header, so it's skipped.",
            7, "coding", "Koda",
        ).copy(
            myFeedback = "up",
            stepsJson = """[{"tool":"read_file","detail":"orders.csv","ok":true},{"tool":"web_search","detail":"kotlin useLines","ok":true}]""",
        ),
        msg("r4", "user", "And the total per day?", 6),
        msg("r5", "assistant", "Group by the date column and sum the amounts.", 5, "coding", "Koda").copy(myFeedback = "down"),
    ),
    draft = null,
    activeAgent = null,
)

private val Bakery = ProjectInfo(
    id = "p1",
    name = "Bakery site",
    description = "A one-page site for the bakery",
    status = "ready",
    stack = "sveltekit",
    files = listOf(
        ProjectFileEntry("package.json", "application/json", 612),
        ProjectFileEntry("src/routes/+page.svelte", "text/plain", 2840),
        ProjectFileEntry("src/routes/menu/+page.svelte", "text/plain", 1932),
        ProjectFileEntry("src/lib/styles.css", "text/css", 1210),
    ),
    sessionId = "s9",
    orgName = "Trian's team",
    openIssues = 2,
)

private val Published = Loadable.Ready(
    ProjectDeployInfo(
        canPublish = true,
        url = "https://bakery.nomi.app",
        publishes = listOf(ProjectPublishInfo("u1", "succeeded", "https://bakery.nomi.app", null, "Trian", "2026-10-30T16:20:00Z")),
        backendSetupStep = null,
        backendSetupError = null,
    ),
)

class PagesScreenshotTest {
    @get:Rule val rule = createComposeRule()

    private val utc = TimeZone.UTC

    @Test fun chatsPhoneLight() = rule.captureScreen("chats", Device.Phone, false) {
        InShell(Device.Phone, "chats") { ChatsContent(SnapshotSessions, false, {}, {}, it, nowMs = SNAPSHOT_NOW) }
    }

    @Test fun chatsWideDark() = rule.captureScreen("chats", Device.Wide, true) {
        InShell(Device.Wide, "chats") { ChatsContent(SnapshotSessions, false, {}, {}, it, selectedId = "s2", nowMs = SNAPSHOT_NOW) }
    }

    @Test fun chatPhoneLight() = rule.captureScreen("chat", Device.Phone, false) {
        ChatContent("Saturday plan", Conversation, ChatComposerState(), {}, {}, {}, onBack = {}, timeZone = utc)
    }

    @Test fun chatPhoneDark() = rule.captureScreen("chat", Device.Phone, true) {
        ChatContent("Saturday plan", Conversation, ChatComposerState(), {}, {}, {}, onBack = {}, timeZone = utc)
    }

    @Test fun chatWideLight() = rule.captureScreen("chat", Device.Wide, false) {
        InShell(Device.Wide, "chats") { ChatContent("Saturday plan", Conversation, ChatComposerState(draft = "And the hike?"), {}, {}, {}, onBack = null, modifier = it, timeZone = utc) }
    }

    @Test fun chatOutboxPhoneLight() = rule.captureScreen("chat_outbox", Device.Phone, false) {
        ChatContent("Friday dinner", Outbox, ChatComposerState(), {}, {}, {}, onBack = {}, timeZone = utc)
    }

    @Test fun chatOutboxPhoneDark() = rule.captureScreen("chat_outbox", Device.Phone, true) {
        ChatContent("Friday dinner", Outbox, ChatComposerState(), {}, {}, {}, onBack = {}, timeZone = utc)
    }

    @Test fun chatAttachmentsPhoneLight() = rule.captureScreen("chat_attachments", Device.Phone, false) {
        ChatContent("Bakery menu", WithFiles, ChatComposerState(), {}, {}, {}, onBack = {}, timeZone = utc)
    }

    @Test fun chatRepliesPhoneLight() = rule.captureScreen("chat_replies", Device.Phone, false) {
        ChatContent(
            "Orders file", Replies, ChatComposerState(), {}, {}, {}, onBack = {}, timeZone = utc,
            feedback = mapOf("r5" to ReplyFeedbackState(dislikeOpen = true)),
        )
    }

    @Test fun chatRepliesPhoneDark() = rule.captureScreen("chat_replies", Device.Phone, true) {
        ChatContent("Orders file", Replies, ChatComposerState(), {}, {}, {}, onBack = {}, timeZone = utc)
    }

    private fun workspace(tab: String, openFile: OpenFile? = null): @androidx.compose.runtime.Composable () -> Unit = {
        ProjectWorkspaceContent(
            project = Bakery, deploy = Published, notice = null, openFile = openFile, tab = tab,
            onTabChange = {}, onBack = {}, onPublish = {}, onRetryDeploy = {}, onOpenTool = {}, onDismissNotice = {},
            onOpenFile = {}, onCloseFile = {},
        ) {
            ChatContent("", Conversation, ChatComposerState(), {}, {}, {}, onBack = null, timeZone = utc, showAppBar = false)
        }
    }

    @Test fun projectChatPhoneLight() = rule.captureScreen("project_chat", Device.Phone, false, workspace(PROJECT_TAB_CHAT))

    @Test fun projectChatPhoneDark() = rule.captureScreen("project_chat", Device.Phone, true, workspace(PROJECT_TAB_CHAT))

    @Test fun projectPreviewPhoneLight() = rule.captureScreen("project_preview", Device.Phone, false, workspace(PROJECT_TAB_PREVIEW))

    @Test fun projectCodePhoneLight() = rule.captureScreen("project_code", Device.Phone, false, workspace(PROJECT_TAB_CODE))

    @Test fun chatEmptyPhoneLight() = rule.captureScreen("chat_empty", Device.Phone, false) {
        ChatContent("", ChatState(emptyList(), null, null), ChatComposerState(), {}, {}, {}, onBack = {}, timeZone = utc)
    }

    @Test fun inboxPhoneLight() = rule.captureScreen("inbox", Device.Phone, false) {
        InShell(Device.Phone, "inbox") { NotificationsContent(Inbox, {}, {}, it, nowMs = SNAPSHOT_NOW) }
    }

    @Test fun inboxWideDark() = rule.captureScreen("inbox", Device.Wide, true) {
        InShell(Device.Wide, "inbox") { NotificationsContent(Inbox, {}, {}, it, nowMs = SNAPSHOT_NOW) }
    }

    @Test fun settingsPhoneLight() = rule.captureScreen("settings", Device.Phone, false) {
        InShell(Device.Phone, "settings") { SettingsContent(ProfileDto("Ana Lima", null, "ana@nomi.test"), {}, it) }
    }
}
