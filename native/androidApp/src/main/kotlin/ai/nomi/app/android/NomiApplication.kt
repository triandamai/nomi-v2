package ai.nomi.app.android

import ai.nomi.app.androidPlatformModule
import ai.nomi.app.di.initKoin
import android.app.Application
import org.koin.android.ext.koin.androidContext

class NomiApplication : Application() {
    override fun onCreate() {
        super.onCreate()
        initKoin(androidPlatformModule(BuildConfig.NOMI_API_URL)) { androidContext(this@NomiApplication) }
    }
}
