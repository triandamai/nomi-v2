package ai.nomi.designsystem.component

import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsFocusedAsState
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.semantics.error
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.unit.dp

/** The web's `.m3-field__input`: 48px tall, 1.5px outline, 2px primary when focused, 14px inside. */
private val FieldHeight = 48.dp
private val FieldPadding = 14.dp
private val FieldPaddingMultiline = 12.dp
private val OutlineWidth = 1.5.dp
private val FocusedOutlineWidth = 2.dp
private val LabelGap = 4.dp
private const val DISABLED_ALPHA = 0.38f
private const val ENABLED_ALPHA = 1f

/**
 * A text field the way the web draws one: its label above an outlined box (large corner, surface fill), the
 * supporting text below. The outline turns primary and thicker on focus, and error-red on [isError].
 */
@Composable
fun NomiTextField(
    value: String,
    onValueChange: (String) -> Unit,
    label: String,
    modifier: Modifier = Modifier,
    placeholder: String? = null,
    supportingText: String? = null,
    isError: Boolean = false,
    enabled: Boolean = true,
    singleLine: Boolean = true,
    /** Lines a multi-line field shows before it grows (a textarea's `rows`). */
    minLines: Int = 1,
    keyboardOptions: KeyboardOptions = KeyboardOptions.Default,
    keyboardActions: KeyboardActions = KeyboardActions.Default,
    visualTransformation: VisualTransformation = VisualTransformation.None,
) {
    val colors = MaterialTheme.colorScheme
    val interaction = remember { MutableInteractionSource() }
    val focused by interaction.collectIsFocusedAsState()
    val outline by animateColorAsState(
        when {
            isError -> colors.error
            focused -> colors.primary
            else -> colors.outline
        },
        MaterialTheme.motionScheme.fastEffectsSpec(),
    )
    val outlineWidth by animateDpAsState(if (focused) FocusedOutlineWidth else OutlineWidth, MaterialTheme.motionScheme.fastEffectsSpec())
    val shape = MaterialTheme.shapes.large
    val textStyle = MaterialTheme.typography.bodyLarge.copy(color = colors.onSurface)

    Column(modifier.alpha(if (enabled) ENABLED_ALPHA else DISABLED_ALPHA), verticalArrangement = Arrangement.spacedBy(LabelGap)) {
        Text(label, style = MaterialTheme.typography.bodySmall, color = colors.onSurfaceVariant)
        BasicTextField(
            value = value,
            onValueChange = onValueChange,
            enabled = enabled,
            singleLine = singleLine,
            minLines = if (singleLine) 1 else minLines,
            textStyle = textStyle,
            cursorBrush = SolidColor(if (isError) colors.error else colors.primary),
            keyboardOptions = keyboardOptions,
            keyboardActions = keyboardActions,
            visualTransformation = visualTransformation,
            interactionSource = interaction,
            modifier = Modifier
                .fillMaxWidth()
                .semantics { if (isError && supportingText != null) error(supportingText) },
            decorationBox = { inner ->
                Box(
                    Modifier
                        .fillMaxWidth()
                        .heightIn(min = FieldHeight)
                        .background(colors.surface, shape)
                        .border(outlineWidth, outline, shape)
                        .padding(horizontal = FieldPadding, vertical = if (singleLine) 0.dp else FieldPaddingMultiline),
                    contentAlignment = Alignment.CenterStart,
                ) {
                    if (value.isEmpty() && placeholder != null) {
                        Text(placeholder, style = textStyle.copy(color = colors.onSurfaceVariant))
                    }
                    inner()
                }
            },
        )
        supportingText?.let {
            Text(it, style = MaterialTheme.typography.bodySmall, color = if (isError) colors.error else colors.onSurfaceVariant)
        }
    }
}
