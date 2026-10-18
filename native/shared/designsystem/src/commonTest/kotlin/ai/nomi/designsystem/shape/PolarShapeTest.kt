package ai.nomi.designsystem.shape

import ai.nomi.designsystem.generated.NomiShapeSpecs
import ai.nomi.designsystem.generated.SHAPE_CENTER
import ai.nomi.designsystem.generated.SHAPE_POINTS
import ai.nomi.designsystem.generated.SHAPE_RADIUS
import kotlin.math.PI
import kotlin.math.cos
import kotlin.math.hypot
import kotlin.math.sin
import kotlin.test.Test
import kotlin.test.assertContentEquals
import kotlin.test.assertEquals

private const val TOLERANCE = 1e-3f
private const val COOKIE9_FIRST_X_FACTOR = 1.09f
private const val TWO_VALUES = 2
private const val MIDPOINT = 0.5f
private const val CLOVER3_AMPLITUDE = 0.19f
private const val CLOVER3_LOBES = 3
private const val QUARTER_TURN_DIVISOR = 4

class PolarShapeTest {
    @Test fun everyShapeHasSamePointCount() {
        NomiShapeSpecs.all.values.forEach { assertEquals(SHAPE_POINTS * TWO_VALUES, polarPoints(it).size) }
    }

    @Test fun circleIsRadiusFromCenter() {
        val p = polarPoints(NomiShapeSpecs.all.getValue("circle"))
        for (i in p.indices step TWO_VALUES) {
            val r = hypot(p[i] - SHAPE_CENTER, p[i + 1] - SHAPE_CENTER)
            assertEquals(SHAPE_RADIUS, r, TOLERANCE)
        }
    }

    @Test fun firstPointMatchesWebFormula() {
        val p = polarPoints(NomiShapeSpecs.all.getValue("cookie9"))
        assertEquals(SHAPE_CENTER + SHAPE_RADIUS * COOKIE9_FIRST_X_FACTOR, p[0], TOLERANCE)
        assertEquals(SHAPE_CENTER, p[1], TOLERANCE)
    }

    @Test fun phasedShapeMatchesWebFormula() {
        // shapes.ts: r = R·(1 + a·cos(lobes·(θ − phase))), θ = 0 for the first point
        val p = polarPoints(NomiShapeSpecs.all.getValue("clover3"))
        val r0 = SHAPE_RADIUS * (1 + CLOVER3_AMPLITUDE * cos(CLOVER3_LOBES * (PI / 2)).toFloat())
        assertEquals(SHAPE_CENTER + r0, p[0], TOLERANCE)
        assertEquals(SHAPE_CENTER, p[1], TOLERANCE)
        val i = SHAPE_POINTS / QUARTER_TURN_DIVISOR
        val theta = 2 * PI * i / SHAPE_POINTS
        val ri = SHAPE_RADIUS * (1 + CLOVER3_AMPLITUDE * cos(CLOVER3_LOBES * (theta + PI / 2)).toFloat())
        assertEquals(SHAPE_CENTER + ri * cos(theta).toFloat(), p[i * TWO_VALUES], TOLERANCE)
        assertEquals(SHAPE_CENTER + ri * sin(theta).toFloat(), p[i * TWO_VALUES + 1], TOLERANCE)
    }

    @Test fun lerpHalfwayIsMidpoint() {
        val a = floatArrayOf(0f, 0f)
        val b = floatArrayOf(10f, 20f)
        assertContentEquals(floatArrayOf(5f, 10f), lerpPoints(a, b, MIDPOINT))
    }

    @Test fun unknownAgentFallsBackToNomi() {
        assertEquals("cookie9", agentIdentity("does-not-exist").shape)
    }

    @Test fun canonicalAgentsWearTheirLook() {
        assertEquals(AgentIdentity("clover3", "citrus", ShapeMotion.Pulse), agentIdentity("reminders"))
        assertEquals(AgentIdentity("soft-square", "slate", ShapeMotion.Wobble), agentIdentity("files"))
        assertEquals(AgentIdentity("sunny12", "dusk", ShapeMotion.Orbit), agentIdentity("supervisor"))
        assertEquals(AgentIdentity("sunny8", "ember", ShapeMotion.Spin), agentIdentity("Money"))
    }

    @Test fun dynamicIdentityWins() {
        val own = AgentIdentity("pill", "bloom", ShapeMotion.Bounce)
        assertEquals(own, agentIdentity("anything", own))
    }

    @Test fun matchingFollowsTheWebRuleOrder() {
        // collisions found in review: each must resolve exactly as shapes.ts agentLook does
        assertEquals(NomiIdentity, agentIdentity("workspace_files"))
        assertEquals(NomiIdentity, agentIdentity("healthcare"))
        assertEquals("sunny8", agentIdentity("budget_reminders").shape)
        assertEquals("clover4", agentIdentity("reminders_planner").shape)
    }

    @Test fun everyAgentHasAPlainMatch() {
        mapOf(
            "money" to "sunny8", "budget" to "sunny8", "coding" to "cookie6", "planning" to "clover4",
            "personality" to "flower5", "memory" to "flower5", "supervisor" to "sunny12", "reminders" to "clover3",
            "files" to "soft-square", "workspace" to "puffy7", "health" to "burst16", "chitchat" to "cookie9",
        ).forEach { (agent, shape) -> assertEquals(shape, agentIdentity(agent).shape, agent) }
    }

    @Test fun crewNamesResolveToTheirAgent() {
        assertEquals(agentIdentity("money"), agentIdentity("Dana"))
        assertEquals(agentIdentity("reminders"), agentIdentity("Kala"))
        assertEquals(agentIdentity("health"), agentIdentity("raga"))
        assertEquals(NomiIdentity, agentIdentity("Nomi"))
    }
}
