plugins {
    alias(libs.plugins.nomi.kmp.library)
}

kotlin {
    sourceSets.commonMain.dependencies {
        api(projects.shared.core)
    }
}
