plugins {
    alias(libs.plugins.nomi.kmp.library)
    alias(libs.plugins.kotlin.serialization)
}

kotlin {
    sourceSets.commonMain.dependencies {
        api(projects.shared.core)
        api(projects.shared.network)
        api(libs.serialization.json)
    }
}
