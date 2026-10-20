package ai.nomi.app.screens.connections

import ai.nomi.designsystem.component.NomiCard
import ai.nomi.designsystem.component.NomiFilterChip
import ai.nomi.designsystem.component.NomiListPanel
import ai.nomi.designsystem.component.NomiLoadingIndicator
import ai.nomi.designsystem.component.NomiRow
import ai.nomi.designsystem.component.NomiSectionHeader
import ai.nomi.designsystem.component.NomiTextField
import ai.nomi.designsystem.icon.NomiIcon
import ai.nomi.designsystem.icon.NomiIcons
import ai.nomi.designsystem.layout.NomiPage
import ai.nomi.designsystem.shape.AgentShape
import ai.nomi.designsystem.shape.agentIdentity
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.compose.collectAsStateWithLifecycle

private const val FONT_WEIGHT_BOLD = 700
private const val FONT_WEIGHT_SEMI = 600

private val SectionSpacing = 20.dp
private val InnerCardPadding = 16.dp
private val SmallSpacing = 8.dp
private val AgentAvatarSize = 36.dp
private val BadgePaddingH = 8.dp
private val BadgePaddingV = 4.dp
private val BadgeCorner = 8.dp
private val BottomListPadding = 32.dp

@Composable
fun ConnectionsScreen(
    viewModel: ConnectionsViewModel,
    modifier: Modifier = Modifier,
) {
    val state by viewModel.uiState.collectAsStateWithLifecycle()

    NomiPage(
        title = "Connections",
        modifier = modifier,
    ) {
        if (state.loading && state.overview == null) {
            Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                NomiLoadingIndicator(contentDescription = "Loading connections")
            }
        } else {
            ConnectionsContent(
                state = state,
                onQueryChange = viewModel::setQuery,
            )
        }
    }
}

@Composable
private fun ConnectionsContent(
    state: ConnectionsUiState,
    onQueryChange: (String) -> Unit,
) {
    val categories = listOf("Productivity", "Health")

    Column(
        Modifier
            .fillMaxWidth()
            .verticalScroll(rememberScrollState())
            .padding(bottom = BottomListPadding),
        verticalArrangement = Arrangement.spacedBy(SectionSpacing),
    ) {
        Text(
            "Every app the crew can work in, grouped by kind and searchable.",
            style = MaterialTheme.typography.bodyLarge,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )

        NomiTextField(
            value = state.query,
            onValueChange = onQueryChange,
            label = "Search",
            placeholder = "Search connections...",
            modifier = Modifier.fillMaxWidth(),
        )

        categories.forEach { category ->
            val itemsInCategory = state.filteredConnectors.filter { it.category == category }
            if (itemsInCategory.isNotEmpty()) {
                NomiSectionHeader(category)
                NomiListPanel {
                    itemsInCategory.forEach { connector ->
                        ConnectorRow(connector)
                    }
                }
            }
        }

        if (state.filteredConnectors.isEmpty()) {
            NomiListPanel {
                Box(Modifier.fillMaxWidth().padding(InnerCardPadding), contentAlignment = Alignment.Center) {
                    Text(
                        "No connections match \"${state.query}\"",
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }
        }
    }
}

@Composable
private fun ConnectorRow(connector: ConnectorItem) {
    val colors = MaterialTheme.colorScheme
    val badgeBg = when {
        connector.connected -> colors.primaryContainer
        connector.soon -> colors.surfaceContainerHighest
        else -> colors.surfaceContainerHigh
    }
    val badgeInk = when {
        connector.connected -> colors.onPrimaryContainer
        connector.soon -> colors.onSurfaceVariant
        else -> colors.onSurfaceVariant
    }

    Row(
        Modifier
            .fillMaxWidth()
            .padding(horizontal = InnerCardPadding, vertical = SmallSpacing),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(SmallSpacing),
    ) {
        AgentShape(
            identity = agentIdentity(connector.agent),
            size = AgentAvatarSize,
        )
        Column(Modifier.weight(1f)) {
            Text(
                connector.name,
                style = MaterialTheme.typography.bodyLarge.copy(fontWeight = FontWeight(FONT_WEIGHT_SEMI)),
                color = colors.onSurface,
            )
            Text(
                "${connector.description} · Used by ${connector.agent}",
                style = MaterialTheme.typography.bodyMedium,
                color = colors.onSurfaceVariant,
            )
        }
        Box(
            Modifier
                .clip(RoundedCornerShape(BadgeCorner))
                .background(badgeBg)
                .padding(horizontal = BadgePaddingH, vertical = BadgePaddingV),
        ) {
            Text(
                text = connector.statusText,
                style = MaterialTheme.typography.labelSmall.copy(fontWeight = FontWeight(FONT_WEIGHT_SEMI)),
                color = badgeInk,
            )
        }
    }
}
