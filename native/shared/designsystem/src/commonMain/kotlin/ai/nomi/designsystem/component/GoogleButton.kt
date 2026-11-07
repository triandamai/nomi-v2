package ai.nomi.designsystem.component

import ai.nomi.designsystem.ds.DsMotion
import ai.nomi.designsystem.ds.DsSpinner
import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.drawscope.scale
import androidx.compose.ui.graphics.vector.PathParser
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.stateDescription
import androidx.compose.ui.unit.dp

// `GoogleButton.svelte`: an M3 outlined pill with Google's multicolour G (its required mark for sign-in buttons).
// While the browser is on its way to Google the G turns into a spinner and the button holds still.

private val ButtonHeight = 56.dp
private val ButtonPadding = 20.dp
private val ContentGap = 12.dp
private val MarkSize = 20.dp
private val OutlineWidth = 1.dp
private const val MARK_GRID = 48f
private const val BUSY_ALPHA = 0.7f

/** The four strokes of the G on its 48-unit grid, with Google's colours. */
private val GoogleMark = listOf(
    0xFFEA4335 to "M24 9.5c3.54 0 6.71 1.22 9.21 3.6l6.85-6.85C35.9 2.38 30.47 0 24 0 14.62 0 6.51 5.38 2.56 13.22l7.98 6.19C12.43 13.72 17.74 9.5 24 9.5z",
    0xFF4285F4 to "M46.98 24.55c0-1.57-.15-3.09-.38-4.55H24v9.02h12.94c-.58 2.96-2.26 5.48-4.78 7.18l7.73 6c4.51-4.18 7.09-10.36 7.09-17.65z",
    0xFFFBBC05 to "M10.53 28.59c-.48-1.45-.76-2.99-.76-4.59s.27-3.14.76-4.59l-7.98-6.19C.92 16.46 0 20.12 0 24c0 3.88.92 7.54 2.56 10.78l7.97-6.19z",
    0xFF34A853 to "M24 48c6.48 0 11.93-2.13 15.89-5.81l-7.73-6c-2.15 1.45-4.92 2.3-8.16 2.3-6.26 0-11.57-4.22-13.47-9.91l-7.98 6.19C6.51 42.62 14.62 48 24 48z",
)

@Composable
private fun GoogleG(modifier: Modifier = Modifier) {
    val paths = remember { GoogleMark.map { (color, d) -> Color(color) to PathParser().parsePathString(d).toPath() } }
    Canvas(modifier.size(MarkSize)) {
        scale(size.width / MARK_GRID, size.height / MARK_GRID, pivot = androidx.compose.ui.geometry.Offset.Zero) {
            paths.forEach { (color, path) -> drawPath(path, color) }
        }
    }
}

/** "Continue with Google". [busy] while the browser goes to Google (or its answer is being checked). */
@Composable
fun GoogleButton(onClick: () -> Unit, modifier: Modifier = Modifier, label: String = "Continue with Google", busy: Boolean = false) {
    val colors = MaterialTheme.colorScheme
    val fadeIn = fadeIn(DsMotion.effects())
    val fadeOut = fadeOut(DsMotion.effects())
    Row(
        modifier
            .fillMaxWidth()
            .height(ButtonHeight)
            .clip(CircleShape)
            .border(OutlineWidth, colors.outline, CircleShape)
            .clickable(enabled = !busy, role = Role.Button, onClick = onClick)
            .semantics { if (busy) stateDescription = "Opening Google" }
            .alpha(if (busy) BUSY_ALPHA else 1f)
            .padding(horizontal = ButtonPadding),
        horizontalArrangement = Arrangement.spacedBy(ContentGap, Alignment.CenterHorizontally),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        AnimatedContent(busy, transitionSpec = { fadeIn togetherWith fadeOut }, label = "google-mark") { leaving ->
            Box(Modifier.size(MarkSize), contentAlignment = Alignment.Center) {
                if (leaving) DsSpinner(MarkSize, colors.onSurfaceVariant) else GoogleG()
            }
        }
        Text(label, style = MaterialTheme.typography.labelLarge, color = colors.onSurface)
    }
}
