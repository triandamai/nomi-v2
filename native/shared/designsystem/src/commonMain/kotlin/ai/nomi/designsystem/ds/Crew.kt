package ai.nomi.designsystem.ds

import ai.nomi.designsystem.shape.AgentShape
import ai.nomi.designsystem.shape.agentIdentity
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.runtime.Composable
import androidx.compose.runtime.ReadOnlyComposable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.compositeOver
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

private const val CREW_BG_ALPHA = 0.16f

/**
 * The crew as the design names them (frontend `ds/agents/crew.ts`). Each owns a colour, deep in light and bright in
 * dark (ds-twins.css `--crew-*`), and a mascot: their agent shape with Nomi's face.
 */
enum class Crew(val displayName: String, private val light: Long, private val darkTone: Long) {
    Nomi("Nomi", 0xFF0B6B4A, 0xFF5BE08F),
    Koda("Koda", 0xFF0E7490, 0xFF3FB8C8),
    Dana("Dana", 0xFFB8430F, 0xFFFF7A4A),
    Kala("Kala", 0xFFA16207, 0xFFFFC23C),
    Rena("Rena", 0xFF3E7BE0, 0xFF7AB0FF),
    Tara("Tara", 0xFF3E7BE0, 0xFF7AB0FF),
    Maya("Maya", 0xFF475569, 0xFF7D93A3),
    Raga("Raga", 0xFFBE185D, 0xFFFF8FA8),
    Personality("Personality", 0xFFBE185D, 0xFFFF8FA8),
    Supervisor("Supervisor", 0xFF6D4FD8, 0xFF9B7BFF),
    ;

    /** The crew member's colour in this theme. */
    val tone: Color @Composable @ReadOnlyComposable get() = Color(if (Ds.dark) darkTone else light)

    /** Their tint on the surface (`--crew-*-bg`). */
    val tint: Color @Composable @ReadOnlyComposable get() = tone.copy(alpha = CREW_BG_ALPHA).compositeOver(Ds.surface)

    companion object {
        /** A crew member by id or name ("koda", "Koda"); Nomi when unknown. */
        fun of(id: String): Crew = entries.firstOrNull { it.name.equals(id, ignoreCase = true) } ?: Nomi
    }
}

/**
 * A crew member's mascot (web `Mascot`): their shape and gradient with the two-dot face. Decorative beside their
 * name; give it a [label] when it stands alone. [working] runs the shape's own motion.
 */
@Composable
fun Mascot(agent: Crew, size: Dp = 20.dp, modifier: Modifier = Modifier, label: String? = null, working: Boolean = false) {
    val described = if (label != null) modifier.semantics { contentDescription = label } else modifier
    AgentShape(agentIdentity(agent.displayName), size, working = working, face = true, modifier = described)
}

/** Several mascots overlapping (`-space-x-1.5`), as the crew-working banner shows them. */
@Composable
fun MascotStack(crew: List<Crew>, size: Dp = 20.dp, modifier: Modifier = Modifier, working: Boolean = false) {
    Row(modifier, horizontalArrangement = Arrangement.spacedBy((-6).dp), verticalAlignment = Alignment.CenterVertically) {
        crew.forEach { Mascot(it, size, working = working) }
    }
}
