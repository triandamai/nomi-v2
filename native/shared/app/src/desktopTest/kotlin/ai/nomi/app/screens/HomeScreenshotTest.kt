package ai.nomi.app.screens

import ai.nomi.data.Session
import ai.nomi.designsystem.icon.NomiIcons
import ai.nomi.designsystem.navigation.NomiNavItem
import ai.nomi.designsystem.navigation.NomiNavigationBar
import ai.nomi.designsystem.navigation.NomiSidebar
import ai.nomi.feature.home.HomeContent
import ai.nomi.feature.home.HomeUiState
import ai.nomi.network.CrewMemberDto
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.test.junit4.createComposeRule
import org.junit.Rule
import kotlin.test.Test

/** Fixed "now" so relative times in the snapshots never change. */
const val SNAPSHOT_NOW = 1_800_000_000_000L
private const val MINUTE = 60_000L

val SnapshotNav = listOf(
    NomiNavItem("home", "Home", NomiIcons.Home),
    NomiNavItem("chats", "Chats", NomiIcons.Chat),
    NomiNavItem("inbox", "Inbox", NomiIcons.Bell, badge = 3),
    NomiNavItem("settings", "Settings", NomiIcons.Settings),
)

val SnapshotSessions = listOf(
    Session("s1", "Saturday plan", "Farmers market at 9, then the hike if it stays dry.", SNAPSHOT_NOW - 4 * MINUTE, unread = true),
    Session("s2", "March spending", "You spent 12% less on food than in February.", SNAPSHOT_NOW - 3 * 60 * MINUTE, unread = false),
    Session("s3", "Portfolio site", "Preview is live — want me to publish it?", SNAPSHOT_NOW - 26 * 60 * MINUTE, unread = false),
)

/** The signed-in shell around [content]: the sidebar on wide windows, the bottom bar on phones. */
@Composable
fun InShell(device: Device, selected: String, content: @Composable (Modifier) -> Unit) {
    if (device == Device.Wide) {
        Row {
            NomiSidebar(SnapshotNav, selected, {}, onNewChat = {}, collapsed = false, onToggleCollapsed = {})
            content(Modifier.weight(1f).fillMaxHeight())
        }
    } else {
        Column {
            content(Modifier.weight(1f))
            NomiNavigationBar(SnapshotNav, selected, {})
        }
    }
}

class HomeScreenshotTest {
    @get:Rule val rule = createComposeRule()

    private val state = HomeUiState(
        greeting = "Afternoon, Ana.",
        subtitle = "Your crew is working in 1 chat.",
        crew = listOf(
            CrewMemberDto("chitchat", "Nomi", "Talks things through with you", false, null, null, null, "working", "Planning your Saturday"),
            CrewMemberDto("money", "Dana", "Keeps an eye on your money", false, null, null, null, "waiting", "Approve the gym refund"),
            CrewMemberDto("planning", "Rena", "Plans and to-do lists", false, null, null, null, "done", "Finished · Weekly plan"),
            CrewMemberDto("coding", "Koda", "Builds small sites and apps", false, null, null, null, "idle", ""),
        ),
        recent = SnapshotSessions,
    )

    private fun capture(device: Device, dark: Boolean) = rule.captureScreen("home", device, dark) {
        InShell(device, "home") { modifier -> HomeContent(state, {}, {}, {}, {}, {}, modifier, nowMs = SNAPSHOT_NOW) }
    }

    @Test fun phoneLight() = capture(Device.Phone, false)
    @Test fun phoneDark() = capture(Device.Phone, true)
    @Test fun wideLight() = capture(Device.Wide, false)
    @Test fun wideDark() = capture(Device.Wide, true)
}
