import org.jetbrains.kotlin.gradle.plugin.mpp.KotlinNativeTarget

plugins {
    alias(libs.plugins.nomi.kmp.library)
}

// The Rust engine (rust/crates/nomi-sdk), built with cargo and bound with gobley-uniffi-bindgen.
// `-Pnomi.rust.release=true` builds it optimized; the default is a debug build.

/** gobley-uniffi-bindgen 0.3.7 generates bindings for UniFFI 0.29.4, the version nomi-sdk pins. */
val bindgenVersion = "0.3.7"
val rustCrate = "nomi-sdk"
val rustLibrary = "nomi_sdk"
val cinteropPackage = "$rustLibrary.cinterop"

val rustWorkspace: File = layout.settingsDirectory.dir("../rust").asFile.canonicalFile
val rustRelease = providers.gradleProperty("nomi.rust.release").map(String::toBoolean).getOrElse(false)
val rustProfile = if (rustRelease) "release" else "debug"
val rustTarget = rustWorkspace.resolve("target")
val bindgen = rustTarget.resolve("tools/bin/gobley-uniffi-bindgen")
val bindingsConfig = rustWorkspace.resolve("crates/$rustCrate/kotlin-bindings.toml")
val uniffiDir = layout.buildDirectory.dir("generated/uniffi/bindings")
val desktopResourcesDir = layout.buildDirectory.dir("generated/uniffi/desktopResources")

/** Cargo from PATH, or rustup's default home when Gradle runs without the shell's PATH (IDEs). */
val cargo: String = System.getenv("PATH").orEmpty().split(File.pathSeparator)
    .map { File(it, "cargo") }
    .firstOrNull { it.canExecute() }?.path
    ?: File(System.getProperty("user.home"), ".cargo/bin/cargo").path

/** The library JNA loads on this machine, and the classpath folder it looks for it in. */
val hostLibrary: File = rustTarget.resolve("$rustProfile/${System.mapLibraryName(rustLibrary)}")
val jnaResourcePrefix: String = run {
    val os = System.getProperty("os.name").lowercase()
    val arch = when (val raw = System.getProperty("os.arch").lowercase()) {
        "amd64", "x86_64" -> "x86-64"
        "arm64" -> "aarch64"
        else -> raw
    }
    when {
        os.startsWith("mac") -> "darwin-$arch"
        os.startsWith("windows") -> "win32-$arch"
        else -> "linux-$arch"
    }
}

/** Kotlin/Native targets and the Rust triples their static libraries are built for. */
val appleTriples = mapOf(
    "iosArm64" to "aarch64-apple-ios",
    "iosSimulatorArm64" to "aarch64-apple-ios-sim",
)

val rustSources = files(
    rustWorkspace.resolve("Cargo.toml"),
    rustWorkspace.resolve("Cargo.lock"),
    rustWorkspace.resolve("crates/$rustCrate/Cargo.toml"),
    rustWorkspace.resolve("crates/$rustCrate/src"),
)

fun Exec.cargoBuild(triple: String?) {
    group = "rust"
    workingDir = rustWorkspace
    commandLine(
        buildList {
            addAll(listOf(cargo, "build", "--package", rustCrate))
            if (rustRelease) add("--release")
            if (triple != null) addAll(listOf("--target", triple))
        },
    )
    inputs.files(rustSources)
}

val installBindgen = tasks.register<Exec>("installBindgen") {
    group = "rust"
    description = "Installs the pinned gobley-uniffi-bindgen into rust/target/tools."
    commandLine(
        cargo, "install", "gobley-uniffi-bindgen",
        "--version", bindgenVersion, "--locked", "--root", rustTarget.resolve("tools").path,
    )
    inputs.property("version", bindgenVersion)
    outputs.file(bindgen)
}

val cargoBuildHost = tasks.register<Exec>("cargoBuildHost") {
    description = "Builds $rustCrate for this machine (desktop JVM, and the source of the bindings)."
    cargoBuild(triple = null)
    outputs.file(hostLibrary)
}

val generateUniffiBindings = tasks.register<Exec>("generateUniffiBindings") {
    group = "rust"
    description = "Generates the Kotlin Multiplatform bindings and the cinterop header."
    dependsOn(installBindgen, cargoBuildHost)
    val outDir = uniffiDir.get().asFile
    commandLine(
        bindgen.path, "--library", hostLibrary.path, "--crate", rustLibrary,
        "--config", bindingsConfig.path, "--out-dir", outDir.path,
    )
    inputs.file(hostLibrary)
    inputs.file(bindingsConfig)
    outputs.dir(outDir)
}

val stageDesktopLibrary = tasks.register<Copy>("stageDesktopLibrary") {
    group = "rust"
    description = "Puts the host library where JNA looks for it on the desktop classpath."
    dependsOn(cargoBuildHost)
    from(hostLibrary)
    into(desktopResourcesDir.map { it.dir(jnaResourcePrefix) })
}

fun generated(sourceSet: String) = generateUniffiBindings.map { uniffiDir.get().dir("$sourceSet/kotlin") }

kotlin {
    targets.withType<KotlinNativeTarget>().configureEach {
        val triple = appleTriples.getValue(name)
        val libraryDir = rustTarget.resolve("$triple/$rustProfile")
        val cargoBuildTarget = tasks.register<Exec>("cargoBuild${name.replaceFirstChar(Char::uppercase)}") {
            description = "Builds $rustCrate as a static library for $triple."
            cargoBuild(triple)
            outputs.file(libraryDir.resolve("lib$rustLibrary.a"))
        }
        compilations.getByName("main").cinterops.create(rustLibrary) {
            packageName(cinteropPackage)
            headers(uniffiDir.map { it.file("nativeInterop/cinterop/headers/$rustLibrary/$rustLibrary.h") })
            // Embeds the static library in the cinterop klib, so every binary that links it gets Rust too.
            extraOpts("-libraryPath", libraryDir.path, "-staticLibrary", "lib$rustLibrary.a")
            tasks.named(interopProcessingTaskName).configure { dependsOn(generateUniffiBindings, cargoBuildTarget) }
        }
    }

    sourceSets {
        commonMain {
            kotlin.srcDir(generated("commonMain"))
            dependencies {
                implementation(libs.coroutines.core)
                implementation(libs.atomicfu)
            }
        }
        commonTest.dependencies {
            implementation(libs.coroutines.test)
        }
        androidMain {
            kotlin.srcDir(generated("androidMain"))
            dependencies {
                implementation("net.java.dev.jna:jna:${libs.versions.jna.get()}@aar")
                implementation(libs.androidx.annotation)
            }
        }
        nativeMain {
            kotlin.srcDir(generated("nativeMain"))
        }
        getByName("desktopMain") {
            kotlin.srcDir(generated("jvmMain"))
            resources.srcDir(stageDesktopLibrary.map { desktopResourcesDir.get() })
            dependencies {
                implementation(libs.jna)
            }
        }
    }
}
