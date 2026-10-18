package ai.nomi.companion

import ai.nomi.data.CompanionSignal
import ai.nomi.designsystem.theme.NomiTheme
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onRoot
import androidx.compose.ui.unit.dp
import io.github.takahirom.roborazzi.captureRoboImage
import org.junit.Rule
import kotlin.test.Test

private const val SETTLE_MS = 600L
private val SheetWidth = 520.dp
private val SheetPadding = 24.dp
private val ItemSpacing = 16.dp
private val SectionSpacing = 20.dp

@OptIn(ExperimentalTestApi::class)
class CompanionScreenshotTest {
    @get:Rule val rule = createComposeRule()

    private fun capture(dark: Boolean, file: String, sheet: @Composable () -> Unit) {
        rule.mainClock.autoAdvance = false
        rule.setContent {
            NomiTheme(accent = "green", dark = dark, reducedMotion = false) {
                Box(
                    Modifier
                        .width(SheetWidth)
                        .background(MaterialTheme.colorScheme.background)
                        .padding(SheetPadding),
                    contentAlignment = Alignment.TopCenter,
                ) {
                    sheet()
                }
            }
        }
        rule.mainClock.advanceTimeBy(SETTLE_MS)
        rule.onRoot().captureRoboImage("src/desktopTest/snapshots/$file")
    }

    @Test
    fun dynamicIslandStatesSheet() = capture(false, "companion_dynamic_island_states.png") {
        Column(
            modifier = Modifier.fillMaxWidth(),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(ItemSpacing),
        ) {
            DynamicIslandPill(
                state = CompanionUiState.Idle(unreadCount = 2),
                onOpenLink = {},
                onOpenApp = {},
            )
            DynamicIslandPill(
                state = CompanionUiState.Working(
                    agentType = "research",
                    phase = "calling_tool",
                    unreadCount = 0,
                ),
                onOpenLink = {},
                onOpenApp = {},
            )
            DynamicIslandPill(
                state = CompanionUiState.Peeking(
                    card = CompanionCard.Reply(
                        sessionId = "s-1",
                        preview = "Your 3 PM design sync starts in 10 minutes. Want me to pull up the agenda?",
                        agentType = "general",
                    ),
                    unreadCount = 1,
                ),
                onOpenLink = {},
                onOpenApp = {},
                onQuickReply = { _, _ -> },
            )
            DynamicIslandPill(
                state = CompanionUiState.Expanded(
                    card = CompanionCard.Reply(
                        sessionId = "s-1",
                        preview = "Done — moved to 3:15 PM and notified the team.",
                        agentType = "general",
                    ),
                    working = CompanionSignal.Working(agentType = "general", phase = "writing_reply"),
                    unreadCount = 0,
                ),
                onOpenLink = {},
                onOpenApp = {},
                onQuickReply = { _, _ -> },
            )
        }
    }

    @Test
    fun messengerBubbleStatesSheet() = capture(false, "companion_messenger_bubble_states.png") {
        Column(
            modifier = Modifier.fillMaxWidth(),
            verticalArrangement = Arrangement.spacedBy(SectionSpacing),
        ) {
            MessengerBubble(
                state = CompanionUiState.Peeking(
                    card = CompanionCard.Reply(
                        sessionId = "s-2",
                        preview = "CI finished green on PR #58.",
                        agentType = "code",
                    ),
                    unreadCount = 3,
                ),
                onTapHead = {},
                onOpenLink = {},
                onOpenApp = {},
                dockedRight = true,
            )
            MessengerBubble(
                state = CompanionUiState.Expanded(
                    card = CompanionCard.Reply(
                        sessionId = "s-2",
                        preview = "CI finished green on PR #58. Ready to merge whenever you are.",
                        agentType = "code",
                    ),
                    unreadCount = 1,
                ),
                onTapHead = {},
                onOpenLink = {},
                onOpenApp = {},
                onQuickReply = { _, _ -> },
                onDismiss = {},
            )
            Row(
                Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.spacedBy(ItemSpacing, Alignment.CenterHorizontally),
            ) {
                BubbleDismissTarget(captured = false)
                BubbleDismissTarget(captured = true)
            }
        }
    }
}
