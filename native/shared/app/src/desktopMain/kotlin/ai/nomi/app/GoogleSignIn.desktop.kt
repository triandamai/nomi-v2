package ai.nomi.app

import androidx.compose.runtime.Composable

/** The desktop app registers no `nomi://` handler, so Google's answer couldn't come back: no Google button. */
@Composable
actual fun rememberGoogleSignIn(): ((String) -> Unit)? = null
