plugins {
    alias(libs.plugins.nomi.kmp.compose)
}

kotlin {
    sourceSets.commonMain.dependencies {
        api(projects.shared.core)
    }
}
