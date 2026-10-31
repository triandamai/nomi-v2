package ai.nomi.sdk

/** Whether this test target ships the Rust library (Android host tests don't, yet). */
expect val hostCanLoadSdk: Boolean

/** A fresh, absolute, not-yet-created directory for one SDK instance. */
expect fun newDataDir(): String
