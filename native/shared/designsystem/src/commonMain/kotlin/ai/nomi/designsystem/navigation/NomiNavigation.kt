package ai.nomi.designsystem.navigation

import ai.nomi.designsystem.ds.Crew
import ai.nomi.designsystem.ds.Ds
import ai.nomi.designsystem.ds.DsAvatar
import ai.nomi.designsystem.ds.DsIcon
import ai.nomi.designsystem.ds.DsIconButton
import ai.nomi.designsystem.ds.DsIconButtonVariant
import ai.nomi.designsystem.ds.DsMotion
import ai.nomi.designsystem.ds.Mascot
import ai.nomi.designsystem.ds.NavItem
import ai.nomi.designsystem.ds.RecentItem
import ai.nomi.designsystem.ds.RecentKind
import ai.nomi.designsystem.ds.RecentsList
import ai.nomi.designsystem.ds.UsageCard
import ai.nomi.designsystem.ds.UserRow
import ai.nomi.designsystem.icon.Lucide
import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.layout.Spacer
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.draw.clipToBounds
import androidx.compose.ui.unit.em
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

data class NomiRecentChat(val id: String, val title: String, val whenLabel: String = "")

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

/** `Sidebar.svelte`: the design system's 300px drawer, or an 80px icon rail when collapsed. */
private val SidebarWidth = 300.dp
private val RailWidth = 80.dp
private val DrawerPadding = 12.dp
private val BrandGap = 8.dp
private val BrandMascot = 32.dp
private val NewChatHeight = 56.dp
private val NewChatGap = 12.dp
private val NewChatIcon = 20.dp
private val SectionGap = 8.dp
private val RecentsTop = 14.dp
private val RailGap = 12.dp
private val RailTop = 16.dp
private val RailItem = 56.dp
private val RailPill = 32.dp
private val RailItemMargin = 4.dp
private val RailIcon = 20.dp
private val RailBadgeX = 14.dp
private val RailBadgeY = (-4).dp
private val LogOutHeight = 44.dp
private val LogOutIcon = 18.dp
private val LogOutGap = 12.dp
private val LogOutPadding = 16.dp
private const val BRAND_TRACKING_EM = -0.045f
private const val FILL = 1f

/** Tokens used of the plan's allowance, 0–1. */
private fun NomiUsageBrief.share(): Float = if (monthlyTokens <= 0) 0f else (tokensUsed.toFloat() / monthlyTokens).coerceIn(0f, FILL)

/**
 * The navigation drawer (web `NavDrawer.svelte` on the design system's `Sidebar`): the brand with collapse, New chat,
 * then the places, More and the recent chats in one scrolling column, and usage, the person and Log out at the foot.
 * [collapsed] draws the web's icon rail of the same places instead.
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
    val frame = modifier
        .fillMaxHeight()
        .background(MaterialTheme.colorScheme.surfaceContainerLow)
        .statusBarsPadding()
        .navigationBarsPadding()
    val fadeIn = fadeIn(DsMotion.effects())
    val fadeOut = fadeOut(DsMotion.effects())
    AnimatedContent(
        collapsed,
        frame,
        transitionSpec = { fadeIn togetherWith fadeOut },
        label = "drawer-rail",
    ) { rail ->
        if (rail) {
            DrawerRail(items, selected, onSelect, onNewChat, onToggleCollapsed, account, onOpenSettings ?: onOpenAccount)
        } else {
            DrawerPanel(
                items, selected, onSelect, onNewChat, onToggleCollapsed ?: onClose, recents, currentChatId, onOpenRecentChat,
                onSeeAllChats, usage, onUpgrade ?: onOpenSettings, account, onOpenSettings ?: onOpenAccount, onLogOut, footer,
            )
        }
    }
}

@Composable
private fun <K> DrawerPanel(
    items: List<NomiNavItem<K>>,
    selected: K,
    onSelect: (K) -> Unit,
    onNewChat: () -> Unit,
    onCollapse: (() -> Unit)?,
    recents: List<NomiRecentChat>,
    currentChatId: String?,
    onOpenRecentChat: ((String) -> Unit)?,
    onSeeAllChats: (() -> Unit)?,
    usage: NomiUsageBrief?,
    onUsage: (() -> Unit)?,
    account: NomiAccount?,
    onSettings: (() -> Unit)?,
    onLogOut: (() -> Unit)?,
    footer: (@Composable () -> Unit)?,
) {
    var recentsOpen by rememberSaveable { mutableStateOf(true) }
    Column(Modifier.width(SidebarWidth).fillMaxHeight().padding(DrawerPadding), verticalArrangement = Arrangement.spacedBy(ItemGapDp)) {
        Row(Modifier.fillMaxWidth().padding(start = BrandGap, bottom = BrandGap), verticalAlignment = Alignment.CenterVertically) {
            Row(Modifier.weight(FILL), horizontalArrangement = Arrangement.spacedBy(BrandGap), verticalAlignment = Alignment.CenterVertically) {
                Mascot(Crew.Nomi, BrandMascot)
                Text("nomi", style = brandWordmark(), color = Ds.text)
            }
            if (onCollapse != null) DsIconButton(Lucide.PanelLeft, "Collapse sidebar", onCollapse, variant = DsIconButtonVariant.Ghost)
        }
        NewChatButton(onNewChat)
        Spacer(Modifier.height(SectionGap))
        Column(Modifier.weight(FILL).verticalScroll(rememberScrollState()), verticalArrangement = Arrangement.spacedBy(ItemGapDp)) {
            items.forEach { item ->
                NavItem(
                    item.glyph,
                    item.label,
                    onClick = { onSelect(item.key) },
                    count = item.badge.takeIf { it > 0 }?.let(::badgeText),
                    active = item.key == selected,
                )
            }
            if (recents.isNotEmpty()) {
                Spacer(Modifier.height(RecentsTop))
                RecentsList(
                    recents.map { chat ->
                        RecentItem(RecentKind.Chat, chat.title, chat.whenLabel, active = chat.id == currentChatId, onClick = { onOpenRecentChat?.invoke(chat.id) })
                    },
                    open = recentsOpen,
                    onToggle = { recentsOpen = !recentsOpen },
                    onAll = { onSeeAllChats?.invoke() },
                )
            }
        }
        Spacer(Modifier.height(SectionGap))
        if (usage != null) {
            UsageCard(
                usage.planName,
                usage.share(),
                "${formatTokens(usage.tokensUsed)} of ${formatTokens(usage.monthlyTokens)} tokens",
                onOpen = { onUsage?.invoke() },
            )
        }
        footer?.invoke()
        if (account != null) UserRow(account.displayName, onSettings = { onSettings?.invoke() })
        if (onLogOut != null) LogOutRow(onLogOut)
    }
}

/** The drawer's FAB: New chat on the primary container. */
@Composable
private fun NewChatButton(onClick: () -> Unit) {
    Row(
        Modifier.fillMaxWidth().height(NewChatHeight).clip(MaterialTheme.shapes.large).background(MaterialTheme.colorScheme.primaryContainer)
            .clickable(role = Role.Button, onClick = onClick).padding(horizontal = LogOutPadding),
        horizontalArrangement = Arrangement.spacedBy(NewChatGap),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        DsIcon(Lucide.Plus, NewChatIcon, MaterialTheme.colorScheme.onPrimaryContainer)
        Text("New chat", style = MaterialTheme.typography.labelLarge, color = MaterialTheme.colorScheme.onPrimaryContainer)
    }
}

