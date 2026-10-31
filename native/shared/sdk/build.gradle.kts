import java.util.Properties
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

/** Executables carry `.exe` on Windows. */
val exe = if (System.getProperty("os.name").startsWith("Windows")) ".exe" else ""

val rustWorkspace: File = layout.settingsDirectory.dir("../rust").asFile.canonicalFile
val rustRelease = providers.gradleProperty("nomi.rust.release").map(String::toBoolean).getOrElse(false)
val rustProfile = if (rustRelease) "release" else "debug"
val rustTarget = rustWorkspace.resolve("target")
val bindgen = rustTarget.resolve("tools/bin/gobley-uniffi-bindgen$exe")
val bindingsConfig = rustWorkspace.resolve("crates/$rustCrate/kotlin-bindings.toml")
val uniffiDir = layout.buildDirectory.dir("generated/uniffi/bindings")
val desktopResourcesDir = layout.buildDirectory.dir("generated/uniffi/desktopResources")

/** Cargo from PATH, or rustup's default home when Gradle runs without the shell's PATH (IDEs). */
val cargo: String = System.getenv("PATH").orEmpty().split(File.pathSeparator)
    .map { File(it, "cargo$exe") }
    .firstOrNull { it.canExecute() }?.path
    ?: File(System.getProperty("user.home"), ".cargo/bin/cargo$exe").path

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

/**
 * The oldest iOS any binary linking the SDK targets: Kotlin/Native's own test binaries (the app
 * itself targets 17). C code in the static library (SQLite, ring) is compiled for it.
 */
val iosDeploymentTarget = "15.0"

/** cargo-ndk builds the Android libraries with the NDK's toolchains. */
val cargoNdkVersion = "4.1.2"
val cargoNdk = rustTarget.resolve("tools/bin/cargo-ndk$exe")
/**
 * The Android ABIs built: phones (arm64, and 32-bit arm for older ones) and the emulator (x86_64).
 * The app packages only these (`abiFilters`), so no device installs it without the engine.
 */
val androidAbis = listOf("arm64-v8a", "armeabi-v7a", "x86_64")
val androidMinSdk = libs.versions.android.minSdk.get()
/** Where the Android libraries land, as `<abi>/libnomi_sdk.so`; the app packages this folder. */
val androidJniLibsDir = layout.buildDirectory.dir("generated/uniffi/jniLibs")

/**
 * The NDK: `ANDROID_NDK_HOME` when set, else `<sdk>/ndk/<nomi.android.ndkVersion>`, else the newest
 * NDK installed in the SDK (`ANDROID_HOME`, or `sdk.dir` in local.properties).
 */
val androidNdk: File? = run {
    System.getenv("ANDROID_NDK_HOME")?.takeIf { it.isNotBlank() }?.let { return@run File(it) }
    val localProperties = Properties().apply {
        layout.settingsDirectory.file("local.properties").asFile.takeIf { it.isFile }?.reader()?.use { load(it) }
    }
    val sdk = (System.getenv("ANDROID_HOME")?.takeIf { it.isNotBlank() } ?: localProperties.getProperty("sdk.dir"))
        ?.let(::File) ?: return@run null
    val pinned = providers.gradleProperty("nomi.android.ndkVersion").orNull
    if (pinned != null) return@run sdk.resolve("ndk/$pinned")
    sdk.resolve("ndk").listFiles()?.filter { it.isDirectory }
        ?.maxByOrNull { dir -> dir.name.split('.').map { it.toIntOrNull() ?: 0 }.fold(0L) { acc, part -> acc * 100_000 + part } }
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
    if (triple != null && triple in appleTriples.values) environment("IPHONEOS_DEPLOYMENT_TARGET", iosDeploymentTarget)
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

val installCargoNdk = tasks.register<Exec>("installCargoNdk") {
    group = "rust"
    description = "Installs the pinned cargo-ndk into rust/target/tools."
    commandLine(
        cargo, "install", "cargo-ndk",
        "--version", cargoNdkVersion, "--locked", "--root", rustTarget.resolve("tools").path,
    )
    inputs.property("version", cargoNdkVersion)
    outputs.file(cargoNdk)
}

val cargoBuildAndroid = tasks.register<Exec>("cargoBuildAndroid") {
    group = "rust"
    description = "Builds $rustCrate for ${androidAbis.joinToString()} into build/generated/uniffi/jniLibs."
    dependsOn(installCargoNdk)
    val outDir = androidJniLibsDir.get().asFile
    val ndk = requireNotNull(androidNdk?.takeIf { it.isDirectory }) {
        "No Android NDK: set ANDROID_NDK_HOME, or install one with the SDK manager"
    }.path
    workingDir = rustWorkspace
    // `cargo ndk` (cargo finds cargo-ndk on the PATH): the plugin refuses to run on its own.
    commandLine(
        buildList {
            addAll(listOf(cargo, "ndk", "--platform", androidMinSdk, "--output-dir", outDir.path))
            androidAbis.forEach { addAll(listOf("--target", it)) }
            addAll(listOf("build", "--package", rustCrate))
            if (rustRelease) add("--release")
        },
    )
    val path = listOf(cargoNdk.parent, File(cargo).parent, System.getenv("PATH").orEmpty())
    environment("PATH", path.joinToString(File.pathSeparator))
    environment("ANDROID_NDK_HOME", ndk)
    inputs.files(rustSources)
    inputs.property("abis", androidAbis)
    outputs.dir(outDir)
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
            // The frameworks the static library needs (see the def file), linked into every consumer.
            definitionFile.set(project.file("src/nativeInterop/cinterop/$rustLibrary.def"))
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
