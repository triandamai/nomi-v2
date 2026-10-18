package ai.nomi.app.android

import ai.nomi.companion.CompanionController
import ai.nomi.data.SessionGate
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import org.koin.core.component.KoinComponent
import org.koin.core.component.inject
import org.koin.core.qualifier.named

/** Restarts [NomiPresenceService] after device reboot when the person is signed in and has the chat head enabled. */
class BootReceiver : BroadcastReceiver(), KoinComponent {
    private val gate: SessionGate by inject()
    private val companion: CompanionController by inject()
    private val appScope: CoroutineScope by inject(named("app"))

    override fun onReceive(context: Context, intent: Intent?) {
        if (intent?.action != Intent.ACTION_BOOT_COMPLETED) return
        val pending = goAsync()
        appScope.launch {
            try {
                gate.restore()
                if (gate.signedIn.value && companion.bubbleEnabled.value) {
                    NomiPresenceService.startIfEligible(context)
                }
            } finally {
                pending.finish()
            }
        }
    }
}
