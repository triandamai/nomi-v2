package ai.nomi.feature.auth

import ai.nomi.designsystem.brand.NomiAuthStage
import ai.nomi.designsystem.brand.brandStyle
import ai.nomi.designsystem.component.GoogleButton
import ai.nomi.designsystem.component.NomiButton
import ai.nomi.designsystem.component.NomiButtonSize
import ai.nomi.designsystem.component.NomiButtonVariant
import ai.nomi.designsystem.component.NomiSnackbarHost
import ai.nomi.designsystem.component.NomiTextField
import ai.nomi.designsystem.ds.Ds
import ai.nomi.designsystem.ds.DsIcon
import ai.nomi.designsystem.ds.DsMotion
import ai.nomi.designsystem.ds.DsSpinner
import ai.nomi.designsystem.icon.Lucide
import ai.nomi.designsystem.theme.NomiTheme
import ai.nomi.network.VerificationInfo
import androidx.compose.animation.AnimatedContent
import androidx.compose.animation.AnimatedContentTransitionScope
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.ContentTransform
import androidx.compose.animation.EnterTransition
import androidx.compose.animation.ExitTransition
import androidx.compose.animation.SizeTransform
import androidx.compose.animation.core.CubicBezierEasing
import androidx.compose.animation.core.snap
import androidx.compose.animation.core.tween
import androidx.compose.animation.expandVertically
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.shrinkVertically
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutVertically
import androidx.compose.animation.togetherWith
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
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
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawingPadding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.LocalContentColor
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.semantics.LiveRegionMode
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.heading
import androidx.compose.ui.semantics.liveRegion
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.stateDescription
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.unit.Density
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.em
import androidx.compose.ui.unit.sp
import androidx.lifecycle.compose.LifecycleResumeEffect
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
private val SpinnerSize = 20.dp
private val ActionTop = 8.dp
private val ForgotGap = 8.dp
private val SentGap = 12.dp
private val LinkTarget = 44.dp
private val LinkPadding = 4.dp
private val EyeSize = 20.dp
private val OrMargin = 16.dp
private val OrGap = 12.dp
private val SnackbarPadding = 16.dp
private const val TITLE_WIDE_SP = 44f
private const val TITLE_COMPACT_SP = 34f
private const val TITLE_TRACKING_EM = -0.04f
private const val FILL = 1f

/** What the sign-in pages can do; every action defaults to nothing (for previews). */
data class SignInActions(
    val onEmailChange: (String) -> Unit = {},
    val onPasswordChange: (String) -> Unit = {},
    val onSubmit: () -> Unit = {},
    val onTogglePassword: () -> Unit = {},
    val onCodeChange: (String) -> Unit = {},
    val onSubmitCode: () -> Unit = {},
    val onResendCode: () -> Unit = {},
    val onStartOver: () -> Unit = {},
    val onForgot: () -> Unit = {},
    val onCloseForgot: () -> Unit = {},
    val onSendReset: () -> Unit = {},
    val onForgotAgain: () -> Unit = {},
    /** "Continue with Google"; null where the platform can't bring Google's answer back (desktop). */
    val onGoogle: (() -> Unit)? = null,
)

/**
 * The sign-in screen. [openGoogle] shows the web's Google hand-off page in the browser (null: no Google button).
 */
@Composable
fun SignInScreen(viewModel: SignInViewModel, modifier: Modifier = Modifier, openGoogle: ((String) -> Unit)? = null) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    val snackbar = remember { SnackbarHostState() }
    LaunchedEffect(state.toast) {
        val toast = state.toast ?: return@LaunchedEffect
        viewModel.toastShown()
        snackbar.showSnackbar(toast)
    }
    LifecycleResumeEffect(viewModel) {
        viewModel.onResumed()
        onPauseOrDispose {}
    }
    Box(modifier) {
        SignInContent(
            state,
            SignInActions(
                onEmailChange = viewModel::onEmailChange,
                onPasswordChange = viewModel::onPasswordChange,
                onSubmit = viewModel::submit,
                onTogglePassword = viewModel::togglePasswordVisible,
                onCodeChange = viewModel::onCodeChange,
                onSubmitCode = viewModel::submitCode,
                onResendCode = viewModel::resendCode,
                onStartOver = viewModel::startOver,
                onForgot = viewModel::openForgot,
                onCloseForgot = viewModel::closeForgot,
                onSendReset = viewModel::sendResetLink,
                onForgotAgain = viewModel::forgotAgain,
                onGoogle = openGoogle?.let { open -> { viewModel.continueWithGoogle(open) } },
            ),
        )
        NomiSnackbarHost(snackbar, Modifier.align(Alignment.BottomCenter).navigationBarsPadding().padding(SnackbarPadding))
    }
}

