package ai.nomi.feature.settings

import ai.nomi.data.THEME_DARK
import ai.nomi.data.THEME_LIGHT
import ai.nomi.data.THEME_SYSTEM
import ai.nomi.designsystem.component.ButtonGroupOption
import ai.nomi.designsystem.component.NomiButtonGroup
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

/** The appearance setting: light, dark, or following the system. The pen has one accent, so there is no colour theme. */
@Composable
fun AppearancePicker(
    mode: String,
    onSelectMode: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(ModeGap)) {
        Text(
            text = "MODE",
            style = MaterialTheme.typography.labelSmall.copy(fontFamily = FontFamily.Monospace, letterSpacing = LabelTracking),
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
}

private val ModeGap = 6.dp
private val LabelTracking = 0.08.sp
