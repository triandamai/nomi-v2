package ai.nomi.app.android

import ai.nomi.app.NomiApp
import ai.nomi.companion.CompanionController
import android.content.Intent
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.lifecycle.lifecycleScope
import kotlinx.coroutines.launch
import org.koin.android.ext.android.inject

class MainActivity : ComponentActivity() {
    private val overlayController: BubbleOverlayController by inject()
    private val companion: CompanionController by inject()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        handleDeepLink(intent)

        lifecycleScope.launch {
            companion.bubbleEnabled.collect { bubbleEnabled ->
                if (bubbleEnabled) {
                    NomiPresenceService.startIfEligible(applicationContext)
                }
            }
        }

        setContent { NomiApp() }
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        setIntent(intent)
        handleDeepLink(intent)
    }

    override fun onStart() {
        super.onStart()
        overlayController.setAppInForeground(true)
    }

    override fun onStop() {
        overlayController.setAppInForeground(false)
        super.onStop()
    }

    private fun handleDeepLink(intent: Intent?) {
        val link = intent?.getStringExtra(EXTRA_DEEP_LINK)?.takeIf { it.isNotBlank() } ?: return
        companion.openLink(link)
    }

    companion object {
        const val EXTRA_DEEP_LINK = "ai.nomi.app.EXTRA_DEEP_LINK"
    }
}
