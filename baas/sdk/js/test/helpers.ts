import { vi } from 'vitest';
import { createClient, type ClientOptions } from '../src/index.js';

export const URL_BASE = 'https://todo.api.nomi.test';
export const ANON_KEY = 'anon-key';

export interface Call {
  url: URL;
  method: string;
  headers: Record<string, string>;
  body: unknown;
}

type Reply = { status?: number; body?: unknown; headers?: Record<string, string> };

/** A fetch that records each call and answers from `replies` in order (the last one repeats). */
export function mockFetch(...replies: Reply[]) {
  const calls: Call[] = [];
  const fetch = vi.fn(async (input: RequestInfo | URL, init: RequestInit = {}) => {
    const headers = Object.fromEntries(Object.entries((init.headers ?? {}) as Record<string, string>));
    let body: unknown = init.body;
    if (typeof body === 'string' && headers['content-type'] === 'application/json') body = JSON.parse(body);
    calls.push({ url: new URL(String(input)), method: init.method ?? 'GET', headers, body });
    const reply = replies[Math.min(calls.length - 1, replies.length - 1)] ?? {};
    const status = reply.status ?? 200;
    const isJson = reply.body !== undefined && typeof reply.body !== 'string';
    return new Response(isJson ? JSON.stringify(reply.body) : (reply.body as string | undefined) ?? null, {
      status,
      headers: { ...(isJson ? { 'content-type': 'application/json' } : {}), ...reply.headers }
    });
  });
  return { fetch: fetch as unknown as typeof globalThis.fetch, calls };
}

export function client(fetch: typeof globalThis.fetch, options: ClientOptions = {}) {
  return createClient(URL_BASE, ANON_KEY, {
    fetch,
    ...options,
    auth: { persistSession: false, autoRefreshToken: false, detectSessionInUrl: false, ...options.auth }
  });
}

export function session(overrides: Record<string, unknown> = {}) {
  return {
    access_token: 'access-1',
    refresh_token: 'refresh-1',
    token_type: 'bearer',
    expires_at: Math.floor(Date.now() / 1000) + 3600,
    user: { id: 'user-1', email: 'a@b.test', email_verified: true, created_at: '2026-01-01T00:00:00Z' },
    ...overrides
  };
}
