package ai.nomi.app

import ai.nomi.app.di.appModules
import ai.nomi.data.SessionGate
import ai.nomi.network.TokenStore
import ai.nomi.network.Tokens
import kotlinx.coroutines.runBlocking
import org.koin.core.context.startKoin
import org.koin.core.context.stopKoin
import org.koin.dsl.module
import kotlin.test.AfterTest
import kotlin.test.Test
import kotlin.test.assertFalse
import kotlin.test.assertTrue

/** A keychain / keystore that cannot be read (locked device, corrupt entry). */
private class UnreadableTokenStore : TokenStore {
    override suspend fun read(): Tokens? = error("token store unreadable")
    override suspend fun write(tokens: Tokens) = Unit
    override suspend fun clear() = Unit
}

class RestoreTest {
    @AfterTest
    fun tearDown() = stopKoin()

    @Test
    fun unreadable_token_store_falls_back_to_sign_in() {
        val koin = startKoin {
            modules(appModules(testPlatformModule) + module { single<TokenStore> { UnreadableTokenStore() } })
        }.koin
        val gate = koin.get<SessionGate>()

        runBlocking { gate.restore() }

        assertTrue(gate.restored.value)
        assertFalse(gate.signedIn.value)
    }
}
