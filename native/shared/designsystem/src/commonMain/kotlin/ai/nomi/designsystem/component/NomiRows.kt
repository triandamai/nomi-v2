package ai.nomi.designsystem.component

import ai.nomi.designsystem.theme.NomiTheme
import androidx.compose.animation.animateColorAsState
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.hoverable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsHoveredAsState
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.ripple
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.selected
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp

/** `.recent__list`: rows in one panel, a touch translucent so a backdrop shows through. */
private val PanelPadding = 6.dp
private const val PANEL_ALPHA = 0.76f
private val RowMinHeight = 56.dp
private val RowStartPadding = 10.dp
private val RowEndPadding = 14.dp
private val RowVerticalPadding = 8.dp
private val RowGap = 14.dp
private const val HOVER_ALPHA = 0.7f
private const val TITLE_WEIGHT = 650
private const val UNREAD_WEIGHT = 750
private const val FILL = 1f

/** A rounded panel holding [NomiRow]s (`.recent__list`, the list pages' rows). */
@Composable
fun NomiListPanel(modifier: Modifier = Modifier, content: @Composable ColumnScope.() -> Unit) {
    Column(
        modifier
            .fillMaxWidth()
            .clip(MaterialTheme.shapes.extraLarge)
            .background(MaterialTheme.colorScheme.surfaceContainerLowest.copy(alpha = PANEL_ALPHA))
            .padding(PanelPadding),
        content = content,
    )
}

/**
 * `.recent__row`: a [leading] avatar, the [title] over one quiet [line], and a mono [meta] at the end. Hover tints it
 * primary-container; [selected] holds that tint; a null [onClick] makes a plain, non-interactive row (the chat open beside the list); [emphasized] bolds an unread title.
 */
@Composable
fun NomiRow(
    title: String,
    onClick: (() -> Unit)?,
    modifier: Modifier = Modifier,
    line: String? = null,
    meta: String? = null,
    selected: Boolean = false,
    emphasized: Boolean = false,
    leading: (@Composable () -> Unit)? = null,
) {
    val colors = MaterialTheme.colorScheme
    val interaction = remember { MutableInteractionSource() }
    val hovered by interaction.collectIsHoveredAsState()
    val tint by animateColorAsState(
        when {
            selected -> colors.primaryContainer
            hovered -> colors.primaryContainer.copy(alpha = HOVER_ALPHA)
            else -> Color.Transparent
        },
        MaterialTheme.motionScheme.fastEffectsSpec(),
    )
    val shape = MaterialTheme.shapes.large
    Row(
        modifier
            .fillMaxWidth()
            .heightIn(min = RowMinHeight)
            .clip(shape)
            .background(tint)
            .semantics { this.selected = selected }
            .then(
                if (onClick == null) Modifier else Modifier
                    .nomiFocusRing(shape)
                    .hoverable(interaction)
                    .clickable(interaction, ripple(), role = Role.Button, onClick = onClick),
            )
            .padding(start = RowStartPadding, end = RowEndPadding, top = RowVerticalPadding, bottom = RowVerticalPadding),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(RowGap),
    ) {
        leading?.invoke()
        Column(Modifier.weight(FILL)) {
            Text(
                title,
                style = MaterialTheme.typography.bodyLarge.copy(fontWeight = FontWeight(if (emphasized) UNREAD_WEIGHT else TITLE_WEIGHT)),
                color = if (selected) colors.onPrimaryContainer else colors.onSurface,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            line?.let {
                Text(it, style = MaterialTheme.typography.bodyMedium, color = colors.onSurfaceVariant, maxLines = 1, overflow = TextOverflow.Ellipsis)
            }
        }
        meta?.let { Text(it, style = NomiTheme.metaStyle, color = colors.onSurfaceVariant, maxLines = 1) }
    }
}

private const val SECTION_TITLE_WEIGHT = 700

/** A section's heading on a page (`Recent`, `Your crew`): the brand face, with an optional [trailing] action. */
@Composable
fun NomiSectionHeader(title: String, modifier: Modifier = Modifier, trailing: @Composable () -> Unit = {}) {
    Row(modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(RowGap)) {
        Text(
            title,
            style = MaterialTheme.typography.titleLarge.copy(fontFamily = MaterialTheme.typography.displaySmall.fontFamily, fontWeight = FontWeight(SECTION_TITLE_WEIGHT)),
            color = MaterialTheme.colorScheme.onSurface,
            modifier = Modifier.weight(FILL).semantics { heading() },
        )
        trailing()
    }
}
