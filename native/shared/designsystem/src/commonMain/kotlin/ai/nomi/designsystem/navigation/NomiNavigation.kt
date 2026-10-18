package ai.nomi.designsystem.navigation

import ai.nomi.designsystem.brand.NomiNewChatButton
import ai.nomi.designsystem.brand.NomiWordmark
import ai.nomi.designsystem.component.nomiFocusRing
import ai.nomi.designsystem.icon.NomiGlyph
import ai.nomi.designsystem.icon.NomiIcon
import ai.nomi.designsystem.icon.NomiIcons
import ai.nomi.designsystem.shape.AgentShape
import ai.nomi.designsystem.shape.NomiIdentity
import ai.nomi.designsystem.theme.NomiTheme
import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Badge
import androidx.compose.material3.BadgedBox
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ShortNavigationBar
import androidx.compose.material3.ShortNavigationBarItem
import androidx.compose.material3.ShortNavigationBarItemDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.selected
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp

/** One destination in the app's navigation: its label, glyph and (optionally) an unread [badge]. */
data class NomiNavItem<K>(val key: K, val label: String, val glyph: NomiGlyph, val badge: Int = 0)

private const val BADGE_MAX = 99

private fun badgeText(count: Int) = if (count > BADGE_MAX) "$BADGE_MAX+" else count.toString()

/**
 * The phone's bottom navigation: M3 Expressive's short navigation bar on the drawer's container colour, with the
 * web nav list's active pill (primary container) and weights.
 */
@Composable
fun <K> NomiNavigationBar(items: List<NomiNavItem<K>>, selected: K, onSelect: (K) -> Unit, modifier: Modifier = Modifier) {
    val colors = MaterialTheme.colorScheme
    ShortNavigationBar(modifier = modifier, containerColor = colors.surfaceContainerLow, contentColor = colors.onSurfaceVariant) {
        items.forEach { item ->
            val active = item.key == selected
            ShortNavigationBarItem(
                selected = active,
                onClick = { onSelect(item.key) },
                icon = {
                    BadgedBox(badge = { if (item.badge > 0) Badge { Text(badgeText(item.badge)) } }) {
                        NomiIcon(item.glyph, contentDescription = null)
                    }
                },
                label = { Text(item.label, fontWeight = if (active) FontWeight.Bold else FontWeight.SemiBold) },
                colors = ShortNavigationBarItemDefaults.colors(
                    selectedIndicatorColor = colors.primaryContainer,
                    selectedIconColor = colors.onPrimaryContainer,
                    selectedTextColor = colors.onSurface,
                    unselectedIconColor = colors.onSurfaceVariant,
                    unselectedTextColor = colors.onSurfaceVariant,
                ),
            )
        }
    }
}

/** `Sidebar.svelte`: 288px drawer, or a 96px rail when collapsed. */
private val SidebarWidth = 288.dp
private val RailWidth = 96.dp
private val HeaderPadding = 16.dp
private val HeaderTopPadding = 20.dp
private val NavPadding = 12.dp
private val NavTopPadding = 14.dp
private val ItemHeight = 44.dp
private val ItemGap = 2.dp
private val IndicatorWidth = 56.dp
private val IndicatorHeight = 32.dp
private val ItemStartPadding = 4.dp
private val ItemLabelGap = 12.dp
private val BadgeOffsetX = 18.dp
private val BadgeOffsetY = (-6).dp
private const val ITEM_LABEL_WEIGHT = 600
private const val ITEM_ACTIVE_WEIGHT = 700
private const val FILL = 1f

/**
 * The wide-window sidebar: the wordmark, the gradient New chat, the nav list with M3 pill indicators, and a
 * collapse toggle that turns it into a rail. [footer] sits at the bottom (the account row).
 */
