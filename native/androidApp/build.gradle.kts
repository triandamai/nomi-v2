plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.compose.compiler)
}

/** The emulator reaches the host's backend (backend/crates/nomi-server listens on 8080) at 10.0.2.2. */
val emulatorBackendUrl = "http://10.0.2.2:8080"

android {
    namespace = "ai.nomi.app.android"
    compileSdk = libs.versions.android.compileSdk.get().toInt()

    defaultConfig {
        applicationId = "ai.nomi.app"
        minSdk = libs.versions.android.minSdk.get().toInt()
        targetSdk = libs.versions.android.targetSdk.get().toInt()
        versionCode = providers.gradleProperty("nomi.versionCode").get().toInt()
        versionName = providers.gradleProperty("nomi.versionName").get()
    }

    buildTypes {
        debug {
            buildConfigField("String", "NOMI_API_URL", "\"${providers.gradleProperty("nomi.apiUrl.debug").getOrElse(emulatorBackendUrl)}\"")
        }
        // Release builds must point at an https backend: -Pnomi.apiUrl.release=https://…
        release {
            buildConfigField("String", "NOMI_API_URL", "\"${providers.gradleProperty("nomi.apiUrl.release").getOrElse("")}\"")
        }
    }

    buildFeatures {
        compose = true
        buildConfig = true
    }

    compileOptions {
        val javaVersion = JavaVersion.toVersion(libs.versions.jvm.toolchain.get())
        sourceCompatibility = javaVersion
        targetCompatibility = javaVersion
    }
}

kotlin {
    jvmToolchain(libs.versions.jvm.toolchain.get().toInt())
}

dependencies {
    implementation(projects.shared.app)
    implementation(libs.androidx.activity.compose)
    implementation(libs.koin.android)
}
