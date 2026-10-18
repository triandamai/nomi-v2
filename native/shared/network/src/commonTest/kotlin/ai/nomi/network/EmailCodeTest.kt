package ai.nomi.network

import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.client.engine.mock.toByteArray
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.http.headersOf
import kotlinx.coroutines.test.runTest
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertFailsWith
import kotlin.test.assertNull

class EmailCodeTest {
    private val config = ApiConfig("https://nomi.example")
    private val json = headersOf(HttpHeaders.ContentType, "application/json")

    private fun api(store: TokenStore, status: HttpStatusCode, body: String, seen: MutableList<Pair<String, String>>? = null): AuthApi {
        val engine = MockEngine { request ->
            seen?.add(request.url.encodedPath to request.body.toByteArray().decodeToString())
            respond(body, status, json)
        }
        return AuthApi(nomiHttpClient(engine, config, store) {}, config, store)
    }

    @Test
    fun rightCodeStoresTheSession() = runTest {
        val store = FakeTokenStore()
        val seen = mutableListOf<Pair<String, String>>()
        val api = api(store, HttpStatusCode.OK, """{"access_token":"a1","refresh_token":"r1"}""", seen)

        assertEquals(Tokens("a1", "r1"), api.verifyCode("c1", "123456"))
        assertEquals(listOf("/api/auth/verify" to """{"challenge_id":"c1","code":"123456"}"""), seen)
        assertEquals(Tokens("a1", "r1"), store.read())
    }

    @Test
    fun wrongCodeSaysHowManyTriesAreLeft() = runTest {
        val store = FakeTokenStore()
        val api = api(store, HttpStatusCode.BadRequest, """{"error":"wrong_code","attempts_left":2}""")

        assertEquals(EmailCodeError.WrongCode(2), assertFailsWith<EmailCodeException> { api.verifyCode("c1", "000000") }.error)
        assertNull(store.read())
    }

    @Test
    fun goneCodesAreExpiredOrLocked() = runTest {
        val expired = api(FakeTokenStore(), HttpStatusCode.Gone, """{"error":"expired"}""")
        assertEquals(EmailCodeError.Expired, assertFailsWith<EmailCodeException> { expired.verifyCode("c1", "1") }.error)
        val locked = api(FakeTokenStore(), HttpStatusCode.Gone, """{"error":"too_many_attempts"}""")
        assertEquals(EmailCodeError.TooManyAttempts, assertFailsWith<EmailCodeException> { locked.verifyCode("c1", "1") }.error)
    }

    @Test
    fun resendReturnsTheNewChallengeOrSaysWhy() = runTest {
        val seen = mutableListOf<Pair<String, String>>()
        val ok = api(
            FakeTokenStore(), HttpStatusCode.OK,
            """{"challenge_id":"c1","purpose":"login","email":"a***@b.c","expires_in":600,"resend_in":60,"sends_left":2}""",
            seen,
        )
        assertEquals(VerificationInfo("c1", "login", "a***@b.c", 600, 60, 2), ok.resendCode("c1"))
        assertEquals(listOf("/api/auth/resend" to """{"challenge_id":"c1"}"""), seen)

        val wait = api(FakeTokenStore(), HttpStatusCode.TooManyRequests, """{"error":"wait","seconds":42}""")
        assertEquals(EmailCodeError.Wait(42), assertFailsWith<EmailCodeException> { wait.resendCode("c1") }.error)
        val tooMany = api(FakeTokenStore(), HttpStatusCode.TooManyRequests, """{"error":"too_many_codes"}""")
        assertEquals(EmailCodeError.TooManyCodes, assertFailsWith<EmailCodeException> { tooMany.resendCode("c1") }.error)
    }

    @Test
    fun anythingElseIsAGenericFailure() = runTest {
        val api = api(FakeTokenStore(), HttpStatusCode.InternalServerError, """{"error":"failed"}""")
        assertEquals(EmailCodeError.Failed, assertFailsWith<EmailCodeException> { api.verifyCode("c1", "1") }.error)
        val notJson = api(FakeTokenStore(), HttpStatusCode.BadGateway, "<html>")
        assertEquals(EmailCodeError.Failed, assertFailsWith<EmailCodeException> { notJson.resendCode("c1") }.error)
    }
}
