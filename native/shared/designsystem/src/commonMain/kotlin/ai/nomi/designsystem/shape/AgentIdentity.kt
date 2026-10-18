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

/** The web matches by substring ("budget" is money, "memory" is personality); these are the extra needles. */
private val aliases = mapOf("budget" to "money", "memory" to "personality", "reminder" to "reminders", "cod" to "coding", "plan" to "planning")

val NomiIdentity: AgentIdentity = identities.getValue(NOMI)

/**
 * The look of an agent_type: a [dynamic] agent wears its own backend-chosen (shape, tone, motion);
 * the built-in crew is matched like the web's agentLook; anything unrecognised (chitchat included) is Nomi.
 */
fun agentIdentity(agentType: String, dynamic: AgentIdentity? = null): AgentIdentity {
    if (dynamic != null) return dynamic
    val key = agentType.lowercase()
    val agent = identities.keys.firstOrNull { it != NOMI && key.contains(it.lowercase()) }
        ?: aliases.entries.firstOrNull { key.contains(it.key) }?.value
    return identities[agent] ?: NomiIdentity
}
