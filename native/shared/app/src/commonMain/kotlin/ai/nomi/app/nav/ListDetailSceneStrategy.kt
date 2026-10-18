package ai.nomi.app.nav

import ai.nomi.designsystem.layout.NomiSpacing
import ai.nomi.designsystem.theme.NomiTheme
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.width
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.navigation3.runtime.NavEntry
import androidx.navigation3.scene.Scene
import androidx.navigation3.scene.SceneStrategy
import androidx.navigation3.scene.SceneStrategyScope

/** Entry metadata marking the list pane (Chats) and the detail pane (Chat) of the list-detail layout. */
internal const val LIST_PANE = "nomi.pane.list"
internal const val DETAIL_PANE = "nomi.pane.detail"

internal val ListPaneMetadata: Map<String, Any> = mapOf(LIST_PANE to true)
internal val DetailPaneMetadata: Map<String, Any> = mapOf(DETAIL_PANE to true)

/** The list takes this share of the width; the detail the rest. */
private const val LIST_WEIGHT = 2f
private const val DETAIL_WEIGHT = 3f
private const val PREVIOUS_OF_TOP = 1

/**
 * On wide windows ([enabled]) shows a list entry and the detail entry pushed on it side by side; a list on top shows
 * next to [placeholder]. Anything else (and every compact window) falls through to the single-pane default.
 */
internal class ListDetailSceneStrategy<T : Any>(
    private val enabled: Boolean,
    private val placeholder: @Composable () -> Unit,
) : SceneStrategy<T> {
    override fun SceneStrategyScope<T>.calculateScene(entries: List<NavEntry<T>>): Scene<T>? {
        if (!enabled) return null
        val top = entries.lastOrNull() ?: return null
        return when {
            DETAIL_PANE in top.metadata -> {
                val list = entries.getOrNull(entries.lastIndex - PREVIOUS_OF_TOP)?.takeIf { LIST_PANE in it.metadata }
                    ?: return null
                ListDetailScene(list, top, entries.dropLast(PREVIOUS_OF_TOP), placeholder)
            }
            LIST_PANE in top.metadata -> ListDetailScene(top, null, entries.dropLast(PREVIOUS_OF_TOP), placeholder)
            else -> null
        }
    }
}

private class ListDetailScene<T : Any>(
    private val list: NavEntry<T>,
    private val detail: NavEntry<T>?,
    override val previousEntries: List<NavEntry<T>>,
    private val placeholder: @Composable () -> Unit,
) : Scene<T> {
    // Keyed on the list alone: switching the detail keeps the scene (no transition), the list stays put.
    override val key: Any = ListDetailScene::class to list.contentKey
    override val entries: List<NavEntry<T>> = listOfNotNull(list, detail)
    override val content: @Composable () -> Unit = {
        Row(Modifier.fillMaxSize()) {
            Box(Modifier.weight(LIST_WEIGHT).fillMaxHeight()) { list.Content() }
            Box(Modifier.width(NomiSpacing.ExtraSmall).fillMaxHeight().background(NomiTheme.extendedColors.ground))
            Box(Modifier.weight(DETAIL_WEIGHT).fillMaxHeight()) { detail?.Content() ?: placeholder() }
        }
    }
}
