package ai.nomi.feature.auth

import ai.nomi.designsystem.brand.NomiAuthStage
import ai.nomi.designsystem.brand.brandStyle
import ai.nomi.designsystem.component.NomiButton
import ai.nomi.designsystem.component.NomiButtonSize
import ai.nomi.designsystem.component.NomiButtonVariant
import ai.nomi.designsystem.component.NomiLoadingIndicator
import ai.nomi.designsystem.component.NomiTextField
import ai.nomi.designsystem.theme.NomiTheme
import ai.nomi.network.VerificationInfo
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawingPadding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.semantics.LiveRegionMode
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.liveRegion
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.em
import androidx.compose.ui.unit.sp
import androidx.lifecycle.compose.collectAsStateWithLifecycle

const val SIGN_IN_SCREEN_TAG = "sign-in-screen"
const val SIGN_IN_CODE_FIELD_TAG = "sign-in-code"

/** `AuthShell.svelte`: below 900px the stage becomes a band above the form. */
private val WideBreakpoint = 900.dp
private val StageMargin = 12.dp
private val CompactStageHeight = 320.dp
private val FormMaxWidth = 420.dp
private val PanelPadding = 24.dp
private val FormGap = 22.dp
private val FieldGap = 20.dp
private val HeadingGap = 10.dp
private val BusyIndicatorSize = 32.dp
/** M3 medium button height, so the loader takes the button's place without the form jumping. */
private val ButtonHeight = 56.dp
private const val TITLE_WIDE_SP = 44f
private const val TITLE_COMPACT_SP = 34f
private const val TITLE_TRACKING_EM = -0.04f
private const val FILL = 1f

@Composable
fun SignInScreen(viewModel: SignInViewModel, modifier: Modifier = Modifier) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    SignInContent(
        state,
        onEmailChange = viewModel::onEmailChange,
        onPasswordChange = viewModel::onPasswordChange,
        onSubmit = viewModel::submit,
        onCodeChange = viewModel::onCodeChange,
        onSubmitCode = viewModel::submitCode,
        onResendCode = viewModel::resendCode,
        onStartOver = viewModel::startOver,
        modifier = modifier,
    )
}

/** The sign-in screen for a given [state]: the crew's stage beside (or above, on phones) the form. */
@Composable
fun SignInContent(
    state: SignInUiState,
    onEmailChange: (String) -> Unit,
    onPasswordChange: (String) -> Unit,
    onSubmit: () -> Unit,
    onCodeChange: (String) -> Unit,
    onSubmitCode: () -> Unit,
    onResendCode: () -> Unit,
    onStartOver: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val form: @Composable ColumnScope.(Boolean) -> Unit = { wide ->
        val challenge = state.challenge
        if (challenge == null) {
            PasswordStep(state, wide, onEmailChange, onPasswordChange, onSubmit)
        } else {
            CodeStep(state, challenge, wide, onCodeChange, onSubmitCode, onResendCode, onStartOver)
        }
    }
    BoxWithConstraints(
        modifier
            .fillMaxSize()
            .background(NomiTheme.extendedColors.ground)
            .testTag(SIGN_IN_SCREEN_TAG),
    ) {
        if (maxWidth >= WideBreakpoint) {
            Row(Modifier.fillMaxSize().safeDrawingPadding()) {
                NomiAuthStage(compact = false, modifier = Modifier.weight(FILL).fillMaxHeight().padding(StageMargin))
                Box(Modifier.weight(FILL).fillMaxHeight().imePadding().verticalScroll(rememberScrollState()), contentAlignment = Alignment.Center) {
                    Column(
                        Modifier.widthIn(max = FormMaxWidth).fillMaxWidth().padding(PanelPadding),
                        verticalArrangement = Arrangement.spacedBy(FormGap),
                    ) { form(true) }
                }
            }
        } else {
            Column(Modifier.fillMaxSize().imePadding().verticalScroll(rememberScrollState()).safeDrawingPadding()) {
                NomiAuthStage(compact = true, modifier = Modifier.fillMaxWidth().height(CompactStageHeight).padding(StageMargin))
                Column(
                    Modifier.align(Alignment.CenterHorizontally).widthIn(max = FormMaxWidth).fillMaxWidth().padding(PanelPadding),
                    verticalArrangement = Arrangement.spacedBy(FormGap),
                ) { form(false) }
            }
        }
    }
}

