package ai.nomi.designsystem.ds

import ai.nomi.designsystem.icon.Lucide
import ai.nomi.designsystem.icon.NomiGlyph
import androidx.compose.animation.animateColorAsState
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.semantics.LiveRegionMode
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.liveRegion
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp

/** Nothing here yet, said plainly, with the one thing to do about it. */
@Composable
fun EmptyState(
    icon: NomiGlyph,
    title: String,
    body: String,
    modifier: Modifier = Modifier,
    count: Int? = null,
    actionLabel: String? = null,
    onAction: (() -> Unit)? = null,
) {
    Column(
        modifier.fillMaxWidth().clip(MaterialTheme.shapes.large).background(Ds.surface).dsOutline(Ds.border, MaterialTheme.shapes.large).padding(20.dp),
        verticalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
            DsIcon(icon, 20.dp, Ds.text2)
            Text(title, style = MaterialTheme.typography.titleSmall, color = Ds.text)
            if (count != null) Text(count.toString(), style = Ds.label, color = Ds.text2)
        }
        Text(body, style = Ds.ui, color = Ds.text2)
        if (actionLabel != null && onAction != null) DsLink(actionLabel, onAction)
    }
}

data class ProjectStatus(val kind: StatusKind, val label: String)

/**
 * A project in a list: its tile (its initial on the crew tint), what it is, whether it's live and where, and when
 * it last changed.
 */
@Composable
fun ProjectRow(
    name: String,
    tone: Crew,
    status: ProjectStatus,
    updated: String,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    description: String? = null,
    url: String? = null,
) {
    Row(
        modifier
            .fillMaxWidth()
            .clip(MaterialTheme.shapes.large)
            .background(Ds.surface)
            .clickable(role = Role.Button, onClick = onClick)
            .padding(horizontal = 16.dp, vertical = 12.dp),
        horizontalArrangement = Arrangement.spacedBy(16.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Row(Modifier.weight(1f), horizontalArrangement = Arrangement.spacedBy(12.dp), verticalAlignment = Alignment.CenterVertically) {
            Box(Modifier.size(40.dp).clip(MaterialTheme.shapes.medium).background(tone.tint), contentAlignment = Alignment.Center) {
                Text(name.take(1).uppercase(), style = MaterialTheme.typography.titleMedium, color = Ds.text)
            }
            Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
                DsText(name, Ds.body, Ds.text, singleLine = true, weight = FontWeight.Medium)
                if (description != null) DsText(description, Ds.ui, Ds.text2, singleLine = true)
            }
        }
        Column(horizontalAlignment = Alignment.End, verticalArrangement = Arrangement.spacedBy(2.dp)) {
            DsStatus(status.kind, status.label)
            if (url != null) DsText(url, Ds.label, Ds.text2, singleLine = true)
            Text(updated, style = Ds.label, color = Ds.text3)
        }
    }
}

/** Stands in while something loads, and says what ("loading 4 projects…"). Still, not shimmering. */
@Composable
fun SkeletonRow(label: String, modifier: Modifier = Modifier) {
    Column(modifier.fillMaxWidth().semantics { liveRegion = LiveRegionMode.Polite }, verticalArrangement = Arrangement.spacedBy(8.dp)) {
        Row(horizontalArrangement = Arrangement.spacedBy(12.dp), verticalAlignment = Alignment.CenterVertically) {
            Box(Modifier.size(40.dp).clip(MaterialTheme.shapes.medium).background(Ds.surface2))
            Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                Box(Modifier.fillMaxWidth(SHORT_LINE).height(10.dp).clip(DsShapes.full).background(Ds.surface2))
                Box(Modifier.fillMaxWidth().height(10.dp).clip(DsShapes.full).background(Ds.surface2))
            }
        }
        Text(label, style = Ds.label, color = Ds.text2)
    }
}

private const val SHORT_LINE = 0.4f

