package ai.nomi.designsystem.component

import ai.nomi.designsystem.generated.NomiShapeSpecs
import ai.nomi.designsystem.shape.toRoundedPolygon
import androidx.compose.foundation.layout.size
import androidx.compose.material3.ExperimentalMaterial3ExpressiveApi
import androidx.compose.material3.LoadingIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

/** The shapes the loader morphs through, in the web LoadingIndicator's order. */
private val LoaderShapes = listOf("cookie9", "sunny8", "cookie6", "clover4", "flower5")

val DefaultLoadingIndicatorSize: Dp = 48.dp

/** M3 Expressive's LoadingIndicator morphing through Nomi's own agent shapes. */
@OptIn(ExperimentalMaterial3ExpressiveApi::class)
@Composable
fun NomiLoadingIndicator(
    modifier: Modifier = Modifier,
    size: Dp = DefaultLoadingIndicatorSize,
    color: Color = MaterialTheme.colorScheme.primary,
    contentDescription: String? = null,
) {
    val polygons = remember { LoaderShapes.map { NomiShapeSpecs.all.getValue(it).toRoundedPolygon() } }
    LoadingIndicator(
        modifier = modifier.size(size).then(
            if (contentDescription != null) Modifier.semantics { this.contentDescription = contentDescription } else Modifier,
        ),
        color = color,
        polygons = polygons,
    )
}
