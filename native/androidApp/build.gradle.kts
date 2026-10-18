plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.compose.compiler)
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

    buildFeatures {
        compose = true
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
}
