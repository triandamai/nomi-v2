package ai.nomi.data

import ai.nomi.network.SignInResult
import ai.nomi.network.TokenStore
import ai.nomi.network.Tokens
import ai.nomi.network.VerificationInfo
import ai.nomi.sdk.CredentialStore
import ai.nomi.sdk.Credentials
import ai.nomi.sdk.NativeNomiClient
import ai.nomi.sdk.defaultSdkConfig
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.MutableSharedFlow
import java.nio.file.Files

/** A port nothing listens on: the engine's requests are refused at once. */
const val UNREACHABLE_BASE_URL = "http://127.0.0.1:9"

/** A signed-in session's scope, already open, child of this scope. */
fun CoroutineScope.openSession(): SessionScope = SessionScope(this).also { it.open() }

/** An unsigned JWT-shaped access token naming [userId] in `sub`, as the backend's do. */
fun accessToken(userId: String): String {
    val encoder = java.util.Base64.getUrlEncoder().withoutPadding()
    val encode = { json: String -> encoder.encodeToString(json.encodeToByteArray()) }
    return "${encode("""{"alg":"none"}""")}.${encode("""{"sub":"$userId"}""")}.signature"
}

class MemoryCredentials(var stored: Credentials? = null) : CredentialStore {
    override fun load(): Credentials? = stored

    override fun save(credentials: Credentials) {
        stored = credentials
    }

    override fun clear() {
        stored = null
    }
}

/** A real engine on a fresh data directory, against a backend that can't be reached. */
fun CoroutineScope.testEngine(
    credentials: CredentialStore = MemoryCredentials(),
    foreground: ForegroundChat = ForegroundChat(),
): NomiEngine {
    val dataDir = Files.createTempDirectory("nomi-data-test").toFile().apply { deleteOnExit() }
    return NomiEngine(this, foreground) {
        NativeNomiClient(defaultSdkConfig(UNREACHABLE_BASE_URL, dataDir.path), credentials)
    }
}

class FakeTokenStore(var tokens: Tokens? = null) : TokenStore {
    override suspend fun read(): Tokens? = tokens

    override suspend fun write(tokens: Tokens) {
        this.tokens = tokens
    }

    override suspend fun clear() {
        tokens = null
    }
}

/** Scripted [AuthRepository]: [session] is what a token store would hold. */
class FakeAuthRepository(var session: Tokens? = null) : AuthRepository {
    var nextSignIn: SignInResult = SignInResult.SignedIn(Tokens("a", "r"))
    var signOuts = 0

    /** When set, [signIn] suspends until it completes: a login still in flight. */
    var signInGate: CompletableDeferred<Unit>? = null
    var failRead = false

    override suspend fun hasSession(): Boolean {
        if (failRead) error("keychain unavailable")
        return session != null
    }

    override suspend fun signIn(email: String, password: String): SignInResult {
        signInGate?.await()
        return nextSignIn.also { if (it is SignInResult.SignedIn) session = it.tokens }
    }

    /** What [verifyCode] does: store these tokens, or throw this error. */
    var nextCode: Result<Tokens> = Result.success(Tokens("a-code", "r-code"))
    val verified = mutableListOf<Pair<String, String>>()

    override suspend fun verifyCode(challengeId: String, code: String) {
        verified += challengeId to code
        session = nextCode.getOrThrow()
    }

    override suspend fun resendCode(challengeId: String): VerificationInfo =
        VerificationInfo(challengeId, "login", "a***@b.c", 600, 60, 2)

    override suspend fun signOut() {
        signOuts++
        session = null
    }
}

/** Records what [SessionGate] asked of the engine; [end] plays the engine ending the session itself. */
class FakeSessionEngine : SessionEngine {
    var begins = 0
    var wipes = 0
    private val endings = MutableSharedFlow<Unit>(extraBufferCapacity = 1)

    override suspend fun begin() {
        begins++
    }

    override suspend fun wipe() {
        wipes++
    }

    override val ended = endings

    fun end() {
        endings.tryEmit(Unit)
    }
}
