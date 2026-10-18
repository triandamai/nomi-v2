package ai.nomi.feature.auth

import ai.nomi.designsystem.component.NomiButton
import ai.nomi.designsystem.component.NomiButtonSize
import ai.nomi.designsystem.component.NomiButtonVariant
import ai.nomi.designsystem.component.NomiCard
import ai.nomi.designsystem.component.NomiLoadingIndicator
import ai.nomi.designsystem.component.NomiTextField
import ai.nomi.designsystem.layout.NomiSpacing
import ai.nomi.designsystem.theme.NomiTheme
import ai.nomi.network.VerificationInfo
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawingPadding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle

const val SIGN_IN_SCREEN_TAG = "sign-in-screen"
const val SIGN_IN_CODE_FIELD_TAG = "sign-in-code"

private val FormMaxWidth = 420.dp
private val BusyIndicatorSize = 32.dp

@Composable
fun SignInScreen(viewModel: SignInViewModel, modifier: Modifier = Modifier) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    Box(
        modifier
            .fillMaxSize()
            .background(NomiTheme.extendedColors.ground)
            .safeDrawingPadding()
            .padding(NomiSpacing.Large)
            .testTag(SIGN_IN_SCREEN_TAG),
        contentAlignment = Alignment.Center,
    ) {
        NomiCard(Modifier.widthIn(max = FormMaxWidth).fillMaxWidth()) {
            Column(Modifier.padding(NomiSpacing.ExtraLarge), verticalArrangement = Arrangement.spacedBy(NomiSpacing.Medium)) {
                val challenge = state.challenge
                if (challenge == null) PasswordStep(state, viewModel) else CodeStep(state, challenge, viewModel)
            }
        }
    }
}

@Composable
private fun ColumnScope.PasswordStep(state: SignInUiState, viewModel: SignInViewModel) {
    Text(
        "Sign in to Nomi",
        style = MaterialTheme.typography.headlineSmall,
        modifier = Modifier.semantics { heading() },
    )
    NomiTextField(
        value = state.email,
        onValueChange = viewModel::onEmailChange,
        label = "Email",
        enabled = !state.busy,
        modifier = Modifier.fillMaxWidth(),
        keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Email, imeAction = ImeAction.Next),
    )
    NomiTextField(
        value = state.password,
        onValueChange = viewModel::onPasswordChange,
        label = "Password",
        enabled = !state.busy,
        isError = state.error != null,
        supportingText = state.error,
        modifier = Modifier.fillMaxWidth(),
        visualTransformation = PasswordVisualTransformation(),
        keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Password, imeAction = ImeAction.Done),
        keyboardActions = KeyboardActions(onDone = { viewModel.submit() }),
    )
    PrimaryAction(busy = state.busy, busyDescription = "Signing in", label = "Sign in", onClick = viewModel::submit)
}

/** The emailed-code step, worded like the web's verify page. */
@Composable
private fun ColumnScope.CodeStep(state: SignInUiState, challenge: VerificationInfo, viewModel: SignInViewModel) {
    Text(
        "Check your email",
        style = MaterialTheme.typography.headlineSmall,
        modifier = Modifier.semantics { heading() },
    )
    Text(
        challenge.instructions(),
        style = MaterialTheme.typography.bodyMedium,
        color = MaterialTheme.colorScheme.onSurfaceVariant,
    )
    if (state.mustStartOver) {
        state.error?.let { Text(it, style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.error) }
        NomiButton(onClick = viewModel::startOver, variant = NomiButtonVariant.Filled, size = NomiButtonSize.Medium, modifier = Modifier.fillMaxWidth()) {
            Text("Start over")
        }
        return
    }
    NomiTextField(
        value = state.code,
        onValueChange = viewModel::onCodeChange,
        label = "Code",
        enabled = !state.busy,
        isError = state.error != null,
        supportingText = state.error ?: state.notice,
        modifier = Modifier.fillMaxWidth().testTag(SIGN_IN_CODE_FIELD_TAG),
        keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.NumberPassword, imeAction = ImeAction.Done),
        keyboardActions = KeyboardActions(onDone = { viewModel.submitCode() }),
    )
    PrimaryAction(busy = state.busy, busyDescription = "Checking", label = "Continue", onClick = viewModel::submitCode)
    Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
        NomiButton(onClick = viewModel::resendCode, variant = NomiButtonVariant.Text, size = NomiButtonSize.Medium, enabled = !state.busy) {
            Text("Send a new code")
        }
        NomiButton(onClick = viewModel::startOver, variant = NomiButtonVariant.Text, size = NomiButtonSize.Medium, enabled = !state.busy) {
            Text("Start over")
        }
    }
}

@Composable
private fun ColumnScope.PrimaryAction(busy: Boolean, busyDescription: String, label: String, onClick: () -> Unit) {
    if (busy) {
        NomiLoadingIndicator(Modifier.align(Alignment.CenterHorizontally), size = BusyIndicatorSize, contentDescription = busyDescription)
    } else {
        NomiButton(onClick = onClick, variant = NomiButtonVariant.Gradient, size = NomiButtonSize.Medium, modifier = Modifier.fillMaxWidth()) {
            Text(label)
        }
    }
}
