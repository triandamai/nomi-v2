package ai.nomi.feature.home

import ai.nomi.designsystem.theme.NomiTheme
import ai.nomi.network.HomeSummaryDto
import ai.nomi.network.OutItemDto
import ai.nomi.network.PlanItemDto
import ai.nomi.network.TodayItemDto
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.test.performClick
import org.junit.Rule
import kotlin.test.Test
import kotlin.test.assertEquals

class HomeStatusCardsTest {
    @get:Rule val rule = createComposeRule()

    @Test
    fun rendersEmptyStatusCards() {
        rule.setContent {
            NomiTheme {
                HomeStatusCards(
                    summary = HomeSummaryDto(),
                    onOpenReminders = {},
                    onOpenPlans = {},
                    onOpenChat = {},
                )
            }
        }

        rule.onNodeWithText("While you were out").assertExists()
        rule.onNodeWithText("Nothing moved while you were away.").assertExists()

        rule.onNodeWithText("Today").assertExists()
        rule.onNodeWithText("No reminders or routines due today.").assertExists()

        rule.onNodeWithText("Plans in progress").assertExists()
        rule.onNodeWithText("No active plans or to-do lists right now.").assertExists()
    }

    @Test
    fun rendersPopulatedStatusCardsAndClicks() {
        var clickedChat: String? = null
        var openedReminders = false
        var openedPlans = false

        val summary = HomeSummaryDto(
            whileYouWereOut = listOf(
                OutItemDto(
                    kind = "finished",
                    agent = "coding",
                    title = "Website deployment complete",
                    detail = "Published to production",
                    sessionId = "chat-123",
                    at = "2026-10-18T10:00:00Z",
                )
            ),
            today = listOf(
                TodayItemDto(
                    id = "rem-1",
                    runAt = "2026-10-18T16:00:00Z",
                    label = "Evening walk",
                    agent = "health",
                    recurrence = "daily",
                )
            ),
            plans = listOf(
                PlanItemDto(
                    kind = "todo",
                    title = "Q4 Goals",
                    agent = "planning",
                    done = 3,
                    total = 5,
                    sessionId = "chat-456",
                    updatedAt = "2026-10-18T12:00:00Z",
                )
            ),
            whileYouWereOutTotal = 2,
            todayTotal = 1,
            plansTotal = 1,
        )

        rule.setContent {
            NomiTheme {
                HomeStatusCards(
                    summary = summary,
                    onOpenReminders = { openedReminders = true },
                    onOpenPlans = { openedPlans = true },
                    onOpenChat = { clickedChat = it },
                )
            }
        }

        rule.onNodeWithText("Website deployment complete").assertExists()
        rule.onNodeWithText("Website deployment complete").performClick()
        assertEquals("chat-123", clickedChat)

        rule.onNodeWithText("Evening walk").assertExists()
        rule.onNodeWithText("Reminders").performClick()
        assertEquals(true, openedReminders)

        rule.onNodeWithText("Q4 Goals").assertExists()
        rule.onNodeWithText("All plans").performClick()
        assertEquals(true, openedPlans)
    }
}
