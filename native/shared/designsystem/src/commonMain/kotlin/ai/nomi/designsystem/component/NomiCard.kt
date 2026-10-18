package ai.nomi.designsystem.component

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.ElevatedCard
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedCard
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

enum class NomiCardVariant { Tonal, Outlined, Filled, Elevated }

/**
 * A content surface on the tinted page ground. Tonal and outlined sit on the lowest container (never a
 * hard-coded white); pass [onClick] to make the whole card an interactive target.
 */
@Composable
fun NomiCard(
    modifier: Modifier = Modifier,
    variant: NomiCardVariant = NomiCardVariant.Tonal,
    onClick: (() -> Unit)? = null,
    content: @Composable ColumnScope.() -> Unit,
) {
    val colors = MaterialTheme.colorScheme
    val shape = MaterialTheme.shapes.extraLarge
    val container = when (variant) {
        NomiCardVariant.Tonal, NomiCardVariant.Outlined -> colors.surfaceContainerLowest
        NomiCardVariant.Filled -> colors.surfaceContainerHighest
        NomiCardVariant.Elevated -> colors.surfaceContainerLow
    }
    val cardColors = CardDefaults.cardColors(containerColor = container, contentColor = colors.onSurface)
    when (variant) {
        NomiCardVariant.Elevated -> if (onClick != null) {
            ElevatedCard(onClick, modifier, shape = shape, colors = cardColors, content = content)
        } else {
            ElevatedCard(modifier, shape = shape, colors = cardColors, content = content)
        }
        NomiCardVariant.Outlined -> {
            val border = BorderStroke(OutlineWidth, colors.outlineVariant)
            if (onClick != null) {
                OutlinedCard(onClick, modifier, shape = shape, colors = cardColors, border = border, content = content)
            } else {
                OutlinedCard(modifier, shape = shape, colors = cardColors, border = border, content = content)
            }
        }
        else -> if (onClick != null) {
            Card(onClick, modifier, shape = shape, colors = cardColors, content = content)
        } else {
            Card(modifier, shape = shape, colors = cardColors, content = content)
        }
    }
}

private val OutlineWidth = 1.dp
