package ai.nomi.app.android

import ai.nomi.app.androidPlatformModule
import ai.nomi.app.di.AppScope
import ai.nomi.app.di.initKoin
import android.app.Application
import org.koin.android.ext.koin.androidContext
import org.koin.dsl.module

class NomiApplication : Application() {
    override fun onCreate() {
        super.onCreate()
        val overlayModule = module {
            single(createdAtStart = true) {
                BubbleOverlayController(
                    context = this@NomiApplication,
                    scope = get(AppScope),
                    gate = get(),
                    companion = get(),
                    appearance = get(),
                )
            }
        }
        initKoin(androidPlatformModule(BuildConfig.NOMI_API_URL)) {
            androidContext(this@NomiApplication)
            modules(overlayModule)
        }
    }
}
