@file:OptIn(ExperimentalForeignApi::class, BetaInteropApi::class)

package ai.nomi.network

import kotlinx.cinterop.BetaInteropApi
import kotlinx.cinterop.ExperimentalForeignApi
import kotlinx.cinterop.alloc
import kotlinx.cinterop.memScoped
import kotlinx.cinterop.ptr
import kotlinx.cinterop.value
import kotlinx.serialization.json.Json
import platform.CoreFoundation.CFDictionaryAddValue
import platform.CoreFoundation.CFDictionaryCreateMutable
import platform.CoreFoundation.CFMutableDictionaryRef
import platform.CoreFoundation.CFRelease
import platform.CoreFoundation.CFTypeRefVar
import platform.CoreFoundation.kCFBooleanTrue
import platform.CoreFoundation.kCFTypeDictionaryKeyCallBacks
import platform.CoreFoundation.kCFTypeDictionaryValueCallBacks
import platform.Foundation.CFBridgingRelease
import platform.Foundation.CFBridgingRetain
import platform.Foundation.NSData
import platform.Foundation.NSString
import platform.Foundation.NSUTF8StringEncoding
import platform.Foundation.create
import platform.Foundation.dataUsingEncoding
import platform.Security.SecItemAdd
import platform.Security.SecItemCopyMatching
import platform.Security.SecItemDelete
import platform.Security.errSecSuccess
import platform.Security.kSecAttrAccessible
import platform.Security.kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly
import platform.Security.kSecAttrAccount
import platform.Security.kSecAttrService
import platform.Security.kSecClass
import platform.Security.kSecClassGenericPassword
import platform.Security.kSecMatchLimit
import platform.Security.kSecMatchLimitOne
import platform.Security.kSecReturnData
import platform.Security.kSecValueData

private const val KEYCHAIN_ACCOUNT = "session"

/** Tokens in the iOS Keychain (`kSecClassGenericPassword`, service `ai.nomi.app.tokens`), device-local. */
class IosTokenStore : TokenStore {
    override suspend fun read(): Tokens? = memScoped {
        val result = alloc<CFTypeRefVar>()
        val status = withQuery(returnData = true) { SecItemCopyMatching(it, result.ptr) }
        if (status != errSecSuccess) return@memScoped null
        val data = CFBridgingRelease(result.value) as? NSData ?: return@memScoped null
        val text = NSString.create(data = data, encoding = NSUTF8StringEncoding)?.toString() ?: return@memScoped null
        runCatching { Json.decodeFromString(Tokens.serializer(), text) }.getOrNull()
    }

    override suspend fun write(tokens: Tokens) {
        clear()
        val payload = Json.encodeToString(Tokens.serializer(), tokens)
        val data = NSString.create(string = payload).dataUsingEncoding(NSUTF8StringEncoding) ?: return
        withQuery(returnData = false) { query ->
            val retainedData = CFBridgingRetain(data)
            CFDictionaryAddValue(query, kSecValueData, retainedData)
            CFDictionaryAddValue(query, kSecAttrAccessible, kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly)
            SecItemAdd(query, null).also { CFRelease(retainedData) }
        }
    }

    override suspend fun clear() {
        withQuery(returnData = false) { SecItemDelete(it) }
    }

    private inline fun <T> withQuery(returnData: Boolean, block: (CFMutableDictionaryRef) -> T): T {
        val dict = CFDictionaryCreateMutable(null, 0, kCFTypeDictionaryKeyCallBacks.ptr, kCFTypeDictionaryValueCallBacks.ptr)!!
        val service = CFBridgingRetain(TOKEN_SERVICE)
        val account = CFBridgingRetain(KEYCHAIN_ACCOUNT)
        try {
            CFDictionaryAddValue(dict, kSecClass, kSecClassGenericPassword)
            CFDictionaryAddValue(dict, kSecAttrService, service)
            CFDictionaryAddValue(dict, kSecAttrAccount, account)
            if (returnData) {
                CFDictionaryAddValue(dict, kSecReturnData, kCFBooleanTrue)
                CFDictionaryAddValue(dict, kSecMatchLimit, kSecMatchLimitOne)
            }
            return block(dict)
        } finally {
            CFRelease(service)
            CFRelease(account)
            CFRelease(dict)
        }
    }
}
