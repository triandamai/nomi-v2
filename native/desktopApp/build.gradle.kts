import org.jetbrains.compose.desktop.application.dsl.TargetFormat

plugins {
    alias(libs.plugins.kotlin.jvm)
    alias(libs.plugins.compose.multiplatform)
    alias(libs.plugins.compose.compiler)
}

kotlin {
    jvmToolchain(libs.versions.jvm.toolchain.get().toInt())
}

dependencies {
    implementation(projects.shared.app)
    implementation(compose.desktop.currentOs)
    implementation(libs.coroutines.swing)
    implementation(libs.jna.platform)
    testImplementation(kotlin("test"))
    testImplementation(libs.junit4)
    testImplementation(libs.coroutines.test)
}

compose.desktop {
    application {
        mainClass = "ai.nomi.desktop.MainKt"

        nativeDistributions {
            targetFormats(TargetFormat.Dmg, TargetFormat.Msi, TargetFormat.Deb)
            packageName = "Nomi"
            packageVersion = providers.gradleProperty("nomi.versionName").get()

            macOS {
                bundleID = "ai.nomi.app"
                infoPlist {
                    extraKeysRawXml = "<key>LSUIElement</key><true/>"
                }
            }
        }
    }
}
