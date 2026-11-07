package ai.nomi.app

import android.content.Intent
import android.net.Uri
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.platform.LocalContext

/** The browser; Google's answer returns through MainActivity's `nomi://auth/google` intent filter. */
@Composable
actual fun rememberGoogleSignIn(): ((String) -> Unit)? {
    val context = LocalContext.current
    return remember(context) {
        { url: String -> context.startActivity(Intent(Intent.ACTION_VIEW, Uri.parse(url)).addFlags(Intent.FLAG_ACTIVITY_NEW_TASK)) }
    }
}
