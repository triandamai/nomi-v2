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
import androidx.compose.runtime.getValue
import androidx.compose.runtime.setValue
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.ui.draw.clip
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
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.HorizontalDivider
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

data class NomiRecentChat(val id: String, val title: String)

data class NomiUsageBrief(
    val planName: String,
    val tokensUsed: Long,
    val monthlyTokens: Long,
    val canUpgrade: Boolean,
)

enum class NomiAccountMenuItem(val label: String) {
    Profile("Profile"),
    Preferences("Preferences"),
    AccountSetting("Account Setting"),
    BillingAndUsage("Billing & Usage"),
    Connections("Connections"),
    Model("Model"),
    Providers("Providers"),
    Memory("Memory"),
    Logout("Logout"),
}

data class NomiAccount(

    val displayName: String,
    val email: String,
    val avatarUrl: String? = null,
)

fun formatTokens(tokens: Long): String = when {
    tokens >= 1_000_000 -> "${tokens / 1_000_000}M"
    tokens >= 1_000 -> "${tokens / 1_000}k"
    else -> tokens.toString()
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
private val BrandMarkSize = 40.dp
private val FooterIconSize = 20.dp
private val FooterRowPadding = 14.dp
private val FooterRowGap = 12.dp

/**
 * The complete navigation drawer content matching web's `Sidebar.svelte`:
 * 1. Brand header with Nomi shape and close/collapse action
 * 2. `+ New chat` hero gradient button
 * 3. Scrollable navigation list + collapsible Recents section
 * 4. Usage progress meter (tokens used / plan allowance)
 * 5. Account avatar with display name and tap action, a settings button and Log out
 */
@Composable
fun <K> NomiNavDrawerContent(
    items: List<NomiNavItem<K>>,
    selected: K,
    onSelect: (K) -> Unit,
    onNewChat: () -> Unit,
    collapsed: Boolean,
    modifier: Modifier = Modifier,
    onToggleCollapsed: (() -> Unit)? = null,
    onClose: (() -> Unit)? = null,
    recents: List<NomiRecentChat> = emptyList(),
    currentChatId: String? = null,
    onOpenRecentChat: ((String) -> Unit)? = null,
    onSeeAllChats: (() -> Unit)? = null,
    usage: NomiUsageBrief? = null,
    onUpgrade: (() -> Unit)? = null,
    account: NomiAccount? = null,
    onOpenAccount: (() -> Unit)? = null,
    onAccountMenuItemClick: ((NomiAccountMenuItem) -> Unit)? = null,
    onOpenSettings: (() -> Unit)? = null,
    onLogOut: (() -> Unit)? = null,
    footer: (@Composable () -> Unit)? = null,
) {

    var recentsExpanded by androidx.compose.runtime.remember { androidx.compose.runtime.mutableStateOf(true) }

    Column(
        modifier = modifier
            .fillMaxHeight()
            .background(MaterialTheme.colorScheme.surfaceContainerLow)
            .statusBarsPadding()
            .navigationBarsPadding(),
        horizontalAlignment = if (collapsed) Alignment.CenterHorizontally else Alignment.Start,
    ) {
        // --- 1. Header ---
        Row(
            Modifier.fillMaxWidth().padding(start = HeaderPadding, end = HeaderPadding, top = HeaderTopPadding, bottom = NavPadding),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = if (collapsed) Arrangement.Center else Arrangement.SpaceBetween,
        ) {
            if (collapsed) {
                AgentShape(NomiIdentity, BrandMarkSize, face = true)
            } else {
                NomiWordmark(markSize = BrandMarkSize)
            }
            if (!collapsed) {
                if (onClose != null) {
                    IconButton(onClick = onClose) {
                        NomiIcon(NomiIcons.Close, contentDescription = "Close navigation menu")
                    }
                } else if (onToggleCollapsed != null) {
                    IconButton(onClick = onToggleCollapsed) {
                        NomiIcon(NomiIcons.ChevronLeft, contentDescription = "Collapse sidebar")
                    }
                }
            }
        }

        // --- 2. Hero Action: New Chat ---
        Box(Modifier.padding(horizontal = if (collapsed) 0.dp else HeaderPadding).padding(top = ItemGap * 4)) {
            NomiNewChatButton(onNewChat, Modifier.then(if (collapsed) Modifier else Modifier.fillMaxWidth()), compact = collapsed)
        }

        // --- 3. Scrollable Items (NavList + Recents) ---
        Column(
            Modifier
                .weight(FILL)
                .fillMaxWidth()
                .verticalScroll(rememberScrollState())
                .padding(horizontal = NavPadding, vertical = NavTopPadding),
            verticalArrangement = Arrangement.spacedBy(ItemGap),
            horizontalAlignment = if (collapsed) Alignment.CenterHorizontally else Alignment.Start,
        ) {
            items.forEach { item ->
                SidebarItem(item, item.key == selected, collapsed) { onSelect(item.key) }
            }

            // Recents section
            if (!collapsed && recents.isNotEmpty()) {
                Box(Modifier.fillMaxWidth().padding(top = 16.dp)) {
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .height(ItemHeight)
                            .clip(CircleShape)
                            .clickable { recentsExpanded = !recentsExpanded }
                            .padding(horizontal = 14.dp),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.SpaceBetween,
                    ) {
                        Text(
                            text = "Recents",
                            style = MaterialTheme.typography.labelLarge.copy(fontWeight = FontWeight.SemiBold),
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                        NomiIcon(
                            glyph = if (recentsExpanded) NomiIcons.ChevronDown else NomiIcons.ChevronRight,
                            contentDescription = if (recentsExpanded) "Collapse recents" else "Expand recents",
                            tint = MaterialTheme.colorScheme.onSurfaceVariant,
                            size = 18.dp,
                        )
                    }
                }

                if (recentsExpanded) {
                    recents.forEach { chat ->
                        val active = chat.id == currentChatId
                        val colors = MaterialTheme.colorScheme
                        val ink = if (active) colors.onPrimaryContainer else colors.onSurfaceVariant
                        Row(
                            modifier = Modifier
                                .fillMaxWidth()
                                .height(40.dp)
                                .clip(CircleShape)
                                .background(if (active) colors.primaryContainer else androidx.compose.ui.graphics.Color.Transparent)
                                .semantics(mergeDescendants = true) {}
                                .clickable { onOpenRecentChat?.invoke(chat.id) }
                                .padding(horizontal = 18.dp),
                            verticalAlignment = Alignment.CenterVertically,
                        ) {
                            Text(
                                text = chat.title,
                                style = MaterialTheme.typography.bodyMedium.copy(fontWeight = if (active) FontWeight.Bold else FontWeight.Normal),
                                color = ink,
                                maxLines = 1,
                                overflow = TextOverflow.Ellipsis,
                            )
                        }
                    }
                    if (onSeeAllChats != null) {
                        Text(
                            text = "See all chats",
                            style = MaterialTheme.typography.labelMedium.copy(fontWeight = FontWeight.SemiBold),
                            color = MaterialTheme.colorScheme.primary,
                            modifier = Modifier
                                .fillMaxWidth()
                                .clip(CircleShape)
                                .clickable(onClick = onSeeAllChats)
                                .padding(horizontal = 18.dp, vertical = 8.dp),
                        )
                    }
                }
            }
        }

        // Expand toggle when collapsed
        if (collapsed && onToggleCollapsed != null) {
            IconButton(onClick = onToggleCollapsed) {
                NomiIcon(NomiIcons.ChevronRight, contentDescription = "Expand sidebar")
            }
        }

        // --- 4. Usage meter & Account footer ---
        Column(
            Modifier.fillMaxWidth().padding(NavPadding),
            verticalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            if (!collapsed && usage != null) {
                val share = if (usage.monthlyTokens > 0) (usage.tokensUsed.toFloat() / usage.monthlyTokens.toFloat()).coerceIn(0f, 1f) else 0f
                val percent = (share * 100).toInt()
                val colors = MaterialTheme.colorScheme
                Column(
                    modifier = Modifier
                        .fillMaxWidth()
                        .clip(MaterialTheme.shapes.large)
                        .background(colors.surfaceContainer)
                        .padding(12.dp),
                    verticalArrangement = Arrangement.spacedBy(6.dp),
                ) {
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.SpaceBetween,
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Text(
                            text = usage.planName,
                            style = MaterialTheme.typography.titleSmall.copy(fontWeight = FontWeight.Bold),
                            color = colors.onSurface,
                        )
                        Text(
                            text = "$percent%",
                            style = MaterialTheme.typography.labelSmall,
                            color = if (share >= 0.9f) colors.error else colors.onSurfaceVariant,
                        )
                    }
                    ai.nomi.designsystem.component.NomiWavyProgress(
                        progress = { share },
                        contentDescription = "Token usage $percent%",
                        color = if (share >= 0.9f) colors.error else colors.primary,
                        modifier = Modifier.fillMaxWidth(),
                    )
                    Text(
                        text = "${formatTokens(usage.tokensUsed)} / ${formatTokens(usage.monthlyTokens)} tokens",
                        style = MaterialTheme.typography.labelSmall,
                        color = colors.onSurfaceVariant,
                    )
                    if (usage.canUpgrade && onUpgrade != null) {
                        Text(
                            text = "Upgrade plan",
                            style = MaterialTheme.typography.labelSmall.copy(fontWeight = FontWeight.Bold),
                            color = colors.primary,
                            modifier = Modifier.clickable(onClick = onUpgrade),
                        )
                    }
                }
            }

            if (footer != null) {
                footer()
            } else if (account != null || onOpenAccount != null || onAccountMenuItemClick != null) {
                val label = account?.displayName?.ifBlank { null } ?: account?.email ?: "Account"
                val colors = MaterialTheme.colorScheme
                var accountMenuExpanded by androidx.compose.runtime.remember { androidx.compose.runtime.mutableStateOf(false) }

                Box(Modifier.fillMaxWidth()) {
                    Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
                        Row(
                            modifier = Modifier
                                .weight(FILL)
                                .clip(CircleShape)
                                .semantics(mergeDescendants = true) {}
                                .clickable {
                                    if (onAccountMenuItemClick != null) {
                                        accountMenuExpanded = true
                                    } else {
                                        onOpenAccount?.invoke()
                                    }
                                }
                                .padding(horizontal = 8.dp, vertical = 6.dp),
                            verticalAlignment = Alignment.CenterVertically,
                            horizontalArrangement = Arrangement.spacedBy(10.dp),
                        ) {
                            ai.nomi.designsystem.component.NomiAvatar(
                                name = label,
                                avatarUrl = account?.avatarUrl,
                                size = 36.dp,
                            )
                            if (!collapsed) {
                                Text(
                                    text = label,
                                    style = MaterialTheme.typography.bodyMedium.copy(fontWeight = FontWeight.SemiBold),
                                    color = colors.onSurface,
                                    maxLines = 1,
                                    overflow = TextOverflow.Ellipsis,
                                )
                            }
                        }
                        if (!collapsed && onOpenSettings != null) {
                            IconButton(onClick = onOpenSettings) {
                                NomiIcon(NomiIcons.Settings, contentDescription = "Settings", tint = colors.onSurfaceVariant, size = FooterIconSize)
                            }
                        }
                    }

                    DropdownMenu(
                        expanded = accountMenuExpanded,
                        onDismissRequest = { accountMenuExpanded = false },
                    ) {
                        NomiAccountMenuItem.entries.forEach { item ->
                            DropdownMenuItem(
                                text = {
                                    Text(
                                        text = item.label,
                                        style = MaterialTheme.typography.bodyMedium.copy(
                                            fontWeight = if (item == NomiAccountMenuItem.Logout) FontWeight.Bold else FontWeight.Normal,
                                        ),
                                        color = if (item == NomiAccountMenuItem.Logout) MaterialTheme.colorScheme.error else MaterialTheme.colorScheme.onSurface,
                                    )
                                },
                                onClick = {
                                    accountMenuExpanded = false
                                    onAccountMenuItemClick?.invoke(item)
                                },
                            )
                        }
                        HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant)
                        Text(
                            text = "Nomi 2.0",
                            style = MaterialTheme.typography.labelSmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                            modifier = Modifier.padding(horizontal = 14.dp, vertical = 6.dp),
                        )
                    }
                }
            }

            if (footer == null && !collapsed && onLogOut != null) {
                val colors = MaterialTheme.colorScheme
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(ItemHeight)
                        .clip(CircleShape)
                        .semantics(mergeDescendants = true) {}
                        .clickable(onClick = onLogOut)
                        .padding(horizontal = FooterRowPadding),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(FooterRowGap),
                ) {
                    NomiIcon(NomiIcons.Logout, contentDescription = null, tint = colors.onSurfaceVariant, size = FooterIconSize)
                    Text(
                        text = "Log out",
                        style = MaterialTheme.typography.labelLarge.copy(fontWeight = FontWeight.SemiBold),
                        color = colors.onSurfaceVariant,
                    )
                }
            }
        }
    }
}

