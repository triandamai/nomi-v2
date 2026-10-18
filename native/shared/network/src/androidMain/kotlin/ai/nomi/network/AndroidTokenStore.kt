package ai.nomi.network

import android.content.Context
import android.util.Base64
import androidx.datastore.preferences.core.edit
import androidx.datastore.preferences.core.stringPreferencesKey
import androidx.datastore.preferences.preferencesDataStore
import com.google.crypto.tink.Aead
import com.google.crypto.tink.KeyTemplates
import com.google.crypto.tink.RegistryConfiguration
import com.google.crypto.tink.aead.AeadConfig
import com.google.crypto.tink.integration.android.AndroidKeysetManager
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.withContext
import kotlinx.serialization.json.Json

private const val DATASTORE_NAME = "nomi_tokens"
private const val KEYSET_NAME = "nomi_tokens_keyset"
private const val KEYSET_PREFS = "nomi_tokens_keyset_prefs"
private const val MASTER_KEY_URI = "android-keystore://nomi_tokens_master"
private const val AEAD_TEMPLATE = "AES256_GCM"
private val ASSOCIATED_DATA = TOKEN_SERVICE.encodeToByteArray()
private val SESSION_KEY = stringPreferencesKey("session")

private val Context.tokenDataStore by preferencesDataStore(DATASTORE_NAME)

/**
 * Tokens encrypted with a Tink AES-256-GCM key whose keyset is wrapped by the Android Keystore, kept in DataStore.
 *
 * The Keystore master key never leaves the device, so the app MUST exclude `datastore/nomi_tokens.preferences_pb`
 * and the `nomi_tokens_keyset_prefs` shared prefs from Auto Backup / device transfer (backup rules); a restored
 * copy could not be decrypted and would only leave a dead session.
 */
class AndroidTokenStore(context: Context) : TokenStore {
    private val appContext = context.applicationContext

    private val aead: Aead by lazy {
        AeadConfig.register()
        AndroidKeysetManager.Builder()
            .withSharedPref(appContext, KEYSET_NAME, KEYSET_PREFS)
            .withKeyTemplate(KeyTemplates.get(AEAD_TEMPLATE))
            .withMasterKeyUri(MASTER_KEY_URI)
            .build()
            .keysetHandle
            .getPrimitive(RegistryConfiguration.get(), Aead::class.java)
    }

    override suspend fun read(): Tokens? = withContext(Dispatchers.IO) {
        val encoded = appContext.tokenDataStore.data.first()[SESSION_KEY] ?: return@withContext null
        runCatching {
            val plain = aead.decrypt(Base64.decode(encoded, Base64.NO_WRAP), ASSOCIATED_DATA)
            Json.decodeFromString(Tokens.serializer(), plain.decodeToString())
        }.getOrNull()
    }

    override suspend fun write(tokens: Tokens) = withContext(Dispatchers.IO) {
        val plain = Json.encodeToString(Tokens.serializer(), tokens).encodeToByteArray()
        val encoded = Base64.encodeToString(aead.encrypt(plain, ASSOCIATED_DATA), Base64.NO_WRAP)
        appContext.tokenDataStore.edit { it[SESSION_KEY] = encoded }
        Unit
    }

    override suspend fun clear() = withContext(Dispatchers.IO) {
        appContext.tokenDataStore.edit { it.remove(SESSION_KEY) }
        Unit
    }
}
