package ai.nomi.feature.settings

import ai.nomi.data.THEME_DARK
import ai.nomi.data.THEME_LIGHT
import ai.nomi.data.THEME_SYSTEM
import ai.nomi.designsystem.component.ButtonGroupOption
import ai.nomi.designsystem.component.NomiButtonGroup
import ai.nomi.designsystem.icon.NomiIcon
import ai.nomi.designsystem.icon.NomiIcons
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.animateColorAsState
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
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
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.role
import androidx.compose.ui.semantics.selected
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

data class ThemeSwatchColors(
    val primary: Color,
    val primaryContainer: Color,
    val tertiary: Color,
    val surface: Color,
    val surfaceContainerLowest: Color,
    val onSurface: Color,
)

data class ThemeOption(
    val id: String,
    val label: String,
    val description: String,
    val light: ThemeSwatchColors,
    val dark: ThemeSwatchColors,
)

val NOMI_THEMES: List<ThemeOption> = listOf(
    ThemeOption(
        id = "canopy",
        label = "Canopy",
        description = "Nomi’s base theme: eucalyptus and tropical rain.",
        light = ThemeSwatchColors(
            primary = Color(0xFF0B6B4A),
            primaryContainer = Color(0xFFB9F5C8),
            tertiary = Color(0xFFA33A12),
            surface = Color(0xFFF4F7F2),
            surfaceContainerLowest = Color(0xFFFFFFFF),
            onSurface = Color(0xFF0F1A14),
        ),
        dark = ThemeSwatchColors(
            primary = Color(0xFF7CE0A3),
            primaryContainer = Color(0xFF0B5236),
            tertiary = Color(0xFFFFB596),
            surface = Color(0xFF0D1511),
            surfaceContainerLowest = Color(0xFF08100C),
            onSurface = Color(0xFFE3EDE5),
        ),
    ),
    ThemeOption(
        id = "coral-reef",
        label = "Coral Reef",
        description = "Warm terracotta and teal tidepools.",
        light = ThemeSwatchColors(
            primary = Color(0xFFA43B32),
            primaryContainer = Color(0xFFFFDAD5),
            tertiary = Color(0xFF006A66),
            surface = Color(0xFFFFF8F7),
            surfaceContainerLowest = Color(0xFFFFFFFF),
            onSurface = Color(0xFF221A19),
        ),
        dark = ThemeSwatchColors(
            primary = Color(0xFFFFB4AA),
            primaryContainer = Color(0xFF84241E),
            tertiary = Color(0xFF80D5CF),
            surface = Color(0xFF191211),
            surfaceContainerLowest = Color(0xFF130C0C),
            onSurface = Color(0xFFEFDFDD),
        ),
    ),
    ThemeOption(
        id = "borneo-dusk",
        label = "Borneo Dusk",
        description = "Deep amber sunsets over the rainforest canopy.",
        light = ThemeSwatchColors(
            primary = Color(0xFF924C00),
            primaryContainer = Color(0xFFFFDCC4),
            tertiary = Color(0xFF3D6846),
            surface = Color(0xFFFFF8F5),
            surfaceContainerLowest = Color(0xFFFFFFFF),
            onSurface = Color(0xFF221A14),
        ),
        dark = ThemeSwatchColors(
            primary = Color(0xFFFFB780),
            primaryContainer = Color(0xFF6F3800),
            tertiary = Color(0xFFA3D2A8),
            surface = Color(0xFF19120D),
            surfaceContainerLowest = Color(0xFF140D08),
            onSurface = Color(0xFFF0DFD6),
        ),
    ),
    ThemeOption(
        id = "phantom",
        label = "Phantom",
        description = "Smoky amethyst and misty quartz.",
        light = ThemeSwatchColors(
            primary = Color(0xFF625886),
            primaryContainer = Color(0xFFE7DEFF),
            tertiary = Color(0xFF50606F),
            surface = Color(0xFFFDF8FB),
            surfaceContainerLowest = Color(0xFFFFFFFF),
            onSurface = Color(0xFF1C1B1E),
        ),
        dark = ThemeSwatchColors(
            primary = Color(0xFFCCBFF4),
            primaryContainer = Color(0xFF4A406C),
            tertiary = Color(0xFFB8C8D9),
            surface = Color(0xFF141315),
            surfaceContainerLowest = Color(0xFF0F0E10),
            onSurface = Color(0xFFE6E1E4),
        ),
    ),
    ThemeOption(
        id = "senja-jakarta",
        label = "Senja Jakarta",
        description = "Electric magenta and golden dusk light.",
        light = ThemeSwatchColors(
            primary = Color(0xFFA2346A),
            primaryContainer = Color(0xFFFFD9E5),
            tertiary = Color(0xFF914D00),
            surface = Color(0xFFFFF8F8),
            surfaceContainerLowest = Color(0xFFFFFFFF),
            onSurface = Color(0xFF21191C),
        ),
        dark = ThemeSwatchColors(
            primary = Color(0xFFFFB0CE),
            primaryContainer = Color(0xFF841A51),
            tertiary = Color(0xFFFFB77D),
            surface = Color(0xFF191114),
            surfaceContainerLowest = Color(0xFF130C0F),
            onSurface = Color(0xFFEEDFE2),
        ),
    ),
    ThemeOption(
        id = "lautan",
        label = "Lautan",
        description = "Deep maritime navy with cresting wavefoam.",
        light = ThemeSwatchColors(
            primary = Color(0xFF005DB7),
            primaryContainer = Color(0xFFD6E3FF),
            tertiary = Color(0xFF006B5F),
            surface = Color(0xFFF9F9FF),
            surfaceContainerLowest = Color(0xFFFFFFFF),
            onSurface = Color(0xFF171C25),
        ),
        dark = ThemeSwatchColors(
            primary = Color(0xFFA9C7FF),
            primaryContainer = Color(0xFF00468C),
            tertiary = Color(0xFF55DBC7),
            surface = Color(0xFF0F131C),
            surfaceContainerLowest = Color(0xFF0A0E17),
            onSurface = Color(0xFFDFE2EF),
        ),
    ),
    ThemeOption(
        id = "matcha",
        label = "Matcha",
        description = "Ceremonial tea green with earthy roasted warmth.",
        light = ThemeSwatchColors(
            primary = Color(0xFF4A6800),
            primaryContainer = Color(0xFFB4F700),
            tertiary = Color(0xFF934B00),
            surface = Color(0xFFF8FBE7),
            surfaceContainerLowest = Color(0xFFFFFFFF),
            onSurface = Color(0xFF191D11),
        ),
        dark = ThemeSwatchColors(
            primary = Color(0xFF9ED900),
            primaryContainer = Color(0xFF374E00),
            tertiary = Color(0xFFFFB781),
            surface = Color(0xFF111509),
            surfaceContainerLowest = Color(0xFF0C1005),
            onSurface = Color(0xFFE1E4D1),
        ),
    ),
    ThemeOption(
        id = "saffron",
        label = "Saffron",
        description = "Sun-dried marigold and spiced turmeric.",
        light = ThemeSwatchColors(
            primary = Color(0xFF7C5800),
            primaryContainer = Color(0xFFFFDEA8),
            tertiary = Color(0xFF704DA0),
            surface = Color(0xFFFFF8F3),
            surfaceContainerLowest = Color(0xFFFFFFFF),
            onSurface = Color(0xFF221A0D),
        ),
        dark = ThemeSwatchColors(
            primary = Color(0xFFFFBA22),
            primaryContainer = Color(0xFF5E4200),
            tertiary = Color(0xFFD8B9FF),
            surface = Color(0xFF191206),
            surfaceContainerLowest = Color(0xFF140D03),
            onSurface = Color(0xFFF0E0CA),
        ),
    ),
    ThemeOption(
        id = "lagoon",
        label = "Lagoon",
        description = "Glacial turquoise and coral sand.",
        light = ThemeSwatchColors(
            primary = Color(0xFF006874),
            primaryContainer = Color(0xFF97F0FF),
            tertiary = Color(0xFF9D4223),
            surface = Color(0xFFEEFCFE),
            surfaceContainerLowest = Color(0xFFFFFFFF),
            onSurface = Color(0xFF111D20),
        ),
        dark = ThemeSwatchColors(
            primary = Color(0xFF00DAF1),
            primaryContainer = Color(0xFF004F58),
            tertiary = Color(0xFFFFB59D),
            surface = Color(0xFF091517),
            surfaceContainerLowest = Color(0xFF051012),
            onSurface = Color(0xFFD7E5E7),
        ),
    ),
    ThemeOption(
        id = "orchid",
        label = "Orchid",
        description = "Vibrant botanical violet and wild petals.",
        light = ThemeSwatchColors(
            primary = Color(0xFF9900CF),
            primaryContainer = Color(0xFFF8D8FF),
            tertiary = Color(0xFF9B432C),
            surface = Color(0xFFFFF7FB),
            surfaceContainerLowest = Color(0xFFFFFFFF),
            onSurface = Color(0xFF201922),
        ),
        dark = ThemeSwatchColors(
            primary = Color(0xFFECB2FF),
            primaryContainer = Color(0xFF74009F),
            tertiary = Color(0xFFFFB4A1),
            surface = Color(0xFF18111A),
            surfaceContainerLowest = Color(0xFF130C15),
            onSurface = Color(0xFFECDEEC),
        ),
    ),
    ThemeOption(
        id = "ruby",
        label = "Ruby",
        description = "Crimson gemstone and polished carnelian.",
        light = ThemeSwatchColors(
            primary = Color(0xFFC00016),
            primaryContainer = Color(0xFFFFDAD6),
            tertiary = Color(0xFF865300),
            surface = Color(0xFFFFF8F7),
            surfaceContainerLowest = Color(0xFFFFFFFF),
            onSurface = Color(0xFF271816),
        ),
        dark = ThemeSwatchColors(
            primary = Color(0xFFFFB4AC),
            primaryContainer = Color(0xFF93000E),
            tertiary = Color(0xFFFFB963),
            surface = Color(0xFF1E100E),
            surfaceContainerLowest = Color(0xFF180B09),
            onSurface = Color(0xFFF9DCD9),
        ),
    ),
    ThemeOption(
        id = "graphite",
        label = "Graphite",
        description = "Neutral slate, cool titanium and architectural zinc.",
        light = ThemeSwatchColors(
            primary = Color(0xFF55606A),
            primaryContainer = Color(0xFFD8E4F0),
            tertiary = Color(0xFF725B2E),
            surface = Color(0xFFFBF9FA),
            surfaceContainerLowest = Color(0xFFFFFFFF),
            onSurface = Color(0xFF1B1C1D),
        ),
        dark = ThemeSwatchColors(
            primary = Color(0xFFBCC8D3),
            primaryContainer = Color(0xFF3D4852),
            tertiary = Color(0xFFE1C28C),
            surface = Color(0xFF131314),
            surfaceContainerLowest = Color(0xFF0D0E0F),
            onSurface = Color(0xFFE4E2E3),
        ),
    ),
)

