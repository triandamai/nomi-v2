package ai.nomi.designsystem.ds

import ai.nomi.designsystem.generated.NomiPenColors
import ai.nomi.designsystem.theme.NomiTheme
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.compositeOver
import androidx.compose.ui.semantics.ProgressBarRangeInfo
import androidx.compose.ui.semantics.SemanticsProperties
import androidx.compose.ui.test.SemanticsMatcher
import androidx.compose.ui.test.assertHeightIsEqualTo
import androidx.compose.ui.test.assertWidthIsEqualTo
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onNodeWithContentDescription
import androidx.compose.ui.test.onNodeWithText
import androidx.compose.ui.text.TextLayoutResult
import androidx.compose.ui.semantics.SemanticsActions
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import org.junit.Rule
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertTrue

private val light = NomiPenColors.light
private fun c(argb: Long) = Color(argb)
private val ground = c(light.bg)

/** Design System / Nomi → Status and Identity: frontend-new `ds/status`, `ds/identity` and the crew. */
class StatusIdentityTest {
    @get:Rule val rule = createComposeRule()

    private fun show(content: @Composable () -> Unit) = rule.setContent {
        NomiTheme(dark = false) { Box(Modifier.background(Ds.bg).padding(8.dp).width(480.dp)) { content() } }
    }

    @Test
    fun `a status is a 6dp dot and a word, coloured by what it means`() {
        val cases = listOf(
            Triple(StatusKind.Live, "Live", c(light.accent) to c(light.text)),
            Triple(StatusKind.Ready, "ready", c(light.accent) to c(light.text2)),
            Triple(StatusKind.Idle, "idle", c(light.text3) to c(light.text3)),
            Triple(StatusKind.Failed, "Deploy failed", c(light.danger) to c(light.danger)),
            Triple(StatusKind.Attention, "Needs attention", c(light.warning) to c(light.warning)),
        )
        cases.forEach { (kind, label, colours) ->
            rule.setContent { NomiTheme(dark = false) { Box(Modifier.background(Ds.bg).padding(8.dp)) { DsStatus(kind, label) } } }
            val status = rule.onNodeWithText(label)
            val middle = with(rule.density) { status.fetchSemanticsNode().boundsInRoot.height.toDp() } / 2
            assertColor(colours.first, status.pixelAt(rule, 3.dp, middle), "$label dot")
            assertColor(ground, status.pixelAt(rule, 7.5.dp, middle), "$label after a 6dp dot")
            assertEquals(colours.second, rule.onNodeWithText(label, useUnmergedTree = true).textStyle().color, "$label word")
        }
    }

    @Test
    fun `ready and idle are mono 11, the rest 12`() {
        show { DsStatus(StatusKind.Ready, "ready") }
        assertEquals(11.sp, rule.onNodeWithText("ready", useUnmergedTree = true).textStyle().fontSize)
        show { DsStatus(StatusKind.Live, "Live") }
        assertEquals(12.sp, rule.onNodeWithText("Live", useUnmergedTree = true).textStyle().fontSize)
    }

    @Test
    fun `a tag says go, no-go or standby in small mono capitals, 18dp tall`() {
        listOf(
            Triple(TagKind.NoGo, "No-go", c(light.danger) to c(light.dangerBg).compositeOver(ground)),
            Triple(TagKind.Go, "Go", c(light.accent) to c(light.accentBg).compositeOver(ground)),
            Triple(TagKind.Neutral, "Stby", c(light.text2) to c(light.surface2)),
        ).forEach { (kind, label, colours) ->
            rule.setContent { NomiTheme(dark = false) { Box(Modifier.background(Ds.bg).padding(8.dp)) { DsTag(kind, label) } } }
            val tag = rule.onNodeWithText(label.uppercase())
            tag.assertHeightIsEqualTo(18.dp)
            assertEquals(colours.first, rule.onNodeWithText(label.uppercase(), useUnmergedTree = true).textStyle().color, "$label ink")
            assertColor(colours.second, tag.pixelAt(rule, 2.dp, 9.dp), "$label fill")
        }
    }

