import { NomiError } from './errors.js';
import type { Fetch, Result } from './types.js';
import { VERSION } from './version.js';

/** How long a normal request may take before it's abandoned. */
export const DEFAULT_REQUEST_TIMEOUT_MS = 30_000;

/** Header that tells the gateway which client version is calling, so old versions keep working. */
export const CLIENT_HEADER = 'x-nomi-client';

const HTTP_NO_CONTENT = 204;

export interface HttpContext {
  baseUrl: string;
  key: string;
  fetch: Fetch;
  headers: Record<string, string>;
  /** The bearer token for this request: the user's access token when signed in, otherwise the key. */
  token: () => Promise<string>;
}

export interface RequestOptions {
  method?: string;
  query?: URLSearchParams | Record<string, string | undefined>;
  headers?: Record<string, string>;
  /** A plain object is sent as JSON; anything else (string, Blob, FormData…) is sent as is. */
  body?: unknown;
  timeoutMs?: number;
  /** Overrides the context's token, e.g. a refresh call that must not send a stale access token. */
  token?: string;
}

export interface RawResponse {
  response: Response | null;
  error: NomiError | null;
}

export function buildUrl(baseUrl: string, path: string, query?: RequestOptions['query']): string {
  const url = new URL(baseUrl + path);
  if (query instanceof URLSearchParams) {
    query.forEach((value, name) => url.searchParams.append(name, value));
  } else if (query) {
    for (const [name, value] of Object.entries(query)) {
      if (value !== undefined) url.searchParams.set(name, value);
    }
  }
  return url.toString();
}

function isPlainBody(body: unknown): boolean {
  if (body === null || typeof body !== 'object') return false;
  if (typeof Blob !== 'undefined' && body instanceof Blob) return false;
  if (typeof FormData !== 'undefined' && body instanceof FormData) return false;
  if (typeof URLSearchParams !== 'undefined' && body instanceof URLSearchParams) return false;
  if (body instanceof ArrayBuffer || ArrayBuffer.isView(body)) return false;
  if (typeof ReadableStream !== 'undefined' && body instanceof ReadableStream) return false;
  return true;
}

/** Sends a request and hands back the raw response, or a `NomiError` when it failed or wasn't 2xx. */
export async function send(ctx: HttpContext, path: string, options: RequestOptions = {}): Promise<RawResponse> {
  const headers: Record<string, string> = {
    apikey: ctx.key,
    [CLIENT_HEADER]: `js/${VERSION}`,
    ...ctx.headers,
    ...options.headers
  };
  headers.authorization = `Bearer ${options.token ?? (await ctx.token())}`;

  let body: BodyInit | undefined;
  if (options.body !== undefined) {
    if (isPlainBody(options.body)) {
      body = JSON.stringify(options.body);
      headers['content-type'] ??= 'application/json';
    } else {
      body = options.body as BodyInit;
    }
  }

  let response: Response;
  try {
    response = await ctx.fetch(buildUrl(ctx.baseUrl, path, options.query), {
      method: options.method ?? 'GET',
      headers,
      body,
      signal: AbortSignal.timeout(options.timeoutMs ?? DEFAULT_REQUEST_TIMEOUT_MS)
    });
  } catch (cause) {
    const timedOut = cause instanceof Error && cause.name === 'TimeoutError';
    return {
      response: null,
      error: new NomiError(
        timedOut ? 'The request timed out' : `Couldn't reach the backend: ${String(cause)}`,
        0,
        timedOut ? 'timeout' : 'network_error',
        cause
      )
    };
  }

  if (!response.ok) return { response, error: await errorFrom(response) };
  return { response, error: null };
}

/** Sends a request and parses a JSON answer. An empty answer becomes `null`. */
export async function request<T>(ctx: HttpContext, path: string, options: RequestOptions = {}): Promise<Result<T>> {
  const { response, error } = await send(ctx, path, options);
  if (error || !response) return { data: null, error: error!, status: response?.status ?? 0 };
  return { data: (await readBody(response)) as T, error: null, status: response.status };
}

export async function readBody(response: Response): Promise<unknown> {
  if (response.status === HTTP_NO_CONTENT) return null;
  const text = await response.text();
  if (!text) return null;
  const type = response.headers.get('content-type') ?? '';
  if (type.includes('json')) {
    try {
      return JSON.parse(text);
    } catch {
      return text;
    }
  }
  return text;
}

async function errorFrom(response: Response): Promise<NomiError> {
  const body = await readBody(response).catch(() => null);
  if (body && typeof body === 'object') {
    const { message, code, error, details, hint } = body as Record<string, unknown>;
    return new NomiError(
      String(message ?? error ?? response.statusText ?? 'Request failed'),
      response.status,
      String(code ?? `http_${response.status}`),
      details ?? hint ?? body
    );
  }
  return new NomiError(
    typeof body === 'string' && body ? body : response.statusText || `Request failed with ${response.status}`,
    response.status,
    `http_${response.status}`
  );
}
