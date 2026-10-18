plugins {
    alias(libs.plugins.nomi.kmp.compose)
    alias(libs.plugins.roborazzi)
}

kotlin {
    sourceSets {
        commonMain.dependencies {
            implementation(projects.shared.core)
            implementation(projects.shared.designsystem)
            implementation(projects.shared.data)
            implementation(libs.compose.material3)
            implementation(libs.coroutines.core)
        }
        commonTest.dependencies {
            implementation(libs.coroutines.test)
            implementation(libs.turbine)
        }
        val desktopTest by getting {
            dependencies {
                implementation(compose.desktop.currentOs)
                implementation(compose.desktop.uiTestJUnit4)
                implementation(libs.junit4)
                implementation(libs.roborazzi.compose.desktop)
                implementation(libs.coroutines.swing)
            }
        }
    }
}
