package ai.nomi.designsystem.ds

import ai.nomi.designsystem.generated.NomiShapeSpecs
import ai.nomi.designsystem.shape.AgentIdentity
import ai.nomi.designsystem.shape.AgentShape
import ai.nomi.designsystem.shape.ShapeMotion
import ai.nomi.designsystem.shape.agentIdentity
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.runtime.Composable
import androidx.compose.runtime.ReadOnlyComposable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

/**
 * The crew as the design names them (frontend-new `ds/agents/crew.ts`). Each owns a colour (`crew-<id>`, `-bg`,
 * `-ink`, from the generated pen tokens) and a mascot: their agent shape with Nomi's face. [look] is the agent the
 * shape is drawn as where the web gives a member a look of their own (`CREW_LOOKS` in `ds/agents/shapes.ts`); the
 * rest are found by name.
 */
enum class Crew(val displayName: String, internal val look: AgentIdentity? = null) {
    Nomi("Nomi"),
    Koda("Koda"),
    Dana("Dana"),
    Kala("Kala"),
    Rena("Rena"),
    Tara("Tara"),
    Maya("Maya"),
    Raga("Raga"),
    Personality("Personality"),
    Supervisor("Supervisor"),
    Iris("Iris", AgentIdentity("clover3", "dusk", ShapeMotion.Orbit)),
    Sora("Sora", AgentIdentity("wave10", "dusk", ShapeMotion.Spin)),
    ;

    private val id: String get() = name.lowercase()

    /** The crew member's own colour (`crew-<id>`). */
    val tone: Color @Composable @ReadOnlyComposable get() = crewColor(id)

    /** Their tinted ground in this theme (`crew-<id>-bg`). */
    val tint: Color @Composable @ReadOnlyComposable get() = crewBg(id)

    /** Their ink on that ground in this theme (`crew-<id>-ink`). */
    val ink: Color @Composable @ReadOnlyComposable get() = crewInk(id)

    companion object {
        /** A crew member by id or name ("koda", "Koda"); Nomi when unknown. */
        fun of(id: String): Crew = entries.firstOrNull { it.name.equals(id, ignoreCase = true) } ?: Nomi

        /** The crew member who is this backend agent_type ("money" → Dana); Nomi for anything else. */
        fun forAgentType(agentType: String?): Crew {
            if (agentType == null) return Nomi
            val name = NomiShapeSpecs.crewTypeByName.entries.firstOrNull { it.value == agentType.lowercase() }?.key ?: agentType
            return of(name)
        }
    }
}

/**
 * A crew member's mascot (web `Mascot`): their shape and gradient with the two-dot face. Decorative beside their
 * name; give it a [label] when it stands alone. [working] runs the shape's own motion.
 */
@Composable
fun Mascot(agent: Crew, size: Dp = 20.dp, modifier: Modifier = Modifier, label: String? = null, working: Boolean = false) {
    val described = if (label != null) modifier.semantics { contentDescription = label } else modifier
    AgentShape(agentIdentity(agent.displayName, agent.look), size, working = working, face = true, modifier = described)
}

/** Several mascots overlapping (`-space-x-1.5`), as the crew-working banner shows them. */
@Composable
fun MascotStack(crew: List<Crew>, size: Dp = 20.dp, modifier: Modifier = Modifier, working: Boolean = false) {
    Row(modifier, horizontalArrangement = Arrangement.spacedBy((-6).dp), verticalAlignment = Alignment.CenterVertically) {
        crew.forEach { Mascot(it, size, working = working) }
    }
}
