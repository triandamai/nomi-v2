package ai.nomi.feature.settings

import ai.nomi.designsystem.component.NomiButtonGroup
import ai.nomi.designsystem.component.NomiGroupOption
import ai.nomi.designsystem.component.NomiListPanel
import ai.nomi.designsystem.component.NomiRow
import ai.nomi.designsystem.component.NomiSectionHeader
import ai.nomi.designsystem.component.NomiSwitch
import ai.nomi.designsystem.icon.NomiIcon
import ai.nomi.designsystem.icon.NomiIcons
import ai.nomi.designsystem.navigation.NomiFeatures
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp

private val RecentsLimitOptions = listOf(
    NomiGroupOption("5", "5"),
    NomiGroupOption("10", "10"),
    NomiGroupOption("15", "15"),
    NomiGroupOption("20", "20"),
)

val CommonTimezones = listOf(
    "UTC",
    "America/New_York",
    "America/Chicago",
    "America/Denver",
    "America/Los_Angeles",
    "Europe/London",
    "Europe/Paris",
    "Europe/Berlin",
    "Asia/Tokyo",
    "Asia/Singapore",
    "Asia/Jakarta",
    "Asia/Shanghai",
    "Australia/Sydney",
)

@Composable
fun DrawerCustomizer(
    pins: List<String>,
    onMovePin: (from: Int, to: Int) -> Unit,
    onUnpin: (id: String) -> Unit,
    onPin: (id: String) -> Unit,
    recentsEnabled: Boolean,
    onToggleRecents: (Boolean) -> Unit,
    recentsLimit: Int,
    onSelectRecentsLimit: (Int) -> Unit,
    currentTimezone: String,
    onSelectTimezone: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    val colors = MaterialTheme.colorScheme
    val available = NomiFeatures.PinnableIds.filter { it !in pins }

    Column(modifier = modifier.fillMaxWidth(), verticalArrangement = Arrangement.spacedBy(16.dp)) {
        // --- 1. Permanent Section ---
        NomiSectionHeader("Always in drawer")
        NomiListPanel {
            NomiFeatures.PermanentIds.forEach { id ->
                val feat = NomiFeatures.All[id] ?: return@forEach
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .height(48.dp)
                        .padding(horizontal = 14.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(12.dp),
                ) {
                    NomiIcon(NomiIcons.Lock, contentDescription = "Locked", tint = colors.onSurfaceVariant, size = 18.dp)
                    NomiIcon(feat.glyph, contentDescription = null, tint = colors.primary, size = 20.dp)
                    Text(
                        feat.label,
                        style = MaterialTheme.typography.bodyLarge.copy(fontWeight = FontWeight.SemiBold),
                        color = colors.onSurface,
                    )
                }
            }
        }

        // --- 2. Pinned Items Section ---
        NomiSectionHeader("Pinned features")
        if (pins.isEmpty()) {
            Text(
                "No features pinned. Add features below.",
                style = MaterialTheme.typography.bodyMedium,
                color = colors.onSurfaceVariant,
                modifier = Modifier.padding(horizontal = 8.dp),
            )
        } else {
            NomiListPanel {
                pins.forEachIndexed { index, id ->
                    val feat = NomiFeatures.All[id] ?: return@forEachIndexed
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .height(52.dp)
                            .padding(horizontal = 12.dp),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(10.dp),
                    ) {
                        NomiIcon(NomiIcons.DragHandle, contentDescription = null, tint = colors.onSurfaceVariant, size = 18.dp)
                        NomiIcon(feat.glyph, contentDescription = null, tint = colors.onSurface, size = 20.dp)
                        Text(
                            feat.label,
                            style = MaterialTheme.typography.bodyLarge.copy(fontWeight = FontWeight.Medium),
                            color = colors.onSurface,
                            modifier = Modifier.weight(1f),
                        )
                        IconButton(
                            onClick = { if (index > 0) onMovePin(index, index - 1) },
                            enabled = index > 0,
                            modifier = Modifier.size(32.dp),
                        ) {
                            NomiIcon(
                                NomiIcons.ChevronUp,
                                contentDescription = "Move ${feat.label} up",
                                tint = if (index > 0) colors.onSurfaceVariant else colors.onSurfaceVariant.copy(alpha = 0.38f),
                                size = 18.dp,
                            )
                        }
                        IconButton(
                            onClick = { if (index < pins.size - 1) onMovePin(index, index + 1) },
                            enabled = index < pins.size - 1,
                            modifier = Modifier.size(32.dp),
                        ) {
                            NomiIcon(
                                NomiIcons.ChevronDown,
                                contentDescription = "Move ${feat.label} down",
                                tint = if (index < pins.size - 1) colors.onSurfaceVariant else colors.onSurfaceVariant.copy(alpha = 0.38f),
                                size = 18.dp,
                            )
                        }
                        IconButton(
                            onClick = { onUnpin(id) },
                            modifier = Modifier.size(32.dp),
                        ) {
                            NomiIcon(
                                NomiIcons.Close,
                                contentDescription = "Remove ${feat.label} from drawer",
                                tint = colors.onSurfaceVariant,
                                size = 18.dp,
                            )
                        }
                    }
                }
            }
        }

        // --- 3. Available Section ---
        NomiSectionHeader("Available features")
        if (available.isEmpty()) {
            Text(
                "All available features are pinned.",
                style = MaterialTheme.typography.bodyMedium,
                color = colors.onSurfaceVariant,
                modifier = Modifier.padding(horizontal = 8.dp),
            )
        } else {
            NomiListPanel {
                available.forEach { id ->
                    val feat = NomiFeatures.All[id] ?: return@forEach
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .height(52.dp)
                            .padding(horizontal = 14.dp),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.spacedBy(12.dp),
                    ) {
                        NomiIcon(feat.glyph, contentDescription = null, tint = colors.onSurfaceVariant, size = 20.dp)
                        Column(modifier = Modifier.weight(1f)) {
                            Text(
                                feat.label,
                                style = MaterialTheme.typography.bodyLarge.copy(fontWeight = FontWeight.Medium),
                                color = colors.onSurface,
                            )
                            Text(
                                feat.description,
                                style = MaterialTheme.typography.bodySmall,
                                color = colors.onSurfaceVariant,
                                maxLines = 1,
                            )
                        }
                        IconButton(
                            onClick = { onPin(id) },
                            modifier = Modifier.size(32.dp),
                        ) {
                            NomiIcon(
                                NomiIcons.Plus,
                                contentDescription = "Add ${feat.label} to drawer",
                                tint = colors.primary,
                                size = 20.dp,
                            )
                        }
                    }
                }
            }
        }

        // --- 4. Recents Section ---
        NomiSectionHeader("Recent chats")
        NomiListPanel {
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 14.dp, vertical = 10.dp),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.SpaceBetween,
            ) {
                Column(Modifier.weight(1f).padding(end = 8.dp)) {
                    Text(
                        "Show recent chats",
                        style = MaterialTheme.typography.bodyLarge.copy(fontWeight = FontWeight.SemiBold),
                        color = colors.onSurface,
                    )
                    Text(
                        "Keep your recent conversations handy in the sidebar",
                        style = MaterialTheme.typography.bodyMedium,
                        color = colors.onSurfaceVariant,
                    )
                }
                NomiSwitch(checked = recentsEnabled, onCheckedChange = onToggleRecents)
            }
            if (recentsEnabled) {
                Column(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(horizontal = 14.dp, vertical = 8.dp),
                    verticalArrangement = Arrangement.spacedBy(8.dp),
                ) {
                    Text(
                        "Number of recents to show",
                        style = MaterialTheme.typography.bodyMedium,
                        color = colors.onSurfaceVariant,
                    )
                    NomiButtonGroup(
                        options = RecentsLimitOptions,
                        selected = recentsLimit.toString(),
                        onSelect = { onSelectRecentsLimit(it.toIntOrNull() ?: 10) },
                        modifier = Modifier.fillMaxWidth(),
                    )
                }
            }
        }

        // --- 5. Timezone Section ---
        NomiSectionHeader("Timezone")
        NomiListPanel {
            var tzExpanded by remember { mutableStateOf(false) }
            Box(Modifier.fillMaxWidth()) {
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .clickable { tzExpanded = true }
                        .padding(horizontal = 14.dp, vertical = 12.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.SpaceBetween,
                ) {
                    Column(Modifier.weight(1f)) {
                        Text(
                            "Timezone",
                            style = MaterialTheme.typography.bodyLarge.copy(fontWeight = FontWeight.SemiBold),
                            color = colors.onSurface,
                        )
                        Text(
                            currentTimezone,
                            style = MaterialTheme.typography.bodyMedium,
                            color = colors.primary,
                        )
                    }
                    NomiIcon(NomiIcons.ChevronDown, contentDescription = "Choose timezone", tint = colors.onSurfaceVariant, size = 18.dp)
                }
                DropdownMenu(
                    expanded = tzExpanded,
                    onDismissRequest = { tzExpanded = false },
                ) {
                    CommonTimezones.forEach { tz ->
                        DropdownMenuItem(
                            text = {
                                Text(
                                    tz,
                                    style = MaterialTheme.typography.bodyMedium.copy(
                                        fontWeight = if (tz == currentTimezone) FontWeight.Bold else FontWeight.Normal
                                    ),
                                    color = if (tz == currentTimezone) colors.primary else colors.onSurface,
                                )
                            },
                            onClick = {
                                tzExpanded = false
                                onSelectTimezone(tz)
                            },
                        )
                    }
                }
            }
        }
    }
}
