package ai.nomi.sdk

import java.nio.file.Files

actual val hostCanLoadSdk: Boolean = true

actual fun newDataDir(): String = Files.createTempDirectory("nomi-sdk").resolve("data").toString()
