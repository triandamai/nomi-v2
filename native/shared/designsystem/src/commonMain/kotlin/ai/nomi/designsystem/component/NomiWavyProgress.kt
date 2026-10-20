package ai.nomi.designsystem.component

import androidx.compose.material3.ExperimentalMaterial3ExpressiveApi
import androidx.compose.material3.LinearWavyProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics

/**
 * M3 Expressive wavy linear progress. M3's API takes a solid Color, so the web's gradient wave is not reproduced here.
 * [progress] 0..1 draws the wave, the rest is a flat track ending in a stop dot. */
@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun NomiWavyProgress(
    progress: () -> Float,
    contentDescription: String,
    modifier: Modifier = Modifier,
    color: androidx.compose.ui.graphics.Color = MaterialTheme.colorScheme.primary,
) {
    val colors = MaterialTheme.colorScheme
    LinearWavyProgressIndicator(
        progress = progress,
        modifier = modifier.semantics { this.contentDescription = contentDescription },
        color = color,
        trackColor = colors.surfaceContainerHighest,
    )
}
