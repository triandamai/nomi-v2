package ai.nomi.app

import ai.nomi.app.di.AppScope
import ai.nomi.app.di.appModules
import ai.nomi.data.AuthRepository
import ai.nomi.data.NomiEngine
import ai.nomi.data.SdkSetup
import ai.nomi.data.SessionEngine
import ai.nomi.data.TokenStoreCredentials
import ai.nomi.network.ApiConfig
import ai.nomi.network.CrewMemberDto
import ai.nomi.network.HomeApi
import ai.nomi.network.ProfileDto
import ai.nomi.network.SignInResult
import ai.nomi.network.TokenStore
import ai.nomi.network.Tokens
import ai.nomi.network.VerificationInfo
import ai.nomi.sdk.NativeNomiClient
import ai.nomi.sdk.NomiClient
import ai.nomi.sdk.defaultSdkConfig
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.emitAll
import kotlinx.coroutines.flow.emptyFlow
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.update
import org.koin.core.module.Module
import org.koin.dsl.module
import java.nio.file.Files

/** What screens asked the engine to do on the backend: approval decisions, and the chats kept in sync right now. */
class BackendRecorder {
    val approvals = mutableListOf<Triple<String, Boolean, Boolean>>()

    /** Thrown by the next approval decisions instead of recording them, while set. */
    var approvalFailure: Exception? = null
    val syncing = MutableStateFlow<Map<String, Int>>(emptyMap())
}

/** The real engine (against a backend that can't be reached) with the screens' backend calls going to [backend]. */
class RecordingNomiClient(private val real: NomiClient, private val backend: BackendRecorder) : NomiClient by real {
    override suspend fun resolveApproval(sessionId: String, messageId: String, approve: Boolean, remember: Boolean) {
        backend.approvalFailure?.let { throw it }
        backend.approvals += Triple(messageId, approve, remember)
    }

    override fun syncChat(sessionId: String): Flow<Nothing> = flow {
        backend.syncing.update { it + (sessionId to (it[sessionId] ?: 0) + 1) }
        try {
            emitAll(real.syncChat(sessionId))
        } finally {
            backend.syncing.update { it + (sessionId to (it[sessionId] ?: 1) - 1) }
        }
    }
}

/** Records what the session gate asked of the engine. */
class FakeSessionEngine : SessionEngine {
    var begins = 0
        private set
    var wipes = 0
        private set

    override suspend fun begin() {
        begins++
    }

    override suspend fun wipe() {
        wipes++
    }

    override val ended: Flow<Unit> = emptyFlow()
}

class FakeTokenStore(private var tokens: Tokens? = null) : TokenStore {
    override suspend fun read(): Tokens? = tokens
    override suspend fun write(tokens: Tokens) {
        this.tokens = tokens
    }
    override suspend fun clear() {
        tokens = null
    }
}

class FakeAuthRepository(var session: Tokens? = null) : AuthRepository {
    var nextSignIn: SignInResult = SignInResult.SignedIn(Tokens("a", "r"))
    val signIns = mutableListOf<Pair<String, String>>()

    override suspend fun hasSession(): Boolean = session != null

    override suspend fun signIn(email: String, password: String): SignInResult {
        signIns += email to password
        return nextSignIn.also { if (it is SignInResult.SignedIn) session = it.tokens }
    }

    var nextCode: Result<Tokens> = Result.success(Tokens("a-code", "r-code"))
    val verified = mutableListOf<Pair<String, String>>()
    var nextResend: Result<VerificationInfo> = Result.success(VerificationInfo("c1", "login", "a***@b.c", 600, 60, 2))
    val resends = mutableListOf<String>()

    override suspend fun verifyCode(challengeId: String, code: String) {
        verified += challengeId to code
        session = nextCode.getOrThrow()
    }

    override suspend fun resendCode(challengeId: String): VerificationInfo {
        resends += challengeId
        return nextResend.getOrThrow()
    }

    override suspend fun signOut() {
        session = null
    }
}

class FakeHomeApi : HomeApi {
    var displayName: String? = "Ana Lima"
    var crew: List<CrewMemberDto> = emptyList()
    override suspend fun profile() = ProfileDto(displayName = displayName, email = "ana@nomi.test")
    override suspend fun crew() = crew
}

/** What a platform module provides, with nothing touching the network; the engine's data goes to a temp folder. */
val testPlatformModule: Module = module {
    single { ApiConfig("http://nomi.test") }
    single<TokenStore> { FakeTokenStore() }
    single { SdkSetup(UNREACHABLE_BASE_URL, Files.createTempDirectory("nomi-app-test").toFile().apply { deleteOnExit() }.path) }
}

/** A port nothing listens on: the engine's requests are refused at once. */
const val UNREACHABLE_BASE_URL = "http://127.0.0.1:9"

/** The real graph with auth and the engine's session swapped for fakes, and the engine's backend calls recorded. */
class TestGraph {
    val auth = FakeAuthRepository()
    val engine = FakeSessionEngine()
    val backend = BackendRecorder()

    val modules: List<Module> = appModules(testPlatformModule) + module {
        single<AuthRepository> { auth }
        single<SessionEngine> { engine }
        single<HomeApi> { FakeHomeApi() }
        single {
            val setup = get<SdkSetup>()
            NomiEngine(get(AppScope), get()) {
                val real = NativeNomiClient(defaultSdkConfig(setup.baseUrl, setup.dataDir), TokenStoreCredentials(FakeTokenStore()))
                RecordingNomiClient(real, backend)
            }
        }
    }
}
