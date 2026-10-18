plugins {
    alias(libs.plugins.nomi.kmp.compose)
}

kotlin {
    sourceSets.commonMain.dependencies {
        implementation(projects.shared.core)
        implementation(projects.shared.designsystem)
        implementation(projects.shared.data)
        implementation(libs.lifecycle.viewmodel.compose)
        implementation(libs.lifecycle.runtime.compose)
        implementation(libs.coroutines.core)
    }
}
