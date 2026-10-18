package ai.nomi.app.screens

import ai.nomi.data.AgentActivity
import ai.nomi.data.ChatState
import ai.nomi.data.Draft
import ai.nomi.data.Message
import ai.nomi.data.NotificationItem
import ai.nomi.feature.chat.ChatComposerState
import ai.nomi.feature.chat.ChatContent
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
