package ai.nomi.designsystem.component

import ai.nomi.designsystem.layout.NomiSpacing
import ai.nomi.designsystem.theme.NomiTheme
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp

private val RowMinHeight = 56.dp
private const val TITLE_LINES = 1
private const val SUPPORTING_LINES = 2
private const val FILL = 1f

/**
 * One row of a list: a title, up to two supporting lines and a quiet [meta] line on the trailing side. [emphasized]
 * marks something new (unread) by weight, not colour decoration. Clickable rows get a focus ring and a 56dp target.
 */
@Composable
fun NomiListRow(
    title: String,
    modifier: Modifier = Modifier,
    supporting: String? = null,
    meta: String? = null,
    emphasized: Boolean = false,
    selected: Boolean = false,
    onClick: (() -> Unit)? = null,
) {
    val shape = MaterialTheme.shapes.large
    val colors = MaterialTheme.colorScheme
    val clickable = if (onClick != null) {
        Modifier.nomiFocusRing(shape).clip(shape).clickable(role = Role.Button, onClick = onClick)
    } else {
        Modifier
    }
    Row(
        modifier
            .fillMaxWidth()
            .heightIn(min = RowMinHeight)
            .then(clickable)
            .then(if (selected) Modifier.clip(shape).background(colors.secondaryContainer) else Modifier)
            .padding(horizontal = NomiSpacing.Medium, vertical = NomiSpacing.Small),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(NomiSpacing.Medium),
    ) {
        Column(Modifier.weight(FILL), verticalArrangement = Arrangement.spacedBy(NomiSpacing.ExtraSmall)) {
            Text(
                title,
                style = MaterialTheme.typography.titleMedium,
                fontWeight = if (emphasized) FontWeight.SemiBold else null,
                color = colors.onSurface,
                maxLines = TITLE_LINES,
                overflow = TextOverflow.Ellipsis,
            )
            if (!supporting.isNullOrEmpty()) {
                Text(
                    supporting,
                    style = MaterialTheme.typography.bodyMedium,
                    color = if (emphasized) colors.onSurface else colors.onSurfaceVariant,
                    maxLines = SUPPORTING_LINES,
                    overflow = TextOverflow.Ellipsis,
                )
            }
        }
        if (meta != null) Text(meta, style = NomiTheme.metaStyle, color = colors.onSurfaceVariant)
    }
}
