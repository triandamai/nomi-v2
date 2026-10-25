import { readBody, send, type HttpContext } from './http.js';
import type { Result } from './types.js';

/**
 * A function's own default timeout is 30 seconds, and a sleeping functions service takes a couple
 * of seconds to wake. Allow both, plus room for the network.
 */
export const FUNCTION_INVOKE_TIMEOUT_MS = 45_000;

export interface InvokeOptions {
  /** A plain object is sent as JSON; a string, Blob or FormData is sent as is. */
  body?: unknown;
  method?: 'GET' | 'POST' | 'PUT' | 'PATCH' | 'DELETE';
  headers?: Record<string, string>;
  query?: Record<string, string>;
  timeoutMs?: number;
}

/** Calls hosted functions in `nomi/functions/`. Reached as `nomi.functions`. */
export class FunctionsClient {
  constructor(private readonly http: HttpContext) {}

  /** Calls `nomi/functions/<name>.ts`. JSON answers are parsed; anything else comes back as text. */
  async invoke<T = unknown>(name: string, options: InvokeOptions = {}): Promise<Result<T>> {
    const { response, error } = await send(this.http, `/functions/${encodeURIComponent(name)}`, {
      method: options.method ?? (options.body === undefined ? 'GET' : 'POST'),
      body: options.body,
      headers: options.headers,
      query: options.query,
      timeoutMs: options.timeoutMs ?? FUNCTION_INVOKE_TIMEOUT_MS
    });
    if (error || !response) return { data: null, error: error!, status: response?.status ?? 0 };
    return { data: (await readBody(response)) as T, error: null, status: response.status };
  }
}
