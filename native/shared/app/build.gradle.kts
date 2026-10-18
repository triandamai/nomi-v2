plugins {
    alias(libs.plugins.nomi.kmp.compose)
    alias(libs.plugins.kotlin.serialization)
}

/** Name of the framework the iOS app links (`import Shared` in Swift). */
val iosFrameworkName = "Shared"

kotlin {
    listOf(iosArm64(), iosSimulatorArm64()).forEach { target ->
        target.binaries.framework {
            baseName = iosFrameworkName
            isStatic = true
        }
    }

    sourceSets {
        commonMain.dependencies {
            api(projects.shared.core)
            api(projects.shared.designsystem)
            implementation(projects.shared.network)
            implementation(projects.shared.realtime)
            implementation(projects.shared.database)
            implementation(projects.shared.data)
            implementation(projects.shared.feature.auth)
            implementation(projects.shared.feature.home)
            implementation(projects.shared.feature.chats)
            implementation(projects.shared.feature.chat)
            implementation(projects.shared.feature.notifications)
            implementation(projects.shared.feature.settings)
            implementation(projects.shared.companion)
            implementation(libs.compose.material3)
            implementation(libs.compose.adaptive)
            implementation(libs.navigation3.ui)
            implementation(libs.lifecycle.viewmodel.navigation3)
            implementation(libs.lifecycle.viewmodel.compose)
            implementation(libs.lifecycle.runtime.compose)
            implementation(libs.serialization.json)
            implementation(libs.coroutines.core)
            api(libs.koin.core)
            implementation(libs.koin.compose)
            implementation(libs.koin.compose.viewmodel)
        }
        commonTest.dependencies {
            implementation(libs.coroutines.test)
        }
        androidMain.dependencies {
            implementation(libs.koin.android)
        }
        val desktopTest by getting {
            dependencies {
                implementation(compose.desktop.currentOs)
                implementation(compose.desktop.uiTestJUnit4)
                implementation(libs.junit4)
                implementation(libs.koin.test)
                implementation(libs.ktor.client.mock)
                // The desktop app's Main dispatcher (desktopApp ships the same).
                implementation(libs.coroutines.swing)
            }
        }
    }
}
