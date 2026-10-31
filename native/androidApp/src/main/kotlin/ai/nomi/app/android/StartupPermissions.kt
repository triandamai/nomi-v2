package ai.nomi.app.android

import android.Manifest
import android.app.AlertDialog
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.net.Uri
import android.os.Build
import android.provider.Settings
import androidx.activity.ComponentActivity
import androidx.activity.result.contract.ActivityResultContracts

private const val PREFS_NAME = "startup_permissions"
private const val KEY_OVERLAY_ASKED = "overlay_asked"

/**
 * What Nomi asks for when it opens: notifications (Android 13+, the system prompt) for replies and the companion's
 * ongoing notice, then "display over other apps" for the chat head, which only Settings can grant. Notifications are
 * asked on every open until answered (the system itself stops showing the prompt after two refusals); the overlay is
 * explained once per install, since it leaves the app. Create it in `onCreate`, before the activity starts.
 */
class StartupPermissions(private val activity: ComponentActivity) {
    private val prefs by lazy { activity.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE) }

    private val notifications =
        activity.registerForActivityResult(ActivityResultContracts.RequestPermission()) { askForOverlay() }

    fun request() {
        if (needsNotificationPermission()) {
            notifications.launch(Manifest.permission.POST_NOTIFICATIONS)
        } else {
            askForOverlay()
        }
    }

    private fun needsNotificationPermission(): Boolean =
        Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU &&
            activity.checkSelfPermission(Manifest.permission.POST_NOTIFICATIONS) != PackageManager.PERMISSION_GRANTED

    private fun askForOverlay() {
        if (Settings.canDrawOverlays(activity) || prefs.getBoolean(KEY_OVERLAY_ASKED, false)) return
        if (activity.isFinishing || activity.isDestroyed) return
        prefs.edit().putBoolean(KEY_OVERLAY_ASKED, true).apply()
        AlertDialog.Builder(activity)
            .setTitle(R.string.permission_overlay_title)
            .setMessage(R.string.permission_overlay_message)
            .setPositiveButton(R.string.permission_overlay_allow) { _, _ ->
                activity.startActivity(
                    Intent(Settings.ACTION_MANAGE_OVERLAY_PERMISSION, Uri.parse("package:${activity.packageName}")),
                )
            }
            .setNegativeButton(R.string.permission_not_now, null)
            .show()
    }
}
