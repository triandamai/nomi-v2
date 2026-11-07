package ai.nomi.app

import androidx.compose.runtime.Composable

/**
 * Opens the web's Google hand-off page so its answer comes back as a `nomi://auth/google` link (delivered to
 * [ai.nomi.data.GoogleSignInReturns]); null where the platform can't receive that link.
 */
@Composable
expect fun rememberGoogleSignIn(): ((String) -> Unit)?
