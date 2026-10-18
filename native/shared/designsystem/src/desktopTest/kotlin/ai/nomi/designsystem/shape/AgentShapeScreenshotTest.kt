package ai.nomi.designsystem.shape

import ai.nomi.designsystem.generated.NomiGradients
import ai.nomi.designsystem.generated.NomiShapeSpecs
import ai.nomi.designsystem.theme.NomiTheme
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.test.junit4.createComposeRule
import androidx.compose.ui.test.onRoot
import androidx.compose.ui.unit.dp
import io.github.takahirom.roborazzi.captureRoboImage
import org.junit.Rule
import kotlin.test.Test

private val Cell = 40.dp
private val Gap = 6.dp
private const val LOOK_LEFT = -1f
private const val LOOK_RIGHT = 1f
private const val LOOK_DOWN_RIGHT = 0.6f
private const val WORKING_FRAME_MS = 450L

class AgentShapeScreenshotTest {
    @get:Rule val rule = createComposeRule()

    private fun capture(file: String, content: @Composable () -> Unit) {
        rule.setContent { NomiTheme(accent = "green", dark = false) { content() } }
        rule.waitForIdle()
        rule.onRoot().captureRoboImage("src/desktopTest/snapshots/$file")
    }

    @Test fun shapeToneGrid() = capture("agent_shapes_grid.png") {
        Column(Modifier.background(androidx.compose.ui.graphics.Color.White).padding(Gap), verticalArrangement = Arrangement.spacedBy(Gap)) {
            NomiShapeSpecs.all.keys.forEach { shape ->
                Row(horizontalArrangement = Arrangement.spacedBy(Gap)) {
                    NomiGradients.all.keys.forEach { tone ->
                        AgentShape(AgentIdentity(shape, tone, ShapeMotion.Spin), Cell)
                    }
                }
            }
        }
    }

    @Test fun faceLooking() = capture("agent_face_look.png") {
        Row(Modifier.background(androidx.compose.ui.graphics.Color.White).padding(Gap), horizontalArrangement = Arrangement.spacedBy(Gap)) {
            listOf<Offset?>(null, Offset(LOOK_LEFT, 0f), Offset(LOOK_RIGHT, 0f), Offset(LOOK_DOWN_RIGHT, LOOK_DOWN_RIGHT)).forEach { look ->
                AgentShape(NomiIdentity, Cell * 3, face = true, lookAt = look)
            }
        }
    }

    @Test fun workingFrames() = capture("agent_working.png") {
        Row(Modifier.background(androidx.compose.ui.graphics.Color.White).padding(Gap), horizontalArrangement = Arrangement.spacedBy(Gap)) {
            ShapeMotion.entries.forEach { motion ->
                val identity = agentIdentity("x", AgentIdentity("cookie9", "dusk", motion))
                AgentShapeFrame(identity, Cell * 3, { motionFrame(motion, WORKING_FRAME_MS) }, face = true, look = Offset.Zero)
            }
        }
    }
}
