package ai.nomi.network

import io.ktor.client.engine.HttpClientEngine

/** The platform's HTTP engine: OkHttp on Android and desktop, Darwin (NSURLSession) on iOS. */
expect fun defaultHttpEngine(): HttpClientEngine
