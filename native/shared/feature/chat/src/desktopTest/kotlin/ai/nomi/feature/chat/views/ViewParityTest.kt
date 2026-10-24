package ai.nomi.feature.chat.views

import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.jsonPrimitive
import java.io.File
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull

/**
 * The web's `frontend/src/lib/views/examples.json` holds one example of every view and is parsed by the backend's and
 * the web's tests too: every one must parse here as the view it names, so the three sides can't drift.
 */
class ViewParityTest {
    private val examples: List<JsonObject> by lazy {
        // Tests run from the module's directory (native/shared/feature/chat).
        val file = File("../../../../frontend/src/lib/views/examples.json")
        (Json.parseToJsonElement(file.readText()) as JsonArray).map { it as JsonObject }
    }

    @Test
    fun everyWebExampleParsesAsItsView() {
        examples.forEach { example ->
            val name = example.getValue("view").jsonPrimitive.content
            val view = assertNotNull(parseView(example), "the $name example didn't parse")
            assertEquals(name, view.name)
        }
    }

    @Test
    fun nativeDrawsEveryViewTheWebDraws() {
        assertEquals(examples.map { it.getValue("view").jsonPrimitive.content }.toSet(), ViewNames.toSet())
    }
}
