package ai.nomi.designsystem.shape

import ai.nomi.designsystem.generated.PolarSpec
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Brush

/** [from] morphing into [to] point by point; [progress] 0 draws [from], 1 draws [to]. */
@Composable
fun MorphingShape(from: PolarSpec, to: PolarSpec, progress: Float, brush: Brush, modifier: Modifier = Modifier) {
    val a = remember(from) { polarPoints(from) }
    val b = remember(to) { polarPoints(to) }
    Box(modifier.background(brush, MorphShape(a, b, progress)))
}
