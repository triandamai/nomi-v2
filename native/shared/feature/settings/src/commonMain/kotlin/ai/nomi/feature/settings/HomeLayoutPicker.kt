package ai.nomi.feature.settings

import ai.nomi.designsystem.theme.NomiTheme
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.role
import androidx.compose.ui.semantics.selected
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp

data class HomeLayoutSpec(
    val id: String,
    val name: String,
    val description: String,
    val isGlass: Boolean,
)

val HOME_LAYOUT_OPTIONS = listOf(
    HomeLayoutSpec(
        id = "stage",
        name = "Stage",
        description = "Centered stage with hero composer and crew",
        isGlass = true,
    ),
    HomeLayoutSpec(
        id = "classic",
        name = "Classic",
        description = "Side-by-side composer and crew roster",
        isGlass = false,
    ),
    HomeLayoutSpec(
        id = "today",
        name = "Today",
        description = "Daily agenda with compact tools and activity",
        isGlass = false,
    ),
)

@Composable
fun HomeLayoutPicker(
    current: String,
    onSelectLayout: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(12.dp)) {
        HOME_LAYOUT_OPTIONS.forEach { spec ->
            val selected = current == spec.id
            val borderColor = if (selected) MaterialTheme.colorScheme.primary else Color.Transparent

            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .clip(RoundedCornerShape(24.dp))
                    .border(2.dp, borderColor, RoundedCornerShape(24.dp))
                    .background(MaterialTheme.colorScheme.surfaceContainerLowest)
                    .semantics(mergeDescendants = true) {
                        this.selected = selected
                        this.role = Role.RadioButton
                    }
                    .clickable { onSelectLayout(spec.id) }
                    .padding(12.dp),
                verticalArrangement = Arrangement.spacedBy(4.dp),
            ) {
                // Preview thumbnail (112.dp)
                LayoutThumbnail(spec.id, spec.isGlass)

                Text(
                    text = spec.name,
                    style = MaterialTheme.typography.titleMedium.copy(fontWeight = FontWeight(650)),
                    color = MaterialTheme.colorScheme.onSurface,
                    modifier = Modifier.padding(top = 4.dp, start = 4.dp),
                )
                Text(
                    text = spec.description,
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier = Modifier.padding(start = 4.dp),
                )
            }
        }
    }
}

@Composable
private fun LayoutThumbnail(layoutId: String, isGlass: Boolean) {
    val bgModifier = if (isGlass) {
        Modifier.background(
            Brush.radialGradient(
                colors = listOf(
                    MaterialTheme.colorScheme.primaryContainer.copy(alpha = 0.5f),
                    NomiTheme.extendedColors.ground,
                ),
            ),
        )
    } else {
        Modifier.background(NomiTheme.extendedColors.ground)
    }

    Box(
        modifier = Modifier
            .fillMaxWidth()
            .height(112.dp)
            .clip(RoundedCornerShape(16.dp))
            .then(bgModifier)
            .padding(10.dp),
    ) {
        when (layoutId) {
            "stage" -> StageThumbnail()
            "classic" -> ClassicThumbnail()
            "today" -> TodayThumbnail()
        }
    }
}

@Composable
private fun StageThumbnail() {
    Column(
        modifier = Modifier.fillMaxWidth().padding(horizontal = 24.dp),
        verticalArrangement = Arrangement.spacedBy(5.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        // Greeting
        Box(
            Modifier
                .width(54.dp)
                .height(9.dp)
                .clip(RoundedCornerShape(4.dp))
                .background(MaterialTheme.colorScheme.onSurface.copy(alpha = 0.22f)),
        )
        // Composer
        Box(
            Modifier
                .fillMaxWidth()
                .height(14.dp)
                .clip(RoundedCornerShape(4.dp))
                .background(MaterialTheme.colorScheme.surfaceContainerLowest),
        )
        // Crew
        Box(
            Modifier
                .fillMaxWidth()
                .height(12.dp)
                .clip(RoundedCornerShape(4.dp))
                .background(MaterialTheme.colorScheme.surfaceContainerLowest.copy(alpha = 0.72f)),
        )
        // Status
        Box(
            Modifier
                .fillMaxWidth()
                .height(12.dp)
                .clip(RoundedCornerShape(4.dp))
                .background(MaterialTheme.colorScheme.surfaceContainerLowest),
        )
        // Recent
        Box(
            Modifier
                .fillMaxWidth()
                .height(10.dp)
                .clip(RoundedCornerShape(4.dp))
                .background(MaterialTheme.colorScheme.surfaceContainerLowest),
        )
    }
}

@Composable
private fun ClassicThumbnail() {
    Column(modifier = Modifier.fillMaxWidth(), verticalArrangement = Arrangement.spacedBy(5.dp)) {
        // Top 2 columns
        Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(5.dp)) {
            Column(modifier = Modifier.weight(1.5f), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                Box(
                    Modifier
                        .width(44.dp)
                        .height(9.dp)
                        .clip(RoundedCornerShape(4.dp))
                        .background(MaterialTheme.colorScheme.onSurface.copy(alpha = 0.22f)),
                )
                Box(
                    Modifier
                        .fillMaxWidth()
                        .height(14.dp)
                        .clip(RoundedCornerShape(4.dp))
                        .background(MaterialTheme.colorScheme.surfaceContainerLowest),
                )
            }
            Column(modifier = Modifier.weight(1f)) {
                Box(
                    Modifier
                        .fillMaxWidth()
                        .height(27.dp)
                        .clip(RoundedCornerShape(4.dp))
                        .background(NomiTheme.extendedColors.stage),
                )
            }
        }
        // Status
        Box(
            Modifier
                .fillMaxWidth()
                .height(12.dp)
                .clip(RoundedCornerShape(4.dp))
                .background(MaterialTheme.colorScheme.surfaceContainerLowest),
        )
        // Recent cards
        Box(
            Modifier
                .fillMaxWidth()
                .height(10.dp)
                .clip(RoundedCornerShape(4.dp))
                .background(MaterialTheme.colorScheme.surfaceContainerLowest),
        )
    }
}

@Composable
private fun TodayThumbnail() {
    Column(modifier = Modifier.fillMaxWidth(), verticalArrangement = Arrangement.spacedBy(5.dp)) {
        // Greeting
        Box(
            Modifier
                .width(44.dp)
                .height(9.dp)
                .clip(RoundedCornerShape(4.dp))
                .background(MaterialTheme.colorScheme.onSurface.copy(alpha = 0.22f)),
        )
        // Status
        Box(
            Modifier
                .fillMaxWidth()
                .height(12.dp)
                .clip(RoundedCornerShape(4.dp))
                .background(MaterialTheme.colorScheme.surfaceContainerLowest),
        )
        // 2 columns
        Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(5.dp)) {
            Column(modifier = Modifier.weight(1.4f), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                Box(
                    Modifier
                        .fillMaxWidth()
                        .height(14.dp)
                        .clip(RoundedCornerShape(4.dp))
                        .background(MaterialTheme.colorScheme.surfaceContainerLowest),
                )
                Box(
                    Modifier
                        .fillMaxWidth()
                        .height(10.dp)
                        .clip(RoundedCornerShape(4.dp))
                        .background(MaterialTheme.colorScheme.surfaceContainerLowest),
                )
            }
            Column(modifier = Modifier.weight(1f)) {
                Box(
                    Modifier
                        .fillMaxWidth()
                        .height(28.dp)
                        .clip(RoundedCornerShape(4.dp))
                        .background(NomiTheme.extendedColors.stage),
                )
            }
        }
    }
}