private val Modes = listOf(
    THEME_LIGHT to "Light",
    THEME_DARK to "Dark",
    THEME_SYSTEM to "System",
)

@Composable
fun AppearancePicker(
    mode: String,
    accentColor: String,
    onSelectMode: (String) -> Unit,
    onSelectAccentColor: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    var expandedAll by remember { mutableStateOf(false) }

    Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(20.dp)) {
        // --- 1. Mode Section ---
        Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
            Text(
                text = "MODE",
                style = MaterialTheme.typography.labelSmall.copy(
                    fontFamily = androidx.compose.ui.text.font.FontFamily.Monospace,
                    letterSpacing = 0.08.sp,
                ),
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
            NomiButtonGroup(
                options = listOf(
                    ButtonGroupOption(THEME_LIGHT, "Light"),
                    ButtonGroupOption(THEME_DARK, "Dark"),
                    ButtonGroupOption(THEME_SYSTEM, "System"),
                ),
                selected = mode,
                onSelect = onSelectMode,
            )
            Text(
                text = "Follows your system theme when set to System.",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }

        // --- 2. Color Theme Section ---
        Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
            Text(
                text = "COLOR THEME",
                style = MaterialTheme.typography.labelSmall.copy(
                    fontFamily = androidx.compose.ui.text.font.FontFamily.Monospace,
                    letterSpacing = 0.08.sp,
                ),
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )

            val initialVisible = 5
            val shown = remember(accentColor, expandedAll) {
                if (expandedAll) {
                    NOMI_THEMES
                } else {
                    val first = NOMI_THEMES.take(initialVisible)
                    if (first.any { it.id == accentColor }) {
                        first
                    } else {
                        val active = NOMI_THEMES.find { it.id == accentColor }
                        if (active != null) first.dropLast(1) + active else first
                    }
                }
            }

            // Grid of theme tiles
            Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
                val chunked = shown.chunked(2)
                chunked.forEach { rowThemes ->
                    Row(
                        modifier = Modifier.fillMaxWidth(),
                        horizontalArrangement = Arrangement.spacedBy(12.dp),
                    ) {
                        rowThemes.forEach { theme ->
                            ThemeTile(
                                option = theme,
                                selected = accentColor == theme.id,
                                onSelect = { onSelectAccentColor(theme.id) },
                                modifier = Modifier.weight(1f),
                            )
                        }
                        if (rowThemes.size == 1) {
                            Spacer(Modifier.weight(1f))
                        }
                    }
                }

                if (!expandedAll && NOMI_THEMES.size > initialVisible) {
                    val remainingCount = NOMI_THEMES.size - shown.size
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .clip(RoundedCornerShape(20.dp))
                            .background(MaterialTheme.colorScheme.surfaceContainerLow)
                            .semantics(mergeDescendants = true) {
                                this.role = Role.Button
                            }
                            .clickable { expandedAll = true }
                            .padding(14.dp),
                        verticalAlignment = Alignment.CenterVertically,
                        horizontalArrangement = Arrangement.SpaceBetween,
                    ) {
                        Column {
                            Text(
                                text = "All themes",
                                style = MaterialTheme.typography.titleSmall.copy(fontWeight = FontWeight.Bold),
                                color = MaterialTheme.colorScheme.onSurface,
                            )
                            Text(
                                text = "$remainingCount more colors available",
                                style = MaterialTheme.typography.bodySmall,
                                color = MaterialTheme.colorScheme.onSurfaceVariant,
                            )
                        }
                        NomiIcon(NomiIcons.ChevronDown, contentDescription = "Show all themes", tint = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                }
            }
        }
    }
}