/** `.panel__heading`: the brand title over a quiet subtitle. */
@Composable
private fun Heading(title: String, subtitle: String, wide: Boolean) {
    Column(verticalArrangement = Arrangement.spacedBy(HeadingGap)) {
        val size = if (wide) TITLE_WIDE_SP else TITLE_COMPACT_SP
        Text(
            title,
            style = brandStyle(size.sp).copy(letterSpacing = TITLE_TRACKING_EM.em),
            color = MaterialTheme.colorScheme.onSurface,
            modifier = Modifier.semantics { heading() },
        )
        Text(subtitle, style = MaterialTheme.typography.bodyLarge, color = MaterialTheme.colorScheme.onSurfaceVariant)
    }
}

@Composable
private fun ColumnScope.PasswordStep(
    state: SignInUiState,
    wide: Boolean,
    onEmailChange: (String) -> Unit,
    onPasswordChange: (String) -> Unit,
    onSubmit: () -> Unit,
) {
    Heading("Welcome back", "Log in and pick up where your crew left off.", wide)
    Column(verticalArrangement = Arrangement.spacedBy(FieldGap)) {
        state.error?.let { ErrorLine(it) }
        NomiTextField(
            value = state.email,
            onValueChange = onEmailChange,
            label = "Email",
            enabled = !state.busy,
            modifier = Modifier.fillMaxWidth(),
            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Email, imeAction = ImeAction.Next),
        )
        NomiTextField(
            value = state.password,
            onValueChange = onPasswordChange,
            label = "Password",
            enabled = !state.busy,
            isError = state.error != null,
            modifier = Modifier.fillMaxWidth(),
            visualTransformation = PasswordVisualTransformation(),
            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Password, imeAction = ImeAction.Done),
            keyboardActions = KeyboardActions(onDone = { onSubmit() }),
        )
        PrimaryAction(state.busy, busyDescription = "Signing in", label = "Log in", onClick = onSubmit)
    }
}

/** The emailed-code step, worded like the web's verify page. */
@Composable
private fun ColumnScope.CodeStep(
    state: SignInUiState,
    challenge: VerificationInfo,
    wide: Boolean,
    onCodeChange: (String) -> Unit,
    onSubmitCode: () -> Unit,
    onResendCode: () -> Unit,
    onStartOver: () -> Unit,
) {
    Heading("Check your email", challenge.instructions(), wide)
    Column(verticalArrangement = Arrangement.spacedBy(FieldGap)) {
        if (state.mustStartOver) {
            state.error?.let { ErrorLine(it) }
            NomiButton(onClick = onStartOver, variant = NomiButtonVariant.Gradient, size = NomiButtonSize.Medium, modifier = Modifier.fillMaxWidth()) {
                Text("Start over")
            }
            return@Column
        }
        NomiTextField(
            value = state.code,
            onValueChange = onCodeChange,
            label = "Code",
            enabled = !state.busy,
            isError = state.error != null,
            supportingText = state.error ?: state.notice,
            modifier = Modifier.fillMaxWidth().testTag(SIGN_IN_CODE_FIELD_TAG),
            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.NumberPassword, imeAction = ImeAction.Done),
            keyboardActions = KeyboardActions(onDone = { onSubmitCode() }),
        )
        PrimaryAction(state.busy, busyDescription = "Checking", label = "Continue", onClick = onSubmitCode)
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
            Text("Didn't get it?", style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
            NomiButton(onClick = onResendCode, variant = NomiButtonVariant.Text, size = NomiButtonSize.Small, enabled = !state.busy) {
                Text("Send a new code")
            }
        }
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
            Text("Wrong email?", style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
            NomiButton(onClick = onStartOver, variant = NomiButtonVariant.Text, size = NomiButtonSize.Small, enabled = !state.busy) {
                Text("Start over")
            }
        }
    }
}

@Composable
private fun ErrorLine(message: String) {
    Text(
        message,
        style = MaterialTheme.typography.bodyMedium,
        color = MaterialTheme.colorScheme.error,
        modifier = Modifier.semantics { liveRegion = LiveRegionMode.Polite },
    )
}

@Composable
private fun ColumnScope.PrimaryAction(busy: Boolean, busyDescription: String, label: String, onClick: () -> Unit) {
    if (busy) {
        Box(Modifier.fillMaxWidth().height(ButtonHeight), contentAlignment = Alignment.Center) {
            NomiLoadingIndicator(size = BusyIndicatorSize, contentDescription = busyDescription)
        }
    } else {
        NomiButton(onClick = onClick, variant = NomiButtonVariant.Gradient, size = NomiButtonSize.Medium, modifier = Modifier.fillMaxWidth()) {
            Text(label)
        }
    }
}