/** The sign-in screen for a given [state]: the crew's stage beside (or above, on phones) the form. */
@Composable
fun SignInContent(state: SignInUiState, actions: SignInActions = SignInActions(), modifier: Modifier = Modifier) {
    val reduced = NomiTheme.reducedMotion
    val density = LocalDensity.current
    val form: @Composable ColumnScope.(Boolean) -> Unit = { wide ->
        // The steps move like the web's pages: forward rises in, back drops in (`AuthShell`'s view transition).
        AnimatedContent(
            state.step,
            transitionSpec = { stepTransition(initialState, targetState, reduced, density) },
            label = "sign-in-step",
        ) { step ->
            Column(verticalArrangement = Arrangement.spacedBy(FormGap)) {
                when (step) {
                    SignInStep.Password -> PasswordStep(state, wide, actions)
                    SignInStep.Code -> state.challenge?.let { CodeStep(state, it, wide, actions) }
                    SignInStep.Forgot -> ForgotStep(state, wide, actions)
                    SignInStep.ResetSent -> ResetSentStep(state.resetSentTo.orEmpty(), wide, actions)
                }
            }
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
                    Column(Modifier.widthIn(max = FormMaxWidth).fillMaxWidth().padding(PanelPadding)) { form(true) }
                }
            }
        } else {
            Column(Modifier.fillMaxSize().imePadding().verticalScroll(rememberScrollState()).safeDrawingPadding()) {
                NomiAuthStage(compact = true, modifier = Modifier.fillMaxWidth().height(CompactStageHeight).padding(StageMargin))
                Column(Modifier.align(Alignment.CenterHorizontally).widthIn(max = FormMaxWidth).fillMaxWidth().padding(PanelPadding)) { form(false) }
            }
        }
    }
}

private const val STEP_OUT_MS = 160
private const val STEP_IN_MS = 420
private const val STEP_IN_DELAY_MS = 60
private val StepOutEasing = CubicBezierEasing(0.3f, 0f, 0.8f, 0.15f)
private val StepInEasing = CubicBezierEasing(0.05f, 0.7f, 0.1f, 1f)
private val StepRise = 16.dp
private val StepDrop = 12.dp
private val StepLeave = 8.dp

/** Forward (deeper into a flow) rises in from below; back comes down from above. Size follows on the spatial curve. */
private fun AnimatedContentTransitionScope<SignInStep>.stepTransition(
    from: SignInStep,
    to: SignInStep,
    reduced: Boolean,
    density: Density,
): ContentTransform {
    if (reduced) return (EnterTransition.None togetherWith ExitTransition.None).using(SizeTransform(clip = false) { _, _ -> snap() })
    val back = to == SignInStep.Password || (from == SignInStep.ResetSent && to == SignInStep.Forgot)
    val rise = with(density) { (if (back) -StepDrop else StepRise).roundToPx() }
    val leave = with(density) { (if (back) StepLeave else -StepLeave).roundToPx() }
    val enter = fadeIn(tween(STEP_IN_MS, STEP_IN_DELAY_MS, StepInEasing)) + slideInVertically(tween(STEP_IN_MS, STEP_IN_DELAY_MS, StepInEasing)) { rise }
    val exit = fadeOut(tween(STEP_OUT_MS, easing = StepOutEasing)) + slideOutVertically(tween(STEP_OUT_MS, easing = StepOutEasing)) { leave }
    return (enter togetherWith exit).using(SizeTransform(clip = false) { _, _ -> tween(STEP_IN_MS, easing = StepInEasing) })
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
private fun ColumnScope.PasswordStep(state: SignInUiState, wide: Boolean, actions: SignInActions) {
    Heading("Welcome back", "Log in and pick up where your crew left off.", wide)
    Column {
        actions.onGoogle?.let { google ->
            GoogleButton(onClick = google, busy = state.googleBusy)
            OrDivider()
        }
        Column(verticalArrangement = Arrangement.spacedBy(FieldGap)) {
            AnimatedVisibility(state.error != null, enter = expandVertically() + fadeIn(), exit = shrinkVertically() + fadeOut()) {
                ErrorLine(state.error.orEmpty())
            }
            NomiTextField(
                value = state.email,
                onValueChange = actions.onEmailChange,
                label = "Email",
                enabled = !state.busy,
                isError = state.error != null,
                modifier = Modifier.fillMaxWidth(),
                keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Email, imeAction = ImeAction.Next),
            )
            Column(verticalArrangement = Arrangement.spacedBy(ForgotGap)) {
                PasswordField(state, actions)
                Text(
                    "Forgot password?",
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.primary,
                    modifier = Modifier
                        .align(Alignment.End)
                        .clip(MaterialTheme.shapes.small)
                        .clickable(role = Role.Button, onClick = actions.onForgot)
                        .heightIn(min = LinkTarget)
                        .wrapContentHeight()
                        .padding(horizontal = LinkPadding),
                )
            }
            PrimaryAction(state.busy, busyDescription = "Signing in", label = "Log in", onClick = actions.onSubmit)
        }
    }
}