/**
 * The wide-window sidebar: wraps [NomiNavDrawerContent] with collapsible rail animation.
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
    recents: List<NomiRecentChat> = emptyList(),
    currentChatId: String? = null,
    onOpenRecentChat: ((String) -> Unit)? = null,
    onSeeAllChats: (() -> Unit)? = null,
    usage: NomiUsageBrief? = null,
    onUpgrade: (() -> Unit)? = null,
    account: NomiAccount? = null,
    onOpenAccount: (() -> Unit)? = null,
    onAccountMenuItemClick: ((NomiAccountMenuItem) -> Unit)? = null,
    onOpenSettings: (() -> Unit)? = null,
    onLogOut: (() -> Unit)? = null,
    footer: (@Composable () -> Unit)? = null,
) {
    val width by animateDpAsState(if (collapsed) RailWidth else SidebarWidth, MaterialTheme.motionScheme.defaultSpatialSpec())
    Box(
        modifier
            .width(width)
            .fillMaxHeight()
    ) {
        NomiNavDrawerContent(
            items = items,
            selected = selected,
            onSelect = onSelect,
            onNewChat = onNewChat,
            collapsed = collapsed,
            onToggleCollapsed = onToggleCollapsed,
            recents = recents,
            currentChatId = currentChatId,
            onOpenRecentChat = onOpenRecentChat,
            onSeeAllChats = onSeeAllChats,
            usage = usage,
            onUpgrade = onUpgrade,
            account = account,
            onOpenAccount = onOpenAccount,
            onAccountMenuItemClick = onAccountMenuItemClick,
            onOpenSettings = onOpenSettings,
            onLogOut = onLogOut,
            footer = footer,
        )
    }
}

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
            .semantics(mergeDescendants = true) { selected = active }
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
