package ai.nomi.app.screens

import ai.nomi.data.ChatState
import ai.nomi.data.Message
import ai.nomi.feature.chat.ChatComposerState
import ai.nomi.designsystem.shape.agentIdentity
import ai.nomi.designsystem.theme.NomiTheme
import ai.nomi.feature.chat.ChatContent
import ai.nomi.feature.chat.blocks.MessageBlocks
import ai.nomi.feature.chat.blocks.parseContentBlocks
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.ui.Modifier
import androidx.compose.ui.test.onRoot
import androidx.compose.ui.unit.dp
import io.github.takahirom.roborazzi.captureRoboImage
import androidx.compose.ui.test.junit4.createComposeRule
import kotlinx.datetime.TimeZone
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import org.junit.Rule
import java.io.File
import kotlin.test.Test

private const val MINUTE = 60_000L
private const val SETTLE = 900L

/**
 * Every view in the web's examples.json, each as its own reply from the crew, in a real chat — checked by eye against
 * the web's view components. Split in two chats so each capture stays readable.
 */
class ViewsScreenshotTest {
    @get:Rule val rule = createComposeRule()

    private val examples: List<JsonObject> by lazy {
        val file = File("../../../frontend/src/lib/views/examples.json")
        (Json.parseToJsonElement(file.readText()) as JsonArray).map { it as JsonObject }
    }

    private fun chat(names: Set<String>): ChatState {
        val shown = examples.filter { (it["view"] as JsonPrimitive).content in names }
        val messages = listOf(Message("u0", "s1", "user", null, null, "Show me what you've got.", null, SNAPSHOT_NOW - 30 * MINUTE, "done")) +
            shown.mapIndexed { i, example ->
                val block = JsonObject(mapOf("kind" to JsonPrimitive("view")) + example)
                Message("v$i", "s1", "assistant", "money", "Dana", "(view)", "[$block]", SNAPSHOT_NOW - (20 - i) * MINUTE, "done")
            }
        return ChatState(messages, null, null)
    }

    /** One view as a reply in the chat column's width, in light or dark. */
    private fun gallery(view: String, dark: Boolean = false, wide: Boolean = false) {
        val example = examples.first { (it["view"] as JsonPrimitive).content == view }
        val block = JsonObject(mapOf("kind" to JsonPrimitive("view")) + example)
        rule.mainClock.autoAdvance = false
        rule.setContent {
            NomiTheme(dark = dark) {
                Box(Modifier.background(NomiTheme.extendedColors.ground).width(if (wide) 760.dp else 390.dp).padding(16.dp)) {
                    MessageBlocks(
                        parseContentBlocks("[$block]"), "Dana", agentIdentity("money"), "(view)", null, { _, _ -> },
                        answered = false, onSend = {}, timeZone = TimeZone.UTC,
                    )
                }
            }
        }
        rule.mainClock.advanceTimeBy(SETTLE)
        rule.onRoot().captureRoboImage("src/desktopTest/snapshots/view_${view}${if (wide) "_wide" else ""}${if (dark) "_dark" else ""}.png")
    }

    @Test fun list() = gallery("list")
    @Test fun table() = gallery("table")
    @Test fun tableWide() = gallery("table", wide = true)
    @Test fun comparison() = gallery("comparison")
    @Test fun comparisonWide() = gallery("comparison", wide = true)
    @Test fun stepper() = gallery("stepper")
    @Test fun timeline() = gallery("timeline")
    @Test fun card() = gallery("card")
    @Test fun cardDark() = gallery("card", dark = true)
    @Test fun chart() = gallery("chart")
    @Test fun answer() = gallery("answer")
    @Test fun suggestions() = gallery("suggestions")
    @Test fun decision() = gallery("decision")
    @Test fun decisionWide() = gallery("decision", wide = true)
    @Test fun form() = gallery("form")
    @Test fun formDark() = gallery("form", dark = true)
    @Test fun link() = gallery("link")
    @Test fun task() = gallery("task")
    @Test fun shell() = gallery("shell")

    /** The thread itself: views as replies, the last ones above the composer. */
    @Test fun inChat() = rule.captureScreen("views_chat", Device.Phone, false) {
        ChatContent("Views", chat(setOf("answer", "suggestions")), ChatComposerState(), {}, {}, {}, onBack = {}, timeZone = TimeZone.UTC, onSendAnswer = {})
    }
}
