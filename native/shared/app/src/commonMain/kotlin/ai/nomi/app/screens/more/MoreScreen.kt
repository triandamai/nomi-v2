package ai.nomi.app.screens.more

import ai.nomi.data.AppearanceRepository
import ai.nomi.designsystem.icon.NomiIcon
import ai.nomi.designsystem.icon.NomiIcons
import ai.nomi.designsystem.layout.NomiPage
import ai.nomi.designsystem.navigation.NomiFeatures
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle

@Composable
fun MoreScreen(
    appearanceRepository: AppearanceRepository,
    onOpenFeature: (id: String) -> Unit,
    onCustomizeDrawer: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val pins by appearanceRepository.drawerPins.collectAsStateWithLifecycle()
    MoreContent(
        pins = pins,
        onTogglePin = appearanceRepository::togglePin,
        onOpenFeature = onOpenFeature,
        onCustomizeDrawer = onCustomizeDrawer,
        modifier = modifier,
    )
}

@Composable
fun MoreContent(
    pins: List<String>,
    onTogglePin: (id: String) -> Unit,
    onOpenFeature: (id: String) -> Unit,
    onCustomizeDrawer: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val colors = MaterialTheme.colorScheme

    NomiPage(title = "All Features", modifier = modifier) {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .verticalScroll(rememberScrollState())
                .padding(bottom = 32.dp),
            verticalArrangement = Arrangement.spacedBy(20.dp),
        ) {
            Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
                Text(
                    text = "Everything in Nomi in one place, opened from the drawer's More. Each place can be pinned to the drawer (or unpinned) right here; the drawer's order is set in Preferences.",
                    style = MaterialTheme.typography.bodyMedium,
                    color = colors.onSurfaceVariant,
                )
                TextButton(
                    onClick = onCustomizeDrawer,
                    modifier = Modifier.padding(top = 4.dp),
                ) {
                    Text(
                        "Customize drawer order",
                        style = MaterialTheme.typography.labelLarge.copy(fontWeight = FontWeight.Bold),
                        color = colors.primary,
                    )
                }
            }

            NomiFeatures.Sections.forEach { section ->
                Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
                    Text(
                        text = section.title,
                        style = MaterialTheme.typography.titleMedium.copy(fontWeight = FontWeight.Bold),
                        color = colors.onSurface,
                    )

                    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                        section.ids.forEach { id ->
                            val feat = NomiFeatures.All[id] ?: return@forEach
                            val pinned = id in pins || feat.isPermanent
                            val cardShape = RoundedCornerShape(16.dp)

                            Row(
                                modifier = Modifier
                                    .fillMaxWidth()
                                    .clip(cardShape)
                                    .background(if (pinned) colors.surfaceContainer else colors.surfaceContainerLow)
                                    .then(
                                        if (pinned) Modifier.border(1.dp, colors.primary.copy(alpha = 0.25f), cardShape)
                                        else Modifier
                                    )
                                    .clickable { onOpenFeature(id) }
                                    .padding(horizontal = 14.dp, vertical = 12.dp),
                                verticalAlignment = Alignment.CenterVertically,
                                horizontalArrangement = Arrangement.spacedBy(14.dp),
                            ) {
                                Box(
                                    modifier = Modifier
                                        .size(40.dp)
                                        .clip(CircleShape)
                                        .background(if (pinned) colors.primaryContainer else colors.surfaceContainerHigh),
                                    contentAlignment = Alignment.Center,
                                ) {
                                    NomiIcon(
                                        feat.glyph,
                                        contentDescription = null,
                                        tint = if (pinned) colors.onPrimaryContainer else colors.onSurfaceVariant,
                                        size = 22.dp,
                                    )
                                }

                                Column(modifier = Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(2.dp)) {
                                    Text(
                                        text = feat.label,
                                        style = MaterialTheme.typography.titleSmall.copy(fontWeight = FontWeight.SemiBold),
                                        color = colors.onSurface,
                                    )
                                    Text(
                                        text = feat.description,
                                        style = MaterialTheme.typography.bodySmall,
                                        color = colors.onSurfaceVariant,
                                    )
                                }

                                if (feat.isPermanent) {
                                    Box(
                                        modifier = Modifier.size(36.dp),
                                        contentAlignment = Alignment.Center,
                                    ) {
                                        NomiIcon(
                                            NomiIcons.Lock,
                                            contentDescription = "Always in drawer",
                                            tint = colors.onSurfaceVariant.copy(alpha = 0.6f),
                                            size = 18.dp,
                                        )
                                    }
                                } else if (id in NomiFeatures.PinnableIds) {
                                    val isPinned = id in pins
                                    IconButton(
                                        onClick = { onTogglePin(id) },
                                        modifier = Modifier.size(36.dp),
                                    ) {
                                        NomiIcon(
                                            NomiIcons.Pin,
                                            contentDescription = if (isPinned) "Unpin ${feat.label} from drawer" else "Pin ${feat.label} to drawer",
                                            tint = if (isPinned) colors.primary else colors.onSurfaceVariant.copy(alpha = 0.5f),
                                            size = 20.dp,
                                        )
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