/** Something to ask, with who it would go to ("→ dana"). */
@Composable
fun SuggestionRow(agent: Crew, label: String, onClick: () -> Unit, modifier: Modifier = Modifier, route: String? = null) {
    Row(
        modifier
            .fillMaxWidth()
            .heightIn(min = 48.dp)
            .clip(MaterialTheme.shapes.medium)
            .clickable(role = Role.Button, onClick = onClick)
            .padding(horizontal = 12.dp, vertical = 8.dp),
        horizontalArrangement = Arrangement.spacedBy(12.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Mascot(agent, 20.dp)
        DsText(label, Ds.ui, Ds.text, Modifier.weight(1f))
        if (route != null) Text(route, style = Ds.label, color = Ds.text3)
    }
}

/** Names a group of drawer rows (Talk, Build, Life) in the meta face; [trailing] sits at the row's end. */
@Composable
fun NavGroupLabel(label: String, modifier: Modifier = Modifier, trailing: (@Composable () -> Unit)? = null) {
    Row(modifier.fillMaxWidth().heightIn(min = 36.dp).padding(start = 16.dp, end = 8.dp), verticalAlignment = Alignment.CenterVertically) {
        Text(label.uppercase(), style = Ds.labelCaps, color = Ds.text2, modifier = Modifier.weight(1f))
        trailing?.invoke()
    }
}

/** An M3 navigation-drawer item: active is the secondary-container pill, which fades in. */
@Composable
fun NavItem(icon: NomiGlyph, label: String, onClick: () -> Unit, modifier: Modifier = Modifier, count: String? = null, active: Boolean = false, dense: Boolean = true) {
    val fill by animateColorAsState(
        if (active) MaterialTheme.colorScheme.secondaryContainer else Color.Transparent, DsMotion.fast(), label = "nav-fill",
    )
    val ink = if (active) MaterialTheme.colorScheme.onSecondaryContainer else Ds.text2
    Row(
        modifier
            .fillMaxWidth()
            .height(if (dense) 44.dp else 56.dp)
            .clip(DsShapes.full)
            .background(fill)
            .clickable(role = Role.Tab, onClick = onClick)
            .padding(horizontal = 16.dp),
        horizontalArrangement = Arrangement.spacedBy(12.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        DsIcon(icon, 20.dp, ink)
        Text(label, style = MaterialTheme.typography.labelLarge, color = if (active) ink else Ds.text, modifier = Modifier.weight(1f))
        if (count != null) Text(count, style = Ds.label, color = ink)
    }
}

enum class RecentKind { Chat, Project }

/** A recent chat or project in the drawer, with how long ago in mono. */
@Composable
fun RecentRow(kind: RecentKind, title: String, whenLabel: String, onClick: () -> Unit, modifier: Modifier = Modifier, active: Boolean = false) {
    Row(
        modifier
            .fillMaxWidth()
            .heightIn(min = 40.dp)
            .clip(DsShapes.full)
            .background(if (active) MaterialTheme.colorScheme.secondaryContainer else Color.Transparent)
            .clickable(role = Role.Button, onClick = onClick)
            .padding(horizontal = 16.dp),
        horizontalArrangement = Arrangement.spacedBy(12.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        DsIcon(if (kind == RecentKind.Chat) Lucide.Message else Lucide.Folder, 18.dp, Ds.text3)
        DsText(title, Ds.ui, Ds.text, Modifier.weight(1f), singleLine = true)
        Text(whenLabel, style = Ds.label, color = Ds.text3)
    }
}

data class RecentItem(val kind: RecentKind, val title: String, val whenLabel: String, val active: Boolean = false, val onClick: () -> Unit = {})

/** The drawer's recent chats and projects, folding open and shut, ending in a link to all chats. */
@Composable
fun RecentsList(items: List<RecentItem>, open: Boolean, onToggle: () -> Unit, onAll: () -> Unit, modifier: Modifier = Modifier) {
    Column(modifier.fillMaxWidth()) {
        Row(
            Modifier.fillMaxWidth().heightIn(min = 36.dp).clip(DsShapes.full).clickable(role = Role.Button, onClick = onToggle).padding(horizontal = 16.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text("RECENTS", style = Ds.labelCaps, color = Ds.text2, modifier = Modifier.weight(1f))
            DsChevron(open, 18.dp, Ds.text2)
        }
        DsFold(open) {
            Column { items.forEach { RecentRow(it.kind, it.title, it.whenLabel, it.onClick, active = it.active) } }
        }
        Row(
            Modifier.fillMaxWidth().heightIn(min = 40.dp).clip(DsShapes.full).clickable(role = Role.Button, onClick = onAll).padding(horizontal = 16.dp),
            horizontalArrangement = Arrangement.spacedBy(8.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Text("All chats", style = MaterialTheme.typography.labelLarge, color = Ds.accent)
            DsIcon(Lucide.ArrowRight, 18.dp, Ds.accent)
        }
    }
}

/** The plan and how much of it is used this month, at the foot of the drawer. */
@Composable
fun UsageCard(plan: String, used: Float, caption: String, onOpen: () -> Unit, modifier: Modifier = Modifier) {
    Column(
        modifier.fillMaxWidth().clip(MaterialTheme.shapes.large).background(Ds.surface).padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        Meter(plan, used, caption = caption, roomy = true)
        DsLink("Plan & usage", onOpen)
    }
}

/** The person at the foot of the drawer, with the way into settings. */
@Composable
fun UserRow(name: String, onSettings: () -> Unit, modifier: Modifier = Modifier) {
    Row(modifier.fillMaxWidth().padding(horizontal = 12.dp, vertical = 4.dp), horizontalArrangement = Arrangement.spacedBy(12.dp), verticalAlignment = Alignment.CenterVertically) {
        DsAvatar(name)
        DsText(name, MaterialTheme.typography.labelLarge, Ds.text, Modifier.weight(1f), singleLine = true)
        DsIconButton(Lucide.Settings, "Settings", onSettings, variant = DsIconButtonVariant.Ghost)
    }
}

/** Which workspace's projects the Build group shows ("T Trian's space ↕"); opens the switcher. */
@Composable
fun WorkspaceChip(name: String, onClick: () -> Unit, modifier: Modifier = Modifier) {
    Row(
        modifier.height(28.dp).clip(DsShapes.full).clickable(role = Role.DropdownList, onClick = onClick).padding(start = 4.dp, end = 8.dp),
        horizontalArrangement = Arrangement.spacedBy(6.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Box(Modifier.size(20.dp).clip(DsShapes.xs).background(MaterialTheme.colorScheme.secondaryContainer), contentAlignment = Alignment.Center) {
            Text(name.take(1).uppercase(), style = Ds.micro, color = MaterialTheme.colorScheme.onSecondaryContainer)
        }
        Text(name, style = Ds.caption, color = Ds.text2)
        DsIcon(Lucide.ChevronsUpDown, 14.dp, Ds.text2)
    }
}

/**
 * The app's navigation drawer (web Sidebar): brand and collapse, New chat as the drawer's FAB, the [sections]
 * (NavItems and group labels), [recents], then [footer] (usage and the person).
 */
@Composable
fun DsSidebar(
    onNewChat: () -> Unit,
    modifier: Modifier = Modifier,
    newChatShortcut: String? = "⌘K",
    onCollapse: (() -> Unit)? = null,
    recents: (@Composable ColumnScope.() -> Unit)? = null,
    footer: (@Composable ColumnScope.() -> Unit)? = null,
    sections: @Composable ColumnScope.() -> Unit,
) {
    Column(modifier.width(SIDEBAR_WIDTH).padding(12.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
        Row(Modifier.fillMaxWidth().padding(start = 8.dp, bottom = 8.dp), verticalAlignment = Alignment.CenterVertically) {
            Row(Modifier.weight(1f), horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
                Mascot(Crew.Nomi, 32.dp)
                Text("nomi", style = MaterialTheme.typography.titleLarge, color = Ds.text)
            }
            if (onCollapse != null) DsIconButton(Lucide.PanelLeft, "Collapse sidebar", onCollapse, variant = DsIconButtonVariant.Ghost)
        }
        Row(
            Modifier.fillMaxWidth().height(56.dp).clip(MaterialTheme.shapes.large).background(MaterialTheme.colorScheme.primaryContainer)
                .clickable(role = Role.Button, onClick = onNewChat).padding(horizontal = 16.dp),
            horizontalArrangement = Arrangement.spacedBy(12.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            DsIcon(Lucide.Plus, 20.dp, MaterialTheme.colorScheme.onPrimaryContainer)
            Text("New chat", style = MaterialTheme.typography.labelLarge, color = MaterialTheme.colorScheme.onPrimaryContainer, modifier = Modifier.weight(1f))
            if (newChatShortcut != null) Text(newChatShortcut, style = Ds.label, color = MaterialTheme.colorScheme.onPrimaryContainer)
        }
        Spacer(Modifier.height(8.dp))
        sections()
        if (recents != null) {
            Spacer(Modifier.height(8.dp))
            recents()
        }
        if (footer != null) {
            Spacer(Modifier.height(8.dp))
            footer()
        }
    }
}

private val SIDEBAR_WIDTH = 300.dp

data class SettingsNavItem(val icon: NomiGlyph, val label: String, val count: String? = null, val onClick: () -> Unit = {})
data class SettingsNavGroup(val label: String, val items: List<SettingsNavItem>)

/**
 * The settings drawer (web SettingsNav): back to Nomi, search, the groups of settings, then the person with
 * Log out and the app's version at the foot.
 */
@Composable
fun SettingsNav(
    groups: List<SettingsNavGroup>,
    active: String?,
    userName: String,
    query: String,
    onQuery: (String) -> Unit,
    onBack: () -> Unit,
    modifier: Modifier = Modifier,
    userDetail: String? = null,
    version: String? = null,
    onLogout: (() -> Unit)? = null,
) {
    Column(modifier.width(SIDEBAR_WIDTH).padding(12.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
        DrawerAction(Lucide.ArrowLeft, "Back to Nomi", "esc", onBack)
        DsSearch(query, onQuery, "Search settings", shortcut = "/", plain = true)
        groups.forEach { group ->
            NavGroupLabel(group.label, Modifier.padding(top = 8.dp))
            group.items.forEach { NavItem(it.icon, it.label, it.onClick, count = it.count, active = it.label == active) }
        }
        DsRule(Modifier.padding(vertical = 8.dp))
        Row(Modifier.padding(horizontal = 12.dp), horizontalArrangement = Arrangement.spacedBy(12.dp), verticalAlignment = Alignment.CenterVertically) {
            DsAvatar(userName)
            Column {
                Text(userName, style = MaterialTheme.typography.labelLarge, color = Ds.text)
                if (userDetail != null) Text(userDetail, style = Ds.label, color = Ds.text2)
            }
        }
        if (onLogout != null) DrawerAction(Lucide.Logout, "Log out", version, onLogout)
        else if (version != null) Text(version, style = Ds.label, color = Ds.text2, modifier = Modifier.padding(horizontal = 16.dp))
    }
}

@Composable
private fun DrawerAction(icon: NomiGlyph, label: String, meta: String?, onClick: () -> Unit) {
    Row(
        Modifier.fillMaxWidth().height(44.dp).clip(DsShapes.full).clickable(role = Role.Button, onClick = onClick).padding(horizontal = 16.dp),
        horizontalArrangement = Arrangement.spacedBy(12.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        DsIcon(icon, 20.dp, Ds.text2)
        Text(label, style = MaterialTheme.typography.labelLarge, color = Ds.text, modifier = Modifier.weight(1f))
        if (meta != null) Text(meta, style = Ds.label, color = Ds.text2)
    }
}
