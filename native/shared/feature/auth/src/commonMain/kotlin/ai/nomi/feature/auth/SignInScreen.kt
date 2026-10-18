package ai.nomi.feature.auth

import ai.nomi.designsystem.component.NomiButton
import ai.nomi.designsystem.component.NomiButtonSize
import ai.nomi.designsystem.component.NomiButtonVariant
import ai.nomi.designsystem.component.NomiCard
import ai.nomi.designsystem.component.NomiLoadingIndicator
import ai.nomi.designsystem.component.NomiTextField
import ai.nomi.designsystem.layout.NomiSpacing
import ai.nomi.designsystem.theme.NomiTheme
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
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
                state.codeSentTo?.let { email ->
                    Text(
                        "We sent a sign-in code to $email. Signing in with a code isn't supported in this app yet; " +
                            "use the web app to finish, or sign in from a device you've used before.",
                        style = MaterialTheme.typography.bodyMedium,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
                if (state.busy) {
                    NomiLoadingIndicator(Modifier.align(Alignment.CenterHorizontally), size = BusyIndicatorSize, contentDescription = "Signing in")
                } else {
                    NomiButton(
                        onClick = viewModel::submit,
                        variant = NomiButtonVariant.Gradient,
                        size = NomiButtonSize.Medium,
                        modifier = Modifier.fillMaxWidth(),
                    ) { Text("Sign in") }
                }
            }
        }
    }
}
