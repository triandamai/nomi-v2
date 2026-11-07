plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.compose.compiler)
}

/**
 * Every build talks to `nomi.apiUrl` (native/gradle.properties), the backend all the apps share. For a local backend
 * (backend/crates/nomi-server listens on 8080), pass `-Pnomi.apiUrl.debug=http://10.0.2.2:8080` (the emulator's address
 * for the host). `-Pnomi.apiUrl.release=https://…` overrides it for one release build.
 */
val hostedBackendUrl: String = providers.gradleProperty("nomi.apiUrl").get().trim()

/** Release builds must talk to an https backend. */
val releaseApiUrl: String = providers.gradleProperty("nomi.apiUrl.release").orNull?.takeIf { it.isNotBlank() }?.trim() ?: hostedBackendUrl

/** Task names that build the release variant (explicitly, or as part of `assemble` / `build` / `bundle`). */
val releaseTaskRequested = gradle.startParameter.taskNames.any { requested ->
    val name = requested.substringAfterLast(':')
    name.contains("Release", ignoreCase = true) || name in setOf("assemble", "build", "bundle")
}
if (releaseTaskRequested && !releaseApiUrl.startsWith("https://")) {
    throw GradleException("Release builds need an https backend: set nomi.apiUrl in native/gradle.properties (or pass -Pnomi.apiUrl.release=https://…)")
}

/** The Android libraries :shared:sdk builds; JNA loads `libnomi_sdk.so` from the APK. */
val rustJniLibs = rootDir.resolve("shared/sdk/build/generated/uniffi/jniLibs")

android {
    namespace = "ai.nomi.app.android"
    compileSdk = libs.versions.android.compileSdk.get().toInt()

    defaultConfig {
        applicationId = "ai.nomi.app"
        minSdk = libs.versions.android.minSdk.get().toInt()
        targetSdk = libs.versions.android.targetSdk.get().toInt()
        versionCode = providers.gradleProperty("nomi.versionCode").get().toInt()
        versionName = providers.gradleProperty("nomi.versionName").get()
        // The ABIs :shared:sdk builds the Rust engine for; a device without one can't run the app.
        ndk { abiFilters += listOf("arm64-v8a", "armeabi-v7a", "x86_64") }
    }

    buildTypes {
        debug {
            buildConfigField("String", "NOMI_API_URL", "\"${providers.gradleProperty("nomi.apiUrl.debug").getOrElse(hostedBackendUrl)}\"")
        }
        // Checked above: a release task without an https backend fails at configuration time.
        release {
            buildConfigField("String", "NOMI_API_URL", "\"$releaseApiUrl\"")
        }
    }

    buildFeatures {
        compose = true
        buildConfig = true
    }

    // The Rust engine (rust/crates/nomi-sdk), built for each ABI by :shared:sdk:cargoBuildAndroid.
    sourceSets.getByName("main").jniLibs.directories.add(rustJniLibs.path)

    compileOptions {
        val javaVersion = JavaVersion.toVersion(libs.versions.jvm.toolchain.get())
        sourceCompatibility = javaVersion
        targetCompatibility = javaVersion
    }
}

tasks.matching { it.name.startsWith("merge") && it.name.endsWith("JniLibFolders") }.configureEach {
    dependsOn(":shared:sdk:cargoBuildAndroid")
}

kotlin {
    jvmToolchain(libs.versions.jvm.toolchain.get().toInt())
}

dependencies {
    implementation(projects.shared.app)
    implementation(libs.androidx.activity.compose)
    implementation(libs.lifecycle.runtime.compose)
    implementation(libs.lifecycle.viewmodel.compose)
    implementation(libs.koin.android)
}