@Composable
fun <K> NomiSidebar(
    items: List<NomiNavItem<K>>,
    selected: K,
    onSelect: (K) -> Unit,
    onNewChat: () -> Unit,
    collapsed: Boolean,
    onToggleCollapsed: () -> Unit,
    modifier: Modifier = Modifier,
    footer: @Composable () -> Unit = {},
) {
    val width by animateDpAsState(if (collapsed) RailWidth else SidebarWidth, MaterialTheme.motionScheme.defaultSpatialSpec())
    Column(
        modifier
            .width(width)
            .fillMaxHeight()
            .background(MaterialTheme.colorScheme.surfaceContainerLow)
            .statusBarsPadding()
            .navigationBarsPadding(),
        horizontalAlignment = if (collapsed) Alignment.CenterHorizontally else Alignment.Start,
    ) {
        Row(
            Modifier.fillMaxWidth().padding(start = HeaderPadding, end = HeaderPadding, top = HeaderTopPadding, bottom = NavPadding),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = if (collapsed) Arrangement.Center else Arrangement.SpaceBetween,
        ) {
            if (collapsed) AgentShape(NomiIdentity, BrandMarkSize, face = true) else NomiWordmark(markSize = BrandMarkSize)
            if (!collapsed) {
                IconButton(onClick = onToggleCollapsed) { NomiIcon(NomiIcons.ChevronLeft, contentDescription = "Collapse sidebar") }
            }
        }
        Box(Modifier.padding(horizontal = if (collapsed) 0.dp else HeaderPadding).padding(top = ItemGap * 4)) {
            NomiNewChatButton(onNewChat, Modifier.then(if (collapsed) Modifier else Modifier.fillMaxWidth()), compact = collapsed)
        }
        Column(
            Modifier.weight(FILL).fillMaxWidth().verticalScroll(rememberScrollState()).padding(horizontal = NavPadding, vertical = NavTopPadding),
            verticalArrangement = Arrangement.spacedBy(ItemGap),
            horizontalAlignment = if (collapsed) Alignment.CenterHorizontally else Alignment.Start,
        ) {
            items.forEach { item -> SidebarItem(item, item.key == selected, collapsed) { onSelect(item.key) } }
        }
        if (collapsed) {
            IconButton(onClick = onToggleCollapsed) { NomiIcon(NomiIcons.ChevronRight, contentDescription = "Expand sidebar") }
        }
        Box(Modifier.fillMaxWidth().padding(NavPadding)) { footer() }
    }
}

private val BrandMarkSize = 40.dp

@Composable
private fun <K> SidebarItem(item: NomiNavItem<K>, active: Boolean, collapsed: Boolean, onClick: () -> Unit) {
    val colors = MaterialTheme.colorScheme
    val effects = MaterialTheme.motionScheme.fastEffectsSpec<androidx.compose.ui.graphics.Color>()
    val indicator by animateColorAsState(if (active) colors.primaryContainer else colors.primaryContainer.copy(alpha = 0f), effects)
    val ink = if (active) colors.onSurface else colors.onSurfaceVariant
    val pill = CircleShape
    Row(
        Modifier
            .then(if (collapsed) Modifier else Modifier.fillMaxWidth())
            .height(ItemHeight)
            .nomiFocusRing(pill)
            .semantics { selected = active }
            .clickable(role = Role.Tab, onClickLabel = item.label, onClick = onClick)
            .padding(start = ItemStartPadding, end = if (collapsed) ItemStartPadding else HeaderPadding - 2.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(ItemLabelGap),
    ) {
        Box(Modifier.size(IndicatorWidth, IndicatorHeight).background(indicator, pill), contentAlignment = Alignment.Center) {
            NomiIcon(item.glyph, contentDescription = if (collapsed) item.label else null, tint = if (active) colors.onPrimaryContainer else ink)
            if (item.badge > 0) {
                Badge(Modifier.align(Alignment.Center).offset(BadgeOffsetX, BadgeOffsetY)) { Text(badgeText(item.badge)) }
            }
        }
        if (!collapsed) {
            Text(
                item.label,
                style = MaterialTheme.typography.labelLarge.copy(fontWeight = FontWeight(if (active) ITEM_ACTIVE_WEIGHT else ITEM_LABEL_WEIGHT)),
                color = ink,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }
    }
}
