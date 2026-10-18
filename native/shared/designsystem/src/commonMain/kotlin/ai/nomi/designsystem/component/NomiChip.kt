package ai.nomi.designsystem.component

import androidx.compose.material3.AssistChip
import androidx.compose.material3.AssistChipDefaults
import androidx.compose.material3.SuggestionChipDefaults
import androidx.compose.material3.FilterChip
import androidx.compose.material3.FilterChipDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SuggestionChip
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

private val CheckSize = 16.dp
private val BorderWidth = 1.5.dp

/** An assist chip: a contextual action. [leading] is an optional icon slot. */
@Composable
fun NomiAssistChip(label: String, onClick: () -> Unit, modifier: Modifier = Modifier, enabled: Boolean = true, leading: (@Composable () -> Unit)? = null) {
    AssistChip(
        onClick = onClick,
        label = { Text(label) },
        modifier = modifier,
        enabled = enabled,
        leadingIcon = leading,
        shape = MaterialTheme.shapes.medium,
        border = AssistChipDefaults.assistChipBorder(enabled, MaterialTheme.colorScheme.outlineVariant, borderWidth = BorderWidth),
    )
}

/** A suggestion chip: a prompt starter. */
@Composable
fun NomiSuggestionChip(label: String, onClick: () -> Unit, modifier: Modifier = Modifier, enabled: Boolean = true) {
    SuggestionChip(
        onClick = onClick,
        label = { Text(label) },
        modifier = modifier,
        enabled = enabled,
        shape = MaterialTheme.shapes.medium,
        border = SuggestionChipDefaults.suggestionChipBorder(enabled, MaterialTheme.colorScheme.outlineVariant, borderWidth = BorderWidth),
    )
}

/** A filter chip: selected shows a check on the primary container. */
@Composable
fun NomiFilterChip(label: String, selected: Boolean, onClick: () -> Unit, modifier: Modifier = Modifier, enabled: Boolean = true) {
    val colors = MaterialTheme.colorScheme
    FilterChip(
        selected = selected,
        onClick = onClick,
        label = { Text(label) },
        modifier = modifier,
        enabled = enabled,
        leadingIcon = if (selected) ({ CheckGlyph(colors.onPrimaryContainer, CheckSize) }) else null,
        shape = MaterialTheme.shapes.medium,
        colors = FilterChipDefaults.filterChipColors(
            selectedContainerColor = colors.primaryContainer,
            selectedLabelColor = colors.onPrimaryContainer,
        ),
        border = FilterChipDefaults.filterChipBorder(
            enabled = enabled,
            selected = selected,
            borderColor = colors.outlineVariant,
            selectedBorderColor = colors.primaryContainer,
            borderWidth = BorderWidth,
            selectedBorderWidth = BorderWidth,
        ),
    )
}
