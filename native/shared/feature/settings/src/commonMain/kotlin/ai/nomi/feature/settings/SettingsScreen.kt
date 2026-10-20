package ai.nomi.feature.settings

import ai.nomi.data.DEFAULT_ACCENT_COLOR
import ai.nomi.data.THEME_DARK
import ai.nomi.data.THEME_LIGHT
import ai.nomi.data.THEME_SYSTEM
import ai.nomi.designsystem.component.NomiButtonGroup
import ai.nomi.designsystem.component.NomiFilterChip
import ai.nomi.designsystem.component.NomiGroupOption
import ai.nomi.designsystem.component.NomiListPanel
import ai.nomi.designsystem.component.NomiRow
import ai.nomi.designsystem.component.NomiSectionHeader
import ai.nomi.designsystem.component.NomiSwitch
import ai.nomi.designsystem.icon.NomiGlyph
import ai.nomi.designsystem.icon.NomiIcon
import ai.nomi.designsystem.icon.NomiIcons
import ai.nomi.designsystem.layout.NomiPage
import ai.nomi.designsystem.theme.NomiTheme
import ai.nomi.network.ProfileDto
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle

const val SETTINGS_SCREEN_TAG = "settings-screen"

private val IconBadgeSize = 36.dp
private val AppearanceInnerPadding = 12.dp
private val AppearanceGap = 12.dp
private val ChipGap = 8.dp
private val SectionGap = 16.dp
private val BottomScrollPadding = 24.dp
private val ToggleRowHorizontalPadding = 12.dp
private val ToggleRowVerticalPadding = 8.dp
private val ToggleRowGap = 14.dp
private const val FILL = 1f
private const val TITLE_WEIGHT = 650

private val ThemeModeOptions = listOf(
    NomiGroupOption(THEME_LIGHT, "Light"),
    NomiGroupOption(THEME_DARK, "Dark"),
    NomiGroupOption(THEME_SYSTEM, "System"),
)

private val AccentThemeOptions = listOf(
    "canopy" to "Canopy",
    "coral-reef" to "Coral Reef",
    "borneo-dusk" to "Borneo Dusk",
    "phantom" to "Phantom",
    "senja-jakarta" to "Senja Jakarta",
    "lautan" to "Lautan",
    "matcha" to "Matcha",
    "saffron" to "Saffron",
    "lagoon" to "Lagoon",
    "orchid" to "Orchid",
    "ruby" to "Ruby",
    "graphite" to "Graphite",
)

@Composable
fun SettingsScreen(
    viewModel: SettingsViewModel,
    modifier: Modifier = Modifier,
    onOpenConnections: () -> Unit = {},
) {
    val profile by viewModel.profile.collectAsStateWithLifecycle()
    val theme by viewModel.theme.collectAsStateWithLifecycle()
    val accentColor by viewModel.accentColor.collectAsStateWithLifecycle()
    val homeLayout by viewModel.homeLayout.collectAsStateWithLifecycle()
    val drawerPins by viewModel.drawerPins.collectAsStateWithLifecycle()
    val drawerRecents by viewModel.drawerRecents.collectAsStateWithLifecycle()
    val drawerRecentsLimit by viewModel.drawerRecentsLimit.collectAsStateWithLifecycle()
    val timezone by viewModel.timezone.collectAsStateWithLifecycle()
    val islandEnabled by viewModel.islandEnabled.collectAsStateWithLifecycle()
    val bubbleEnabled by viewModel.bubbleEnabled.collectAsStateWithLifecycle()
    SettingsContent(
        profile = profile,
        onSignOut = viewModel::signOut,
        modifier = modifier,
        theme = theme,
        accentColor = accentColor,
        onSelectTheme = viewModel::selectTheme,
        onSelectAccentColor = viewModel::selectAccentColor,
        homeLayout = homeLayout,
        onSelectHomeLayout = viewModel::selectHomeLayout,
        drawerPins = drawerPins,
        onMovePin = viewModel::movePin,
        onUnpin = viewModel::togglePin,
        onPin = viewModel::togglePin,
        drawerRecents = drawerRecents,
        onToggleDrawerRecents = viewModel::toggleDrawerRecents,
        drawerRecentsLimit = drawerRecentsLimit,
        onSelectDrawerRecentsLimit = viewModel::selectDrawerRecentsLimit,
        timezone = timezone,
        onSelectTimezone = viewModel::selectTimezone,
        islandEnabled = islandEnabled,
        onIslandEnabledChange = viewModel::setIslandEnabled,
        bubbleEnabled = bubbleEnabled,
        onBubbleEnabledChange = viewModel::setBubbleEnabled,
        onOpenConnections = onOpenConnections,
    )
}

