package ai.nomi.designsystem.shape

import ai.nomi.designsystem.generated.NomiShapeSpecs

/** How a shape moves while its agent works (mirrors SHAPE_MOTIONS in shapes.ts). */
enum class ShapeMotion(val key: String) {
    Spin("spin"),
    Wobble("wobble"),
    Bounce("bounce"),
    Pulse("pulse"),
    Orbit("orbit"),
    ;

    companion object {
        fun fromKey(key: String?): ShapeMotion = entries.firstOrNull { it.key == key } ?: Spin
    }
}

/** An agent's shape name (a key of [ai.nomi.designsystem.generated.NomiShapeSpecs.all]), gradient tone and working motion. */
data class AgentIdentity(val shape: String, val tone: String, val motion: ShapeMotion)

private const val NOMI = "Nomi"

private val identities: Map<String, AgentIdentity> = NomiShapeSpecs.agentShape.mapValues { (agent, shape) ->
    AgentIdentity(
        shape = shape,
        tone = NomiShapeSpecs.agentTone.getValue(agent),
        motion = ShapeMotion.fromKey(NomiShapeSpecs.agentMotion[agent]),
    )
}

val NomiIdentity: AgentIdentity = identities.getValue(NOMI)

private const val KIND_EXACT = "exact"

/**
 * The look of an agent_type or crew display name, resolved exactly as the web's agentLook: crew names map to
 * their agent_type, then NomiShapeSpecs.agentRules apply in order (first hit wins). A [dynamic] agent wears
 * its own backend-chosen (shape, tone, motion); anything unrecognised (chitchat included) is Nomi.
 */
fun agentIdentity(agentType: String, dynamic: AgentIdentity? = null): AgentIdentity {
    if (dynamic != null) return dynamic
    val lowered = agentType.lowercase()
    val key = NomiShapeSpecs.crewTypeByName[lowered] ?: lowered
    val rule = NomiShapeSpecs.agentRules.firstOrNull { r ->
        r.needles.any { if (r.kind == KIND_EXACT) key == it else key.contains(it) }
    }
    return identities[rule?.agent] ?: NomiIdentity
}
