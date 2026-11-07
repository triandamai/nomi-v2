plugins {
    alias(libs.plugins.nomi.kmp.library)
    alias(libs.plugins.kotlin.serialization)
}

/** `nomi.apiUrl` from native/gradle.properties, compiled into the shared code as [HOSTED_API_URL]. */
val hostedApiUrl: Provider<String> = providers.gradleProperty("nomi.apiUrl").map { it.trim() }
val generateHostedApi = tasks.register("generateHostedApi") {
    val url = hostedApiUrl
    val outDir = layout.buildDirectory.dir("generated/hostedApi/commonMain/kotlin")
    inputs.property("url", url)
    outputs.dir(outDir)
    doLast {
        val value = url.get()
        require(value.startsWith("https://") || value.startsWith("http://")) { "nomi.apiUrl must be an http(s) URL, got \"$value\"" }
        val file = outDir.get().file("ai/nomi/network/HostedApi.kt").asFile
        file.parentFile.mkdirs()
        file.writeText(
            "// Generated from nomi.apiUrl in native/gradle.properties; edit it there.\n" +
                "package ai.nomi.network\n\n" +
                "/** The backend the apps use unless a build overrides it. */\n" +
                "const val HOSTED_API_URL = \"$value\"\n",
        )
    }
}

kotlin {
    sourceSets {
        commonMain {
            kotlin.srcDir(generateHostedApi)
        }
        commonMain.dependencies {
            api(projects.shared.core)
            api(libs.serialization.json)
            implementation(libs.coroutines.core)
        }
        commonTest.dependencies {
            implementation(libs.coroutines.test)
        }
        androidMain.dependencies {
            implementation(libs.tink.android)
            implementation(libs.datastore.preferences)
        }
        val desktopMain by getting {
            dependencies {
                implementation(libs.java.keyring)
            }
        }
        val desktopTest by getting {
            dependencies {
                implementation(kotlin("test"))
            }
        }
    }
}
