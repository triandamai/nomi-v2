package ai.nomi.designsystem.component

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material3.ButtonGroupDefaults
import androidx.compose.material3.ExperimentalMaterial3ExpressiveApi
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.ToggleButton
import androidx.compose.material3.ToggleButtonDefaults
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.role
import androidx.compose.ui.semantics.semantics

class NomiGroupOption<T>(val value: T, val label: String, val enabled: Boolean = true)

/**
 * A connected, single-select button group (a segmented control): segments sit a hair apart with tight inner corners
 * and the selected one springs to round. Built on M3 Expressive's connected ToggleButton shapes.
 */
@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun <T> NomiButtonGroup(
    options: List<NomiGroupOption<T>>,
    selected: T,
    onSelect: (T) -> Unit,
    modifier: Modifier = Modifier,
) {
    Row(
        modifier.fillMaxWidth().semantics { role = Role.RadioButton },
        horizontalArrangement = Arrangement.spacedBy(ButtonGroupDefaults.ConnectedSpaceBetween),
    ) {
        options.forEachIndexed { index, option ->
            val shapes = when {
                options.size == 1 -> ToggleButtonDefaults.shapes()
                index == 0 -> ButtonGroupDefaults.connectedLeadingButtonShapes()
                index == options.lastIndex -> ButtonGroupDefaults.connectedTrailingButtonShapes()
                else -> ButtonGroupDefaults.connectedMiddleButtonShapes()
            }
            ToggleButton(
                checked = option.value == selected,
                onCheckedChange = { onSelect(option.value) },
                modifier = Modifier.weight(1f),
                enabled = option.enabled,
                shapes = shapes,
                colors = ToggleButtonDefaults.toggleButtonColors(containerColor = MaterialTheme.colorScheme.surfaceContainerHigh),
            ) { Text(option.label) }
        }
    }
}