/** The account and preferences page: who is signed in, appearance, home layout, companion surfaces, and signing out. */
@OptIn(ExperimentalLayoutApi::class)
@Composable
fun SettingsContent(
    profile: ProfileDto?,
    onSignOut: () -> Unit,
    modifier: Modifier = Modifier,
    theme: String = THEME_SYSTEM,
    accentColor: String = DEFAULT_ACCENT_COLOR,
    onSelectTheme: (String) -> Unit = {},
    onSelectAccentColor: (String) -> Unit = {},
    homeLayout: String = "stage",
    onSelectHomeLayout: (String) -> Unit = {},
    drawerPins: List<String> = emptyList(),
    onMovePin: (from: Int, to: Int) -> Unit = { _, _ -> },
    onUnpin: (String) -> Unit = {},
    onPin: (String) -> Unit = {},
    drawerRecents: Boolean = true,
    onToggleDrawerRecents: (Boolean) -> Unit = {},
    drawerRecentsLimit: Int = 10,
    onSelectDrawerRecentsLimit: (Int) -> Unit = {},
    timezone: String = "UTC",
    onSelectTimezone: (String) -> Unit = {},
    islandEnabled: Boolean = true,
    onIslandEnabledChange: (Boolean) -> Unit = {},
    bubbleEnabled: Boolean = true,
    onBubbleEnabledChange: (Boolean) -> Unit = {},
    onOpenConnections: () -> Unit = {},
) {
    NomiPage(title = "Settings", modifier = modifier.testTag(SETTINGS_SCREEN_TAG)) {
        Column(
            Modifier
                .fillMaxWidth()
                .verticalScroll(rememberScrollState())
                .padding(bottom = BottomScrollPadding),
            verticalArrangement = Arrangement.spacedBy(SectionGap),
        ) {
            val name = profile?.displayName?.takeIf { it.isNotBlank() } ?: profile?.username
            NomiListPanel {
                NomiRow(
                    title = name ?: profile?.email ?: "Your account",
                    line = if (name != null) profile?.email else null,
                    onClick = null,
                    leading = { IconBadge(NomiIcons.Person, MaterialTheme.colorScheme.primaryContainer, MaterialTheme.colorScheme.onPrimaryContainer) },
                )
                NomiRow(
                    title = "Connections",
                    line = "Connected accounts and integrations",
                    onClick = onOpenConnections,
                    leading = { IconBadge(NomiIcons.Link, MaterialTheme.colorScheme.secondaryContainer, MaterialTheme.colorScheme.onSecondaryContainer) },
                )
            }

            NomiSectionHeader("Appearance")
            AppearancePicker(
                mode = theme,
                accentColor = accentColor,
                onSelectMode = onSelectTheme,
                onSelectAccentColor = onSelectAccentColor,
            )

            NomiSectionHeader("Home layout")
            HomeLayoutPicker(
                current = homeLayout,
                onSelectLayout = onSelectHomeLayout,
            )

            NomiSectionHeader("Navigation drawer")
            DrawerCustomizer(
                pins = drawerPins,
                onMovePin = onMovePin,
                onUnpin = onUnpin,
                onPin = onPin,
                recentsEnabled = drawerRecents,
                onToggleRecents = onToggleDrawerRecents,
                recentsLimit = drawerRecentsLimit,
                onSelectRecentsLimit = onSelectDrawerRecentsLimit,
                currentTimezone = timezone,
                onSelectTimezone = onSelectTimezone,
            )

            NomiSectionHeader("Companion")
            NomiListPanel {
                CompanionToggleRow(
                    title = "Dynamic Island",
                    subtitle = "Top-of-screen pill that expands on hover",
                    checked = islandEnabled,
                    onCheckedChange = onIslandEnabledChange,
                )
                CompanionToggleRow(
                    title = "Chat head bubble",
                    subtitle = "Floating bubble for live replies and quick chat",
                    checked = bubbleEnabled,
                    onCheckedChange = onBubbleEnabledChange,
                )
            }

            NomiSectionHeader("Integrations")
            NomiListPanel {
                NomiRow(
                    title = "Connections",
                    line = "Google Workspace, Hevy and health integrations",
                    onClick = onOpenConnections,
                    leading = { IconBadge(NomiIcons.Link, MaterialTheme.colorScheme.secondaryContainer, MaterialTheme.colorScheme.onSecondaryContainer) },
                )
            }

            NomiListPanel {
                NomiRow(
                    title = "Log out",
                    onClick = onSignOut,
                    leading = { IconBadge(NomiIcons.Logout, MaterialTheme.colorScheme.errorContainer, MaterialTheme.colorScheme.onErrorContainer) },
                )
            }
            Text("Nomi for ${platformName()}", style = NomiTheme.metaStyle, color = MaterialTheme.colorScheme.onSurfaceVariant)
        }
    }
}

@Composable
private fun CompanionToggleRow(
    title: String,
    subtitle: String,
    checked: Boolean,
    onCheckedChange: (Boolean) -> Unit,
) {
    val colors = MaterialTheme.colorScheme
    Row(
        Modifier
            .fillMaxWidth()
            .padding(horizontal = ToggleRowHorizontalPadding, vertical = ToggleRowVerticalPadding),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(ToggleRowGap),
    ) {
        Column(Modifier.weight(FILL)) {
            Text(
                title,
                style = MaterialTheme.typography.bodyLarge.copy(fontWeight = FontWeight(TITLE_WEIGHT)),
                color = colors.onSurface,
            )
            Text(
                subtitle,
                style = MaterialTheme.typography.bodyMedium,
                color = colors.onSurfaceVariant,
            )
        }
        NomiSwitch(checked = checked, onCheckedChange = onCheckedChange)
    }
}

@Composable
private fun IconBadge(glyph: NomiGlyph, container: Color, ink: Color) {
    Box(Modifier.size(IconBadgeSize).clip(CircleShape).background(container), contentAlignment = Alignment.Center) {
        NomiIcon(glyph, contentDescription = null, tint = ink)
    }
}