/** `.log-out`: a quiet full-width row. */
@Composable
private fun LogOutRow(onLogOut: () -> Unit) {
    Row(
        Modifier.fillMaxWidth().height(LogOutHeight).clip(CircleShape).clickable(role = Role.Button, onClick = onLogOut).padding(horizontal = LogOutPadding),
        horizontalArrangement = Arrangement.spacedBy(LogOutGap),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        DsIcon(Lucide.Logout, LogOutIcon, Ds.text2)
        Text("Log out", style = MaterialTheme.typography.labelLarge, color = Ds.text2)
    }
}

/** `.rail`: the brand, New chat, the places as pills with their badges, expand, and the person. */
@Composable
private fun <K> DrawerRail(
    items: List<NomiNavItem<K>>,
    selected: K,
    onSelect: (K) -> Unit,
    onNewChat: () -> Unit,
    onExpand: (() -> Unit)?,
    account: NomiAccount?,
    onSettings: (() -> Unit)?,
) {
    Column(
        Modifier.width(RailWidth).fillMaxHeight().padding(top = RailTop, bottom = DrawerPadding),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(RailGap),
    ) {
        Mascot(Crew.Nomi, BrandMascot, label = "Nomi home")
        Box(
            Modifier.size(RailItem).clip(MaterialTheme.shapes.large).background(MaterialTheme.colorScheme.primaryContainer)
                .clickable(role = Role.Button, onClickLabel = "New chat", onClick = onNewChat),
            contentAlignment = Alignment.Center,
        ) { DsIcon(Lucide.Plus, RailIcon, MaterialTheme.colorScheme.onPrimaryContainer) }
        Column(Modifier.weight(FILL).verticalScroll(rememberScrollState()), horizontalAlignment = Alignment.CenterHorizontally) {
            items.forEach { item -> RailItem(item, item.key == selected) { onSelect(item.key) } }
        }
        if (onExpand != null) DsIconButton(Lucide.PanelLeftOpen, "Expand sidebar", onExpand, variant = DsIconButtonVariant.Ghost)
        if (account != null) {
            Box(Modifier.clip(CircleShape).clickable(role = Role.Button, onClickLabel = "Settings") { onSettings?.invoke() }) { DsAvatar(account.displayName) }
        }
    }
}

@Composable
private fun <K> RailItem(item: NomiNavItem<K>, active: Boolean, onClick: () -> Unit) {
    val colors = MaterialTheme.colorScheme
    val fill by animateColorAsState(if (active) colors.secondaryContainer else colors.secondaryContainer.copy(alpha = 0f), DsMotion.fast(), label = "rail-fill")
    Box(
        Modifier.padding(vertical = RailItemMargin).size(RailItem, RailPill).clip(CircleShape).background(fill)
            .semantics { selected = active }
            .clickable(role = Role.Tab, onClickLabel = item.label, onClick = onClick),
        contentAlignment = Alignment.Center,
    ) {
        DsIcon(item.glyph, RailIcon, if (active) colors.onSecondaryContainer else Ds.text2)
        if (item.badge > 0) Badge(Modifier.offset(RailBadgeX, RailBadgeY)) { Text(badgeText(item.badge)) }
    }
}

private val ItemGapDp = 4.dp

/** The wordmark beside the mascot (`font-weight: 800; letter-spacing: -0.045em`). */
@Composable
private fun brandWordmark() = MaterialTheme.typography.titleLarge.copy(fontWeight = FontWeight.ExtraBold, letterSpacing = BRAND_TRACKING_EM.em)

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
            .clipToBounds()
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
