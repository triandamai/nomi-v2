package ai.nomi.designsystem.component

import ai.nomi.designsystem.shape.AgentShape
import ai.nomi.designsystem.shape.NomiIdentity
import ai.nomi.designsystem.theme.NomiTheme
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.unit.dp

/** Width of the phone-sized frame every component sheet is captured in. */
val SheetWidth = 390.dp
private val SheetPadding = 16.dp
private val SheetGap = 12.dp
private const val HALF_PROGRESS = 0.6f
private val AgentSize = 56.dp

@Composable
private fun Sheet(content: @Composable () -> Unit) {
    Column(
        Modifier.width(SheetWidth).background(NomiTheme.extendedColors.ground).padding(SheetPadding),
        verticalArrangement = Arrangement.spacedBy(SheetGap),
    ) { content() }
}

@Composable
private fun Label(text: String) = Text(text, style = NomiTheme.metaStyle, color = MaterialTheme.colorScheme.onSurfaceVariant)

/** Buttons, chips, groups, split button. */
@Composable
fun ActionsSheet() = Sheet {
    Label("Buttons")
    FlowRow(horizontalArrangement = Arrangement.spacedBy(SheetGap), verticalArrangement = Arrangement.spacedBy(SheetGap)) {
        NomiButton({}, variant = NomiButtonVariant.Filled) { Text("Filled") }
        NomiButton({}, variant = NomiButtonVariant.Gradient) { Text("Gradient") }
        NomiButton({}, variant = NomiButtonVariant.Tonal) { Text("Tonal") }
        NomiButton({}, variant = NomiButtonVariant.Outlined) { Text("Outlined") }
        NomiButton({}, variant = NomiButtonVariant.Text) { Text("Text") }
        NomiButton({}, variant = NomiButtonVariant.Elevated) { Text("Elevated") }
        NomiButton({}, enabled = false) { Text("Disabled") }
    }
    Label("Sizes XS S M L")
    Row(horizontalArrangement = Arrangement.spacedBy(SheetGap), verticalAlignment = Alignment.CenterVertically) {
        NomiButton({}, size = NomiButtonSize.ExtraSmall) { Text("XS") }
        NomiButton({}, size = NomiButtonSize.Small) { Text("S") }
        NomiButton({}, size = NomiButtonSize.Medium) { Text("M") }
        NomiButton({}, size = NomiButtonSize.Large) { Text("L") }
    }
    Label("Send button: disabled, ready, sending, stoppable")
    Row(horizontalArrangement = Arrangement.spacedBy(SheetGap), verticalAlignment = Alignment.CenterVertically) {
        SendButton(SendButtonState.Disabled, {}, sendLabel = "Send")
        SendButton(SendButtonState.Ready, {}, sendLabel = "Send")
        SendButton(SendButtonState.Sending, {}, sendLabel = "Send")
        SendButton(SendButtonState.Sending, {}, onStop = {}, sendLabel = "Send")
        AgentShape(NomiIdentity, AgentSize)
    }
    Label("Chips")
    FlowRow(horizontalArrangement = Arrangement.spacedBy(SheetGap)) {
        NomiAssistChip("Assist", {})
        NomiSuggestionChip("Plan my Saturday", {})
        NomiFilterChip("Filter", selected = false, onClick = {})
        NomiFilterChip("Selected", selected = true, onClick = {})
    }
    Label("Button group")
    var picked by remember { mutableStateOf("week") }
    NomiButtonGroup(
        options = listOf(NomiGroupOption("day", "Day"), NomiGroupOption("week", "Week"), NomiGroupOption("month", "Month")),
        selected = picked, onSelect = { picked = it },
    )
    Label("Split button")
    NomiSplitButton("Save", {}, expanded = false, onExpandedChange = {}, menuLabel = "More") {
        DropdownMenu(false, {}) { DropdownMenuItem({ Text("Save as draft") }, {}) }
    }
}

/** Loaders, progress, card, text fields, snackbar, composer. */
@Composable
fun SurfacesSheet() = Sheet {
    Label("Loading indicator and wavy progress")
    Row(horizontalArrangement = Arrangement.spacedBy(SheetGap), verticalAlignment = Alignment.CenterVertically) {
        NomiLoadingIndicator()
        NomiWavyProgress({ HALF_PROGRESS }, "Uploading", Modifier.weight(1f))
    }
    Label("Cards")
    NomiCard(Modifier.fillMaxWidth()) { Text("Tonal card", Modifier.padding(SheetPadding), style = MaterialTheme.typography.bodyLarge) }
    NomiCard(Modifier.fillMaxWidth(), NomiCardVariant.Outlined) { Text("Outlined card", Modifier.padding(SheetPadding)) }
    Row(horizontalArrangement = Arrangement.spacedBy(SheetGap)) {
        NomiCard(Modifier.weight(1f), NomiCardVariant.Filled) { Text("Filled", Modifier.padding(SheetPadding)) }
        NomiCard(Modifier.weight(1f), NomiCardVariant.Elevated) { Text("Elevated", Modifier.padding(SheetPadding)) }
    }
    Label("Text fields")
    NomiTextField("Ada", {}, "Name", Modifier.fillMaxWidth(), supportingText = "What Nomi calls you")
    NomiTextField("", {}, "Email", Modifier.fillMaxWidth(), supportingText = "Enter a valid email", isError = true)
    Label("Snackbar")
    NomiSnackbar("Reminder saved", actionLabel = "Undo")
}

/** The floating composer, empty and with a draft. */
@Composable
fun ComposerSheet() = Sheet {
    Label("Composer: empty, typed, sending")
    Composer("", {}, {}, placeholder = "Message Nomi", sendLabel = "Send")
    Composer("Remind me to stretch at 5pm", {}, {}, placeholder = "Message Nomi", sendLabel = "Send")
    Composer("Remind me to stretch at 5pm", {}, {}, placeholder = "Message Nomi", sendLabel = "Send", sending = true, onStop = {})
}

/** Which control [FocusSheet] focuses. */
enum class FocusTarget { Gradient, Chip, Segment }

/** The gradient button, a chip and a group segment, with [target] focused so its ring shows. */
@Composable
fun FocusSheet(target: FocusTarget) = Sheet {
    Label("Focused: $target")
    val requesters = remember { FocusTarget.entries.associateWith { androidx.compose.ui.focus.FocusRequester() } }
    androidx.compose.runtime.LaunchedEffect(Unit) { requesters.getValue(target).requestFocus() }
    NomiButton({}, Modifier.focusRequester(requesters.getValue(FocusTarget.Gradient)), variant = NomiButtonVariant.Gradient) { Text("Gradient") }
    NomiSuggestionChip("Plan my Saturday", {}, Modifier.focusRequester(requesters.getValue(FocusTarget.Chip)))
    NomiButtonGroup(
        listOf(NomiGroupOption("a", "Day"), NomiGroupOption("b", "Week")), "a", {},
        Modifier.focusRequester(requesters.getValue(FocusTarget.Segment)),
    )
}
