plugins {
    alias(libs.plugins.nomi.kmp.compose)
    alias(libs.plugins.roborazzi)
}

kotlin {
    sourceSets.commonMain.dependencies {
        api(projects.shared.core)
        api(libs.compose.material3)
        api(compose.components.resources)
    }
    sourceSets.named("desktopTest").configure {
        dependencies {
            implementation(compose.desktop.currentOs)
            implementation(compose.desktop.uiTestJUnit4)
            implementation(libs.roborazzi.compose.desktop)
            implementation(libs.junit4)
        }
    }
}

compose.resources {
    packageOfResClass = "ai.nomi.designsystem.resources"
    publicResClass = true
}
