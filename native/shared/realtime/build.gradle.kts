plugins {
    alias(libs.plugins.nomi.kmp.library)
    alias(libs.plugins.kotlin.serialization)
}

kotlin {
    sourceSets {
        commonMain.dependencies {
            api(projects.shared.core)
            api(projects.shared.network)
            api(libs.serialization.json)
            api(libs.coroutines.core)
            implementation(libs.ktor.client.websockets)
        }
        commonTest.dependencies {
            implementation(libs.coroutines.test)
        }
        val desktopTest by getting {
            dependencies {
                implementation(libs.ktor.client.okhttp)
            }
        }
    }
}
