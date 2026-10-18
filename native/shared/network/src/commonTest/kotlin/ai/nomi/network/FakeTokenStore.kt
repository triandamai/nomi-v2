package ai.nomi.network

class FakeTokenStore(private var tokens: Tokens? = null) : TokenStore {
    var clearCount = 0
        private set

    override suspend fun read(): Tokens? = tokens
    override suspend fun write(tokens: Tokens) { this.tokens = tokens }
    override suspend fun clear() { tokens = null; clearCount++ }
}