@Composable
fun ThemeTile(
    option: ThemeOption,
    selected: Boolean,
    onSelect: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val borderColor = if (selected) MaterialTheme.colorScheme.primary else Color.Transparent
    Column(
        modifier = modifier
            .clip(RoundedCornerShape(24.dp))
            .border(2.dp, borderColor, RoundedCornerShape(24.dp))
            .background(MaterialTheme.colorScheme.surfaceContainerLowest)
            .semantics(mergeDescendants = true) {
                this.selected = selected
                this.role = Role.RadioButton
            }
            .clickable(onClick = onSelect)
            .padding(10.dp),
        verticalArrangement = Arrangement.spacedBy(4.dp),
    ) {
        // Preview Box: 92.dp height, split into Light and Dark halves
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .height(92.dp)
                .clip(RoundedCornerShape(16.dp)),
        ) {
            Row(Modifier.matchParentSize()) {
                // Light half
                SwatchHalf(option.light, Modifier.weight(1f))
                // Dark half
                SwatchHalf(option.dark, Modifier.weight(1f))
            }
            if (selected) {
                Box(
                    modifier = Modifier
                        .align(Alignment.TopEnd)
                        .padding(6.dp)
                        .size(24.dp)
                        .background(MaterialTheme.colorScheme.primary, CircleShape),
                    contentAlignment = Alignment.Center,
                ) {
                    NomiIcon(NomiIcons.Check, contentDescription = "Selected", tint = MaterialTheme.colorScheme.onPrimary, size = 14.dp)
                }
            }
        }

        Text(
            text = option.label,
            style = MaterialTheme.typography.titleMedium.copy(fontWeight = FontWeight.Bold),
            color = MaterialTheme.colorScheme.onSurface,
            modifier = Modifier.padding(horizontal = 4.dp, vertical = 2.dp),
        )
        Text(
            text = option.description,
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            modifier = Modifier.padding(horizontal = 4.dp),
            maxLines = 2,
        )
    }
}

