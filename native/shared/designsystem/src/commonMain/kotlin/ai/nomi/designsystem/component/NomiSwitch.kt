package ai.nomi.designsystem.component

import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Switch
import androidx.compose.material3.SwitchDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

private val ThumbCheckSize = 14.dp

/** M3 Switch with a check glyph inside the thumb when checked, matching `Switch.svelte`. */
@Composable
fun NomiSwitch(
    checked: Boolean,
    onCheckedChange: (Boolean) -> Unit,
    modifier: Modifier = Modifier,
    enabled: Boolean = true,
) {
    val colors = MaterialTheme.colorScheme
    val source = remember { MutableInteractionSource() }
    Switch(
        checked = checked,
        onCheckedChange = onCheckedChange,
        modifier = modifier.nomiFocusRing(CircleShape),
        enabled = enabled,
        interactionSource = source,
        thumbContent = if (checked) {
            { CheckGlyph(colors.onPrimaryContainer, ThumbCheckSize) }
        } else {
            null
        },
        colors = SwitchDefaults.colors(
            checkedThumbColor = colors.primaryContainer,
            checkedTrackColor = colors.primary,
            checkedBorderColor = colors.primary,
            uncheckedThumbColor = colors.outline,
            uncheckedTrackColor = colors.surfaceContainerHighest,
            uncheckedBorderColor = colors.outline,
        ),
    )
}
