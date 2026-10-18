plugins {
    alias(libs.plugins.nomi.kmp.compose)
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

    sourceSets.commonMain.dependencies {
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
    }
}
