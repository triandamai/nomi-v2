import { describe, expect, it } from 'vitest';
import { createFunctionContext, defineFunction, isNomiFunction, CLAIMS_HEADER } from '../src/functions/index.js';
import { client, mockFetch, URL_BASE } from './helpers.js';

describe('storage', () => {
  it('uploads to an encoded path without overwriting by default', async () => {
    const { fetch, calls } = mockFetch({ body: { path: 'u 1/a.png' } });
    const file = new Blob(['x'], { type: 'image/png' });
    await client(fetch).storage.from('avatars').upload('u 1/a.png', file);
    expect(calls[0]!.method).toBe('PUT');
    expect(calls[0]!.url.pathname).toBe('/storage/object/avatars/u%201/a.png');
    expect(calls[0]!.headers['content-type']).toBe('image/png');
    expect(calls[0]!.headers['x-nomi-upsert']).toBe('false');
  });

  it('returns an absolute signed URL', async () => {
    const { fetch } = mockFetch({ body: { signed_url: '/storage/sign/avatars/a.png?token=t' } });
    const { data } = await client(fetch).storage.from('avatars').createSignedUrl('a.png', 60);
    expect(data?.signedUrl).toBe(`${URL_BASE}/storage/sign/avatars/a.png?token=t`);
  });

  it('builds public URLs without a request', () => {
    const { fetch, calls } = mockFetch();
    expect(client(fetch).storage.from('pub').getPublicUrl('/a/b.png').publicUrl).toBe(`${URL_BASE}/storage/public/pub/a/b.png`);
    expect(calls).toHaveLength(0);
  });
});

describe('functions.invoke', () => {
  it('posts JSON and parses a JSON answer', async () => {
    const { fetch, calls } = mockFetch({ body: { id: 9 } });
    const { data } = await client(fetch).functions.invoke('place-order', { body: { cart: [1] } });
    expect(data).toEqual({ id: 9 });
    expect(calls[0]!.url.pathname).toBe('/functions/place-order');
    expect(calls[0]!.method).toBe('POST');
    expect(calls[0]!.body).toEqual({ cart: [1] });
  });

  it('uses GET with no body and returns text answers as text', async () => {
    const { fetch, calls } = mockFetch({ body: 'pong' });
    const { data } = await client(fetch).functions.invoke('ping');
    expect(data).toBe('pong');
    expect(calls[0]!.method).toBe('GET');
  });
});

describe('nomi-baas/functions', () => {
  const env = { NOMI_URL: URL_BASE, NOMI_ANON_KEY: 'anon', NOMI_SERVICE_KEY: 'service', STRIPE_SECRET: 's' };

  it('marks handlers made with defineFunction', () => {
    const fn = defineFunction(() => new Response('ok'));
    expect(isNomiFunction(fn)).toBe(true);
    expect(isNomiFunction(() => new Response('ok'))).toBe(false);
  });

  it('builds ctx from the claims and token the gateway forwarded', () => {
    const claims = Buffer.from(JSON.stringify({ sub: 'user-1', role: 'authenticated', email: 'a@b.test' })).toString('base64url');
    const req = new Request('http://fn/place-order', { headers: { [CLAIMS_HEADER]: claims, authorization: 'Bearer user-token' } });
    const ctx = createFunctionContext(req, env);
    expect(ctx.user).toEqual({ id: 'user-1', email: 'a@b.test', role: 'authenticated' });
    expect(ctx.env.STRIPE_SECRET).toBe('s');
    ctx.nomi.dispose();
    ctx.admin.dispose();
  });

  it('treats a call without claims as anonymous', () => {
    const ctx = createFunctionContext(new Request('http://fn/hook'), env);
    expect(ctx.user).toBeNull();
    expect(ctx.claims.role).toBe('anon');
  });

  it('refuses to start without the NOMI_* values', () => {
    expect(() => createFunctionContext(new Request('http://fn/x'), {})).toThrow(/NOMI_URL/);
  });
});