    @Test
    fun `a section label is mono capitals in text-3 with its count`() {
        show { SectionLabel("Recents", count = "4") }
        assertEquals(c(light.text3), rule.onNodeWithText("RECENTS", useUnmergedTree = true).textStyle().color)
        rule.onNodeWithText("4", useUnmergedTree = true).assertExists()
    }

    @Test
    fun `a meter shows how much is used as a 4dp text-2 bar and in words`() {
        show { Meter("Free plan", 0.5f, caption = "500K / 1M tokens this month") }
        rule.onNode(SemanticsMatcher.expectValue(SemanticsProperties.ProgressBarRangeInfo, ProgressBarRangeInfo(0.5f, 0f..1f))).assertExists()
        rule.onNodeWithText("50%", useUnmergedTree = true).assertExists()
        rule.onNodeWithText("500K / 1M tokens this month", useUnmergedTree = true).assertExists()
        assertEquals(c(light.text3), rule.onNodeWithText("50%", useUnmergedTree = true).textStyle().color)
    }

    @Test
    fun `an avatar is a 28dp tile with mono initials and the person's name`() {
        show { DsAvatar("Mario Prasetyo") }
        val tile = rule.onNodeWithContentDescription("Mario Prasetyo")
        tile.assertWidthIsEqualTo(28.dp).assertHeightIsEqualTo(28.dp)
        assertColor(c(light.borderStrong), tile.pixelAt(rule, 0.5.dp, 14.dp), "tile edge")
        assertColor(c(light.surface2), tile.pixelAt(rule, 3.dp, 3.dp), "tile fill")
        assertEquals(11.sp, rule.onNodeWithText("MP", useUnmergedTree = true).textStyle().fontSize)
    }

    @Test
    fun `a byline says who is speaking in 13 medium and when in mono text-3`() {
        show { AgentByline(Crew.Koda, "Koda", meta = "15:37 · 4m 12s") }
        val name = rule.onNodeWithText("Koda", useUnmergedTree = true).textStyle()
        assertEquals(13.sp, name.fontSize)
        assertEquals(androidx.compose.ui.text.font.FontWeight.Medium, name.fontWeight)
        assertEquals(c(light.text3), rule.onNodeWithText("15:37 · 4m 12s", useUnmergedTree = true).textStyle().color)
    }

    @Test
    fun `a crew row keeps what they do on one line, cut short`() {
        val role = "Budgets, transactions, subscriptions, bills, savings goals and the monthly money check-in"
        rule.setContent { NomiTheme(dark = false) { Box(Modifier.background(Ds.bg).width(240.dp)) { CrewRow(Crew.Dana, "Dana", role, state = "idle") } } }
        val results = mutableListOf<TextLayoutResult>()
        rule.onNodeWithText(role, useUnmergedTree = true).fetchSemanticsNode().config[SemanticsActions.GetTextLayoutResult].action?.invoke(results)
        assertEquals(1, results.single().lineCount)
        assertTrue(results.single().hasVisualOverflow)
        assertEquals(c(light.text3), results.single().layoutInput.style.color)
    }

    @Test
    fun `the crew is all twelve, each in its own colour`() {
        assertEquals(
            listOf("Nomi", "Koda", "Dana", "Kala", "Rena", "Tara", "Maya", "Raga", "Personality", "Supervisor", "Iris", "Sora"),
            Crew.entries.map { it.displayName },
        )
        var sora = Color.Unspecified
        show { sora = Crew.Sora.tone }
        rule.waitForIdle()
        assertEquals(Color(0xFF7F8FF4), sora)
    }

    @Test
    fun `iris and sora wear the web's looks`() {
        assertEquals(ai.nomi.designsystem.shape.AgentIdentity("clover3", "dusk", ai.nomi.designsystem.shape.ShapeMotion.Orbit), Crew.Iris.look)
        assertEquals(ai.nomi.designsystem.shape.AgentIdentity("wave10", "dusk", ai.nomi.designsystem.shape.ShapeMotion.Spin), Crew.Sora.look)
    }
}
