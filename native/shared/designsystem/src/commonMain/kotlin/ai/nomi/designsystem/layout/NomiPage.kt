package ai.nomi.designsystem.layout

import ai.nomi.designsystem.theme.NomiTheme
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.RowScope
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawingPadding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.dp

/** Spacing steps shared by the app's screens (multiples of the 4dp grid). */
object NomiSpacing {
    val ExtraSmall = 4.dp
    val Small = 8.dp
    val Medium = 12.dp
    val Large = 16.dp
    val ExtraLarge = 24.dp
}

/** Fills the weight left over in a row or column. */
private const val FILL = 1f

/**
 * A full screen on the theme's page ground, inside the safe drawing area: a heading row ([title] plus optional
 * [actions]) above [content].
 */
@Composable
fun NomiPage(
    title: String,
    modifier: Modifier = Modifier,
    actions: @Composable RowScope.() -> Unit = {},
    content: @Composable ColumnScope.() -> Unit,
) {
    Column(
        modifier
            .fillMaxSize()
            .background(NomiTheme.extendedColors.ground)
            .safeDrawingPadding()
            .padding(horizontal = NomiSpacing.Large),
        verticalArrangement = Arrangement.spacedBy(NomiSpacing.Medium),
    ) {
        Row(
            Modifier.fillMaxWidth().padding(top = NomiSpacing.Large),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(NomiSpacing.Small),
        ) {
            Text(
                title,
                style = MaterialTheme.typography.headlineSmall,
                color = MaterialTheme.colorScheme.onSurface,
                modifier = Modifier.weight(FILL).semantics { heading() },
            )
            actions()
        }
        content()
    }
}
