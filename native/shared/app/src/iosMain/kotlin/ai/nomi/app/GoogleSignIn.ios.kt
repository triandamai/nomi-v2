package ai.nomi.app

import ai.nomi.data.GoogleSignInReturns
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import kotlinx.cinterop.ExperimentalForeignApi
import platform.AuthenticationServices.ASPresentationAnchor
import platform.AuthenticationServices.ASWebAuthenticationPresentationContextProvidingProtocol
import platform.AuthenticationServices.ASWebAuthenticationSession
import platform.Foundation.NSURL
import platform.UIKit.UIApplication
import platform.UIKit.UIWindow
import platform.UIKit.UIWindowScene
import platform.darwin.NSObject

private const val RETURN_SCHEME = "nomi"
private const val CANCELLED_LINK = "${GoogleSignInReturns.LINK_PREFIX}?error=cancelled"

/** Shows the sheet over the app's key window. */
private class KeyWindowAnchor : NSObject(), ASWebAuthenticationPresentationContextProvidingProtocol {
    override fun presentationAnchorForWebAuthenticationSession(session: ASWebAuthenticationSession): ASPresentationAnchor =
        UIApplication.sharedApplication.connectedScenes
            .filterIsInstance<UIWindowScene>()
            .flatMap { scene -> scene.windows.filterIsInstance<UIWindow>() }
            .firstOrNull { it.isKeyWindow() } ?: UIWindow()
}

/**
 * The system's web sign-in sheet, which ends on the `nomi://` link itself (no URL scheme to register). The anchor and
 * session are kept while the sheet is up.
 */
@OptIn(ExperimentalForeignApi::class)
@Composable
actual fun rememberGoogleSignIn(): ((String) -> Unit)? = remember {
    val anchor = KeyWindowAnchor()
    var current: ASWebAuthenticationSession? = null
    val open: (String) -> Unit = { url ->
        val target = NSURL.URLWithString(url)
        if (target != null) {
            val session = ASWebAuthenticationSession(target, RETURN_SCHEME) { callback, error ->
                current = null
                val link = callback?.absoluteString
                GoogleSignInReturns.deliver(if (error == null && link != null) link else CANCELLED_LINK)
            }
            session.presentationContextProvider = anchor
            session.prefersEphemeralWebBrowserSession = false
            current = session
            session.start()
        }
    }
    open
}
