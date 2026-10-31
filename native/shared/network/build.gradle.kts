plugins {
    alias(libs.plugins.nomi.kmp.library)
    alias(libs.plugins.kotlin.serialization)
}

kotlin {
    sourceSets {
        commonMain.dependencies {
            api(projects.shared.core)
            api(libs.serialization.json)
            implementation(libs.coroutines.core)
        }
        commonTest.dependencies {
            implementation(libs.coroutines.test)
        }
        androidMain.dependencies {
            implementation(libs.tink.android)
            implementation(libs.datastore.preferences)
        }
        val desktopMain by getting {
            dependencies {
                implementation(libs.java.keyring)
            }
        }
        val desktopTest by getting {
            dependencies {
                implementation(kotlin("test"))
            }
        }
    }
}
