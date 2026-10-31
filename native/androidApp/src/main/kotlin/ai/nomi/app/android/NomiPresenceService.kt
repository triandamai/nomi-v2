package ai.nomi.app.android

import ai.nomi.data.SessionGate
import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Context
import android.content.Intent
import android.content.pm.ServiceInfo
import android.os.Build
import android.os.IBinder
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import org.koin.android.ext.android.inject
import org.koin.core.qualifier.named

private const val PRESENCE_CHANNEL_ID = "presence"
private const val PRESENCE_CHANNEL_NAME = "Nomi Companion"
private const val PRESENCE_NOTIFICATION_ID = 1001

/**
 * Keeps the engine's realtime socket and the Messenger-style chat-head overlay alive while the person is away from
 * `MainActivity`. Restoring the session starts the engine, which the Activity shares.
 */
class NomiPresenceService : Service() {
    private val gate: SessionGate by inject()
    private val appScope: CoroutineScope by inject(named("app"))

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onCreate() {
        super.onCreate()
        ensureChannel()
        val notification = buildNotification()
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
            startForeground(PRESENCE_NOTIFICATION_ID, notification, ServiceInfo.FOREGROUND_SERVICE_TYPE_SPECIAL_USE)
        } else {
            startForeground(PRESENCE_NOTIFICATION_ID, notification)
        }
        appScope.launch { gate.restore() }
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int = START_STICKY

    private fun ensureChannel() {
        val manager = getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
        if (manager.getNotificationChannel(PRESENCE_CHANNEL_ID) == null) {
            val channel = NotificationChannel(
                PRESENCE_CHANNEL_ID,
                PRESENCE_CHANNEL_NAME,
                NotificationManager.IMPORTANCE_LOW,
            ).apply {
                description = "Keeps the Nomi chat head and live replies connected"
                setShowBadge(false)
            }
            manager.createNotificationChannel(channel)
        }
    }

    private fun buildNotification(): Notification {
        val openIntent = Intent(this, MainActivity::class.java).apply {
            flags = Intent.FLAG_ACTIVITY_SINGLE_TOP or Intent.FLAG_ACTIVITY_CLEAR_TOP
        }
        val pendingIntent = PendingIntent.getActivity(
            this,
            0,
            openIntent,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE,
        )
        return Notification.Builder(this, PRESENCE_CHANNEL_ID)
            .setContentTitle("Nomi is around")
            .setContentText("Tap to open Nomi")
            .setSmallIcon(android.R.drawable.sym_def_app_icon)
            .setContentIntent(pendingIntent)
            .setOngoing(true)
            .build()
    }

    companion object {
        fun startIfEligible(context: Context) {
            val intent = Intent(context, NomiPresenceService::class.java)
            runCatching { context.startForegroundService(intent) }
        }
    }
}