/** The password, with the web's eye to show it as typed; the glyph crossfades as it flips. */
@Composable
private fun PasswordField(state: SignInUiState, actions: SignInActions) {
    val fadeIn = fadeIn(DsMotion.effects())
    val fadeOut = fadeOut(DsMotion.effects())
    NomiTextField(
        value = state.password,
        onValueChange = actions.onPasswordChange,
        label = "Password",
        enabled = !state.busy,
        isError = state.error != null,
        modifier = Modifier.fillMaxWidth(),
        visualTransformation = if (state.showPassword) VisualTransformation.None else PasswordVisualTransformation(),
        keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Password, imeAction = ImeAction.Done),
        keyboardActions = KeyboardActions(onDone = { actions.onSubmit() }),
        trailing = {
            Box(
                Modifier.size(LinkTarget).clip(CircleShape).clickable(
                    role = Role.Button,
                    onClickLabel = if (state.showPassword) "Hide password" else "Show password",
                    onClick = actions.onTogglePassword,
                ).semantics { contentDescription = if (state.showPassword) "Hide password" else "Show password" },
                contentAlignment = Alignment.Center,
            ) {
                AnimatedContent(state.showPassword, transitionSpec = { fadeIn togetherWith fadeOut }, label = "password-eye") { shown ->
                    DsIcon(if (shown) Lucide.EyeOff else Lucide.Eye, EyeSize, MaterialTheme.colorScheme.onSurfaceVariant)
                }
            }
        },
    )
}

/** `.or`: a hairline either side of a quiet mono "OR". */
@Composable
private fun OrDivider() {
    Row(Modifier.fillMaxWidth().padding(vertical = OrMargin), horizontalArrangement = Arrangement.spacedBy(OrGap), verticalAlignment = Alignment.CenterVertically) {
        HorizontalDivider(Modifier.weight(FILL), color = MaterialTheme.colorScheme.outlineVariant)
        Text("OR", style = Ds.labelCaps, color = MaterialTheme.colorScheme.onSurfaceVariant)
        HorizontalDivider(Modifier.weight(FILL), color = MaterialTheme.colorScheme.outlineVariant)
    }
}

/** `forgot-password/+page.svelte`, before sending: the email, and the link's way back. */
@Composable
private fun ColumnScope.ForgotStep(state: SignInUiState, wide: Boolean, actions: SignInActions) {
    Heading("Forgot your password?", "Enter your email and we'll send you a link to choose a new one.", wide)
    Column(verticalArrangement = Arrangement.spacedBy(FieldGap)) {
        NomiTextField(
            value = state.email,
            onValueChange = actions.onEmailChange,
            label = "Email",
            enabled = !state.busy,
            isError = state.error != null,
            supportingText = state.error,
            modifier = Modifier.fillMaxWidth(),
            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Email, imeAction = ImeAction.Send),
            keyboardActions = KeyboardActions(onSend = { actions.onSendReset() }),
        )
        PrimaryAction(state.busy, busyDescription = "Sending", label = "Send reset link", onClick = actions.onSendReset)
        SwitchLine("Have an account?", "Log in", actions.onCloseForgot, enabled = !state.busy)
    }
}

