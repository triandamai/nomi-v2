plugins {
    `kotlin-dsl`
}

group = "ai.nomi.buildlogic"

kotlin {
    jvmToolchain(libs.versions.jvm.toolchain.get().toInt())
}

dependencies {
    // compileOnly: the root build puts these plugins on the classpath (apply false).
    compileOnly(libs.gradle.plugin.kotlin)
    compileOnly(libs.gradle.plugin.android)
    compileOnly(libs.gradle.plugin.compose)
    compileOnly(libs.gradle.plugin.compose.compiler)
}

gradlePlugin {
    plugins {
        register("kmpLibrary") {
            id = "nomi.kmp.library"
            implementationClass = "KmpLibraryConventionPlugin"
        }
        register("kmpCompose") {
            id = "nomi.kmp.compose"
            implementationClass = "KmpComposeConventionPlugin"
        }
    }
}
