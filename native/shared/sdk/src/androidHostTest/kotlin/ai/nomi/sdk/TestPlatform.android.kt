package ai.nomi.sdk

import java.nio.file.Files

/** Host tests run on the build machine's JVM with Android's JNA bindings and no `.so` packaged. */
actual val hostCanLoadSdk: Boolean = false

actual fun newDataDir(): String = Files.createTempDirectory("nomi-sdk").resolve("data").toString()
