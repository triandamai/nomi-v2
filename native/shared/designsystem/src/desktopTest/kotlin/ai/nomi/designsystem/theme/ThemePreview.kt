package ai.nomi.designsystem.theme

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.unit.dp

private val PreviewPadding = 24.dp
private val PreviewGap = 12.dp
private val GradientButtonCorner = 20.dp
private val GradientButtonHPadding = 24.dp
private val GradientButtonVPadding = 12.dp

/** Every type role, the button families, a gradient button and cards, on the Nomi ground. */
@Composable
fun ThemePreview() {
    val colors = MaterialTheme.colorScheme
    Column(
        Modifier.fillMaxSize().background(NomiTheme.extendedColors.ground).padding(PreviewPadding),
        verticalArrangement = Arrangement.spacedBy(PreviewGap),
    ) {
        Text("Display Nomi", style = MaterialTheme.typography.displayMedium, color = colors.onSurface)
        Text("Headline: plan the week", style = MaterialTheme.typography.headlineMedium, color = colors.onSurface)
        Text("Title: Groceries and rent", style = MaterialTheme.typography.titleMedium, color = colors.onSurface)
        Text("Body: Instrument Sans reads calmly at paragraph size.", style = MaterialTheme.typography.bodyLarge, color = colors.onSurface)
        Text("META 2026-10-18 12:04 #a1b2c3", style = NomiTheme.metaStyle, color = colors.onSurfaceVariant)
        Row(horizontalArrangement = Arrangement.spacedBy(PreviewGap), verticalAlignment = Alignment.CenterVertically) {
            Button(onClick = {}) { Text("Primary") }
            FilledTonalButton(onClick = {}) { Text("Secondary") }
            OutlinedButton(onClick = {}) { Text("Tertiary") }
            Box(
                Modifier.clip(RoundedCornerShape(GradientButtonCorner))
                    .background(NomiTheme.extendedColors.actionBrush)
                    .padding(horizontal = GradientButtonHPadding, vertical = GradientButtonVPadding),
            ) {
                Text("Gradient", style = MaterialTheme.typography.labelLarge, color = NomiTheme.extendedColors.onAction)
            }
        }
        Row(horizontalArrangement = Arrangement.spacedBy(PreviewGap), modifier = Modifier.fillMaxWidth()) {
            Card(colors = CardDefaults.cardColors(containerColor = colors.surfaceContainerHigh)) {
                Text("Card on ground", Modifier.padding(PreviewPadding), color = colors.onSurface)
            }
            Card(colors = CardDefaults.cardColors(containerColor = NomiTheme.extendedColors.stage)) {
                Text("Stage card", Modifier.padding(PreviewPadding), color = NomiTheme.extendedColors.onStage)
            }
        }
    }
}
