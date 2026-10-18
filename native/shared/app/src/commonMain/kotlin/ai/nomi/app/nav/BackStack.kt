package ai.nomi.app.nav

/** Pops the top screen; the root stays. Returns whether anything was popped. */
fun MutableList<NomiRoute>.back(): Boolean {
    if (size <= ROOT_ONLY) return false
    removeAt(lastIndex)
    return true
}

/** A chat opened from a chat (the list-detail pane, a link inside a chat) replaces it instead of stacking up. */
fun MutableList<NomiRoute>.openChat(sessionId: String) = open(Chat(sessionId))

/** Goes to [route]: a [Chat] on top of a chat replaces it, a top-level route resets to Home + it. */
fun MutableList<NomiRoute>.open(route: NomiRoute) {
    when {
        route in TopLevelRoutes -> selectTopLevel(route)
        route is Chat && lastOrNull() is Chat -> set(lastIndex, route)
        lastOrNull() != route -> add(route)
    }
}

/** The navigation bar: Home is the root, any other tab sits directly on it. */
fun MutableList<NomiRoute>.selectTopLevel(route: NomiRoute) {
    val target = if (route == Home) listOf(Home) else listOf(Home, route)
    if (this == target) return
    clear()
    addAll(target)
}

/** Signed out: only sign-in. Signed in: Home. */
fun MutableList<NomiRoute>.resetTo(route: NomiRoute) {
    clear()
    add(route)
}

/** The top-level tab a back stack belongs to (the route right above Home, or Home). */
val List<NomiRoute>.currentTopLevel: NomiRoute
    get() = getOrNull(TAB_INDEX)?.takeIf { it in TopLevelRoutes } ?: Home

private const val ROOT_ONLY = 1
private const val TAB_INDEX = 1
