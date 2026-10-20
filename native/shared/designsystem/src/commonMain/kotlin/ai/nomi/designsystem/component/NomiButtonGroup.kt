package ai.nomi.designsystem.component

import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.core.spring
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.role
import androidx.compose.ui.semantics.selected
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp

data class ButtonGroupOption(
    val value: String,
    val label: String,
    val disabled: Boolean = false,
)

typealias NomiGroupOption = ButtonGroupOption

private val GroupHeight = 48.dp
private val GroupGap = 2.dp
private val OuterRound = 24.dp
private val InnerCorner = 8.dp

/**
 * M3 Expressive connected button group: segments sit 2dp apart with tight inner corners; the
 * selected segment springs to fully round (24dp pill). Single-select, matching web `ButtonGroup.svelte`.
 */
@Composable
fun NomiButtonGroup(
    options: List<ButtonGroupOption>,
    selected: String,
    onSelect: (String) -> Unit,
    modifier: Modifier = Modifier,
) {
    Row(
        modifier = modifier
            .fillMaxWidth()
            .height(GroupHeight),
        horizontalArrangement = Arrangement.spacedBy(GroupGap),
    ) {
        options.forEachIndexed { index, option ->
            val isSelected = option.value == selected
            val isFirst = index == 0
            val isLast = index == options.size - 1
            val isOnly = options.size == 1

            // Target corners based on position and selection
            val targetStart = if (isSelected || isFirst || isOnly) OuterRound else InnerCorner
            val targetEnd = if (isSelected || isLast || isOnly) OuterRound else InnerCorner

            val animatedStart by animateDpAsState(
                targetValue = targetStart,
                animationSpec = spring(),
                label = "btn-group-start-$index",
            )
            val animatedEnd by animateDpAsState(
                targetValue = targetEnd,
                animationSpec = spring(),
                label = "btn-group-end-$index",
            )

            val bgColor by animateColorAsState(
                targetValue = if (isSelected) {
                    MaterialTheme.colorScheme.primary
                } else {
                    MaterialTheme.colorScheme.surfaceContainerHigh
                },
                label = "btn-group-bg-$index",
            )
            val textColor by animateColorAsState(
                targetValue = if (isSelected) {
                    MaterialTheme.colorScheme.onPrimary
                } else {
                    MaterialTheme.colorScheme.onSurface
                },
                label = "btn-group-text-$index",
            )

            val shape = RoundedCornerShape(
                topStart = animatedStart,
                bottomStart = animatedStart,
                topEnd = animatedEnd,
                bottomEnd = animatedEnd,
            )

            Box(
                modifier = Modifier
                    .weight(1f)
                    .height(GroupHeight)
                    .clip(shape)
                    .background(bgColor)
                    .semantics {
                        this.selected = isSelected
                        this.role = Role.RadioButton
                    }
                    .clickable(enabled = !option.disabled) {
                        onSelect(option.value)
                    }
                    .padding(horizontal = 12.dp),
                contentAlignment = Alignment.Center,
            ) {
                Text(
                    text = option.label,
                    style = MaterialTheme.typography.labelLarge.copy(
                        fontWeight = if (isSelected) FontWeight.SemiBold else FontWeight.Medium,
                    ),
                    color = textColor,
                    maxLines = 1,
                )
            }
        }
    }
}
