package ai.nomi.designsystem.component

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Snackbar
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier

/** The inverse-surface snackbar. Let it size itself: M3's Snackbar overlaps its message and action under `fillMaxWidth`. */
@Composable
fun NomiSnackbar(message: String, modifier: Modifier = Modifier, actionLabel: String? = null, onAction: () -> Unit = {}) {
    val colors = MaterialTheme.colorScheme
    Snackbar(
        modifier = modifier,
        action = actionLabel?.let { label ->
            { TextButton(onClick = onAction) { Text(label, color = colors.inversePrimary) } }
        },
        shape = MaterialTheme.shapes.large,
        containerColor = colors.inverseSurface,
        contentColor = colors.inverseOnSurface,
    ) { Text(message) }
}

/** A host for [NomiSnackbar]s; show messages through [SnackbarHostState.showSnackbar]. */
@Composable
fun NomiSnackbarHost(state: SnackbarHostState, modifier: Modifier = Modifier) {
    SnackbarHost(state, modifier) { data ->
        NomiSnackbar(
            message = data.visuals.message,
            actionLabel = data.visuals.actionLabel,
            onAction = { data.performAction() },
        )
    }
}
