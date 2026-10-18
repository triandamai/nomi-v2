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
        implementation(libs.datetime)
        implementation(libs.serialization.json)
        implementation(libs.markdown.renderer.m3)
    }
}
