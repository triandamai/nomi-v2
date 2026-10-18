package ai.nomi.designsystem.component

import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.compositionLocalOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.foundation.interaction.collectIsFocusedAsState
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.isShiftPressed
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.onPreviewKeyEvent
import androidx.compose.ui.input.key.type
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardCapitalization
import androidx.compose.ui.text.input.TextFieldValue
import androidx.compose.ui.unit.dp

/** Whether the person types on a hardware keyboard (Enter sends) or a soft one (the IME's Send action sends). */
enum class ComposerPlatform { Desktop, Mobile }

/** The shell provides [ComposerPlatform.Desktop] on desktop; everything else is a soft-keyboard device. */
val LocalComposerPlatform = compositionLocalOf { ComposerPlatform.Mobile }

enum class ComposerKeyAction { Send, Newline, Ignore }

/**
 * What a key press does in the composer. Desktop: Enter sends, Shift+Enter is a newline; while an IME composition is
 * active ([composing]) Enter belongs to the IME (it commits the candidate), so it is left alone.
 * Mobile: Enter, hardware or soft, is never a send shortcut and stays a newline in the field: sending is the IME Send action.
 */
fun composerKeyAction(key: Key, shift: Boolean, platform: ComposerPlatform, composing: Boolean = false): ComposerKeyAction = when {
    platform == ComposerPlatform.Mobile || composing -> ComposerKeyAction.Ignore
    key != Key.Enter && key != Key.NumPadEnter -> ComposerKeyAction.Ignore
    shift -> ComposerKeyAction.Newline
    else -> ComposerKeyAction.Send
}

/** The field grows with its text up to this many lines, then scrolls (the web caps it at 200px of 24px lines). */
const val COMPOSER_MAX_LINES = 8
private const val COMPOSER_MIN_LINES = 1

/** The field takes all the width the leading slot and send button leave. */
private const val FILL = 1f
private val ComposerPadding = 8.dp
private val ComposerFieldPadding = 12.dp
private val ComposerGap = 8.dp
private val FocusRingWidth = 2.dp
private val FieldMinHeight = 48.dp
private val ComposerShadow = 6.dp

/**
 * The floating message box: a rounded surface holding an auto-growing field, an optional [leading] slot (attach,
 * thinking level…) and the [SendButton]. State is hoisted: [value] is the draft, [sending] the turn in flight.
 */
@Composable
fun Composer(
    value: String,
    onValueChange: (String) -> Unit,
    onSend: () -> Unit,
    placeholder: String,
    sendLabel: String,
    modifier: Modifier = Modifier,
    sending: Boolean = false,
    onStop: (() -> Unit)? = null,
    stopLabel: String = sendLabel,
    platform: ComposerPlatform = LocalComposerPlatform.current,
    leading: (@Composable () -> Unit)? = null,
) {
    val colors = MaterialTheme.colorScheme
    val state = sendButtonState(value, sending)
    val interaction = remember { MutableInteractionSource() }
    val focused by interaction.collectIsFocusedAsState()
    val send = { if (state == SendButtonState.Ready) onSend() }
    // The field is driven by a TextFieldValue so the IME composition range is known; the draft stays the caller's String.
    var field by remember { mutableStateOf(TextFieldValue(value)) }
    if (field.text != value) field = TextFieldValue(value)
    Surface(
        modifier = modifier.fillMaxWidth().then(
            if (focused) Modifier.border(FocusRingWidth, colors.primary, MaterialTheme.shapes.extraLarge) else Modifier,
        ),
        shape = MaterialTheme.shapes.extraLarge,
        color = colors.surfaceContainerLowest,
        contentColor = colors.onSurface,
        shadowElevation = ComposerShadow,
    ) {
        Row(
            Modifier.padding(ComposerPadding),
            horizontalArrangement = Arrangement.spacedBy(ComposerGap),
            verticalAlignment = Alignment.Bottom,
        ) {
            leading?.invoke()
            BasicTextField(
                value = field,
                onValueChange = { next ->
                    field = next
                    if (next.text != value) onValueChange(next.text)
                },
                modifier = Modifier
                    .weight(FILL)
                    .heightIn(min = FieldMinHeight)
                    .onPreviewKeyEvent { event ->
                        if (event.type != KeyEventType.KeyDown) return@onPreviewKeyEvent false
                        when (composerKeyAction(event.key, event.isShiftPressed, platform, composing = field.composition != null)) {
                            ComposerKeyAction.Send -> { send(); true }
                            // Shift+Enter: let the field insert the line break itself.
                            ComposerKeyAction.Newline, ComposerKeyAction.Ignore -> false
                        }
                    },
                textStyle = MaterialTheme.typography.bodyLarge.copy(color = colors.onSurface),
                cursorBrush = SolidColor(colors.primary),
                minLines = COMPOSER_MIN_LINES,
                maxLines = COMPOSER_MAX_LINES,
                interactionSource = interaction,
                keyboardOptions = KeyboardOptions(
                    capitalization = KeyboardCapitalization.Sentences,
                    imeAction = if (platform == ComposerPlatform.Mobile) ImeAction.Send else ImeAction.Default,
                ),
                keyboardActions = KeyboardActions(onSend = { send() }),
                decorationBox = { inner ->
                    Box(Modifier.padding(horizontal = ComposerFieldPadding, vertical = ComposerFieldPadding), contentAlignment = Alignment.CenterStart) {
                        if (value.isEmpty()) {
                            Text(placeholder, style = MaterialTheme.typography.bodyLarge, color = colors.onSurfaceVariant)
                        }
                        inner()
                    }
                },
            )
            SendButton(
                state = state,
                onSend = send,
                onStop = onStop,
                sendLabel = sendLabel,
                stopLabel = stopLabel,
            )
        }
    }
}
