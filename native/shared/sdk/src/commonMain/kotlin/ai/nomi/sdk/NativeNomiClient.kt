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

    private val mutableAccount = MutableStateFlow(sdk.currentAccount())

    override val state: StateFlow<SdkState> = mutableState.asStateFlow()

    override val account: StateFlow<AccountStore?> = mutableAccount.asStateFlow()

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

    override suspend fun openAccount(accountId: String): AccountStore = accountChange { sdk.openAccount(accountId) }

    override suspend fun closeAccount() = accountChange { sdk.closeAccount() }

    override suspend fun removeAccount(accountId: String) = accountChange { sdk.removeAccount(accountId) }

    override suspend fun resetStorage() = accountChange { sdk.resetStorage() }

    override suspend fun close() {
        try {
            sdk.shutdown()
        } finally {
            mutableState.value = sdk.state()
            mutableAccount.value = sdk.currentAccount()
        }
    }

    private suspend fun <T> accountChange(change: suspend () -> T): T = try {
        change()
    } finally {
        mutableAccount.value = sdk.currentAccount()
    }
}