/** After sending: where the link went, back to sign in, or another address. */
@Composable
private fun ColumnScope.ResetSentStep(email: String, wide: Boolean, actions: SignInActions) {
    Heading(
        "Check your email",
        "If $email has a Nomi account, a link to choose a new password is on its way. It works for an hour. " +
            "Check your spam folder if it isn't there.",
        wide,
    )
    Column(verticalArrangement = Arrangement.spacedBy(SentGap)) {
        NomiButton(onClick = actions.onCloseForgot, variant = NomiButtonVariant.Gradient, size = NomiButtonSize.Medium, modifier = Modifier.fillMaxWidth()) {
            Text("Back to sign in")
        }
        NomiButton(onClick = actions.onForgotAgain, variant = NomiButtonVariant.Text, size = NomiButtonSize.Medium, modifier = Modifier.fillMaxWidth()) {
            Text("Use a different email")
        }
    }
}

/** `AuthShell`'s switch line ("Have an account? Log in"). */
@Composable
private fun SwitchLine(prompt: String, label: String, onClick: () -> Unit, enabled: Boolean = true) {
    Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.Center, verticalAlignment = Alignment.CenterVertically) {
        Text(prompt, style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
        NomiButton(onClick = onClick, variant = NomiButtonVariant.Text, size = NomiButtonSize.Small, enabled = enabled) { Text(label) }
    }
}

/** The emailed-code step, worded like the web's verify page. */
@Composable
private fun ColumnScope.CodeStep(state: SignInUiState, challenge: VerificationInfo, wide: Boolean, actions: SignInActions) {
    Heading("Check your email", challenge.instructions(), wide)
    Column(verticalArrangement = Arrangement.spacedBy(FieldGap)) {
        if (state.mustStartOver) {
            state.error?.let { ErrorLine(it) }
            NomiButton(onClick = actions.onStartOver, variant = NomiButtonVariant.Gradient, size = NomiButtonSize.Medium, modifier = Modifier.fillMaxWidth()) {
                Text("Start over")
            }
            return@Column
        }
        NomiTextField(
            value = state.code,
            onValueChange = actions.onCodeChange,
            label = "Code",
            enabled = !state.busy,
            isError = state.error != null,
            supportingText = state.error ?: state.notice,
            modifier = Modifier.fillMaxWidth().testTag(SIGN_IN_CODE_FIELD_TAG),
            keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.NumberPassword, imeAction = ImeAction.Done),
            keyboardActions = KeyboardActions(onDone = { actions.onSubmitCode() }),
        )
        PrimaryAction(state.busy, busyDescription = "Checking", label = "Continue", onClick = actions.onSubmitCode)
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
            Text("Didn't get it?", style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
            NomiButton(onClick = actions.onResendCode, variant = NomiButtonVariant.Text, size = NomiButtonSize.Small, enabled = !state.busy) {
                Text("Send a new code")
            }
        }
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
            Text("Wrong email?", style = MaterialTheme.typography.bodyMedium, color = MaterialTheme.colorScheme.onSurfaceVariant)
            NomiButton(onClick = actions.onStartOver, variant = NomiButtonVariant.Text, size = NomiButtonSize.Small, enabled = !state.busy) {
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

/**
 * The web's `Button loading`: the gradient button keeps its place and its label gives way to a spinner, so the form
 * never jumps.
 */
@Composable
private fun PrimaryAction(busy: Boolean, busyDescription: String, label: String, onClick: () -> Unit) {
    val fadeIn = fadeIn(DsMotion.effects())
    val fadeOut = fadeOut(DsMotion.effects())
    NomiButton(
        onClick = { if (!busy) onClick() },
        variant = NomiButtonVariant.Gradient,
        size = NomiButtonSize.Medium,
        modifier = Modifier.fillMaxWidth().padding(top = ActionTop).semantics { if (busy) stateDescription = busyDescription },
    ) {
        AnimatedContent(busy, transitionSpec = { fadeIn togetherWith fadeOut }, label = "primary-busy") { loading ->
            if (loading) DsSpinner(SpinnerSize, LocalContentColor.current) else Text(label)
        }
    }
}
