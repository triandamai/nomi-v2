plugins {
    alias(libs.plugins.nomi.kmp.library)
}

kotlin {
    sourceSets {
        commonMain.dependencies {
            api(projects.shared.core)
            api(projects.shared.network)
            api(projects.shared.sdk)
            implementation(libs.coroutines.core)
        }
        commonTest.dependencies {
            implementation(libs.coroutines.test)
            implementation(libs.turbine)
        }
    }
}
