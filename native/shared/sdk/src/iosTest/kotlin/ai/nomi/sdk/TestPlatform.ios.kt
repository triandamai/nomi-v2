package ai.nomi.sdk

import platform.Foundation.NSTemporaryDirectory
import platform.Foundation.NSUUID

actual val hostCanLoadSdk: Boolean = true

actual fun newDataDir(): String = NSTemporaryDirectory() + "nomi-sdk-" + NSUUID().UUIDString + "/data"
