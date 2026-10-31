package ai.nomi.sdk

import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.flow

/**
 * [NomiClient] backed by the Rust engine through UniFFI.
 *
 * Constructing it initializes the engine: it throws [NomiException.InvalidConfig] for a config the
 * engine rejects and [NomiException.Storage] when the data directory can't be created.
 */
class NativeNomiClient(config: SdkConfig) : NomiClient {
    private val sdk = NomiSdk(config)
    private val mutableState = MutableStateFlow(sdk.state())

    override val state: StateFlow<SdkState> = mutableState.asStateFlow()

    override val events: Flow<SdkEvent> = flow {
        val subscription = sdk.subscribe()
        try {
            while (true) {
                emit(subscription.nextEvent() ?: break)
            }
        } finally {
            subscription.cancel()
            subscription.destroy()
        }
    }

    override fun diagnostics(): Diagnostics = sdk.diagnostics()

    override suspend fun start() {
        try {
            sdk.start()
        } finally {
            mutableState.value = sdk.state()
        }
    }

    override suspend fun close() {
        sdk.shutdown()
        mutableState.value = sdk.state()
    }
}