@Composable
private fun SwatchHalf(swatch: ThemeSwatchColors, modifier: Modifier = Modifier) {
    Box(
        modifier = modifier
            .background(swatch.surface)
            .padding(8.dp),
    ) {
        // Mini card
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .clip(RoundedCornerShape(10.dp))
                .background(swatch.surfaceContainerLowest)
                .padding(6.dp),
            verticalArrangement = Arrangement.spacedBy(4.dp),
        ) {
            // Text line 1
            Box(
                Modifier
                    .width(42.dp)
                    .height(4.dp)
                    .clip(RoundedCornerShape(2.dp))
                    .background(swatch.onSurface.copy(alpha = 0.75f)),
            )
            // Text line 2
            Box(
                Modifier
                    .width(26.dp)
                    .height(4.dp)
                    .clip(RoundedCornerShape(2.dp))
                    .background(swatch.onSurface.copy(alpha = 0.4f)),
            )
            // Row with pill + dot
            Row(
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(4.dp),
                modifier = Modifier.padding(top = 2.dp),
            ) {
                Box(
                    Modifier
                        .width(22.dp)
                        .height(8.dp)
                        .clip(RoundedCornerShape(4.dp))
                        .background(swatch.primary),
                )
                Box(
                    Modifier
                        .size(8.dp)
                        .background(swatch.tertiary, CircleShape),
                )
            }
        }

        // Chip at bottom right
        Box(
            modifier = Modifier
                .align(Alignment.BottomEnd)
                .width(16.dp)
                .height(6.dp)
                .clip(RoundedCornerShape(3.dp))
                .background(swatch.primaryContainer),
        )
    }
}
