plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.compose.compiler)
}

/** The emulator reaches the host's backend (backend/crates/nomi-server listens on 8080) at 10.0.2.2. */
val emulatorBackendUrl = "http://10.0.2.2:8080"

/** Release builds must name an https backend; there is no safe default to ship. */
val releaseApiUrl: String? = providers.gradleProperty("nomi.apiUrl.release").orNull?.takeIf { it.isNotBlank() }

/** Task names that build the release variant (explicitly, or as part of `assemble` / `build` / `bundle`). */
val releaseTaskRequested = gradle.startParameter.taskNames.any { requested ->
    val name = requested.substringAfterLast(':')
    name.contains("Release", ignoreCase = true) || name in setOf("assemble", "build", "bundle")
}
if (releaseTaskRequested && releaseApiUrl == null) {
    throw GradleException("Release builds need the backend URL: pass -Pnomi.apiUrl.release=https://…")
}

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
        // Checked above: a release task without -Pnomi.apiUrl.release fails at configuration time.
        release {
            buildConfigField("String", "NOMI_API_URL", "\"${releaseApiUrl.orEmpty()}\"")
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
