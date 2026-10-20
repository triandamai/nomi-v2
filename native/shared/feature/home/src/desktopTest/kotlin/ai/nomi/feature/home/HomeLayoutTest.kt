package ai.nomi.feature.home

import ai.nomi.data.Session
import ai.nomi.designsystem.theme.NomiTheme
import ai.nomi.network.CrewMemberDto
import ai.nomi.network.HomeSummaryDto
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithText
import org.junit.Rule
import kotlin.test.Test

class HomeLayoutTest {
    @get:Rule val rule = createComposeRule()

    private val baseState = HomeUiState(
        greeting = "Good afternoon, Ana.",
        subtitle = "What should we get done?",
        crew = listOf(
            CrewMemberDto("chitchat", "Nomi", "Talks with you", false, null, null, null, "idle", ""),
            CrewMemberDto("coding", "Coder", "Builds software", false, null, null, null, "working", "Refactoring"),
        ),
        recent = listOf(
            Session(id = "s1", title = "Alpha project", lastMessagePreview = "Hello world", updatedAt = 1729245600000L, unread = false),
            Session(id = "s2", title = "Beta chores", lastMessagePreview = "Done today", updatedAt = 1729242000000L, unread = false),
        ),
        summary = HomeSummaryDto(),
        draft = "",
    )

    @Test
    fun stageLayoutRendersSuggestionsAndHero() {
        rule.setContent {
            NomiTheme(reducedMotion = true) {
                HomeContent(
                    state = baseState.copy(homeLayout = "stage"),
                    onDraftChange = {},
                    onSend = {},
                    onSuggestion = {},
                    onOpenChat = {},
                    onOpenChats = {},
                )
            }
        }

        rule.onNodeWithText("Good afternoon, Ana.").assertExists()
        rule.onNodeWithText("Summarize this week’s spending").assertExists()
        rule.onNodeWithText("While you were out").assertExists()
        rule.onNodeWithText("Alpha project").assertExists()
    }

    @Test
    fun classicLayoutRendersCrewCardsAndRecentCards() {
        rule.setContent {
            NomiTheme(reducedMotion = true) {
                HomeContent(
                    state = baseState.copy(homeLayout = "classic"),
                    onDraftChange = {},
                    onSend = {},
                    onSuggestion = {},
                    onOpenChat = {},
                    onOpenChats = {},
                )
            }
        }

        rule.onNodeWithText("Good afternoon, Ana.").assertExists()
        rule.onNodeWithText("Your crew").assertExists()
        rule.onNodeWithText("While you were out").assertExists()
        rule.onNodeWithText("Alpha project").assertExists()
    }

    @Test
    fun todayLayoutRendersGreetingAndStatusFirst() {
        rule.setContent {
            NomiTheme(reducedMotion = true) {
                HomeContent(
                    state = baseState.copy(homeLayout = "today"),
                    onDraftChange = {},
                    onSend = {},
                    onSuggestion = {},
                    onOpenChat = {},
                    onOpenChats = {},
                )
            }
        }

        rule.onNodeWithText("Good afternoon, Ana.").assertExists()
        rule.onNodeWithText("While you were out").assertExists()
        rule.onNodeWithText("Alpha project").assertExists()
    }
}
