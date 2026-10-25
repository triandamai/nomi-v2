import { describe, expect, it } from 'vitest';
import { createPkcePair, type AuthEvent } from '../src/auth.js';
import { ANON_KEY, client, mockFetch, session } from './helpers.js';

class Store {
  items = new Map<string, string>();
  getItem(key: string) {
    return this.items.get(key) ?? null;
  }
  setItem(key: string, value: string) {
    this.items.set(key, value);
  }
  removeItem(key: string) {
    this.items.delete(key);
  }
}

describe('email code sign-in', () => {
  it('sends the code, verifies it and uses the new session for requests', async () => {
    const { fetch, calls } = mockFetch({ status: 204 }, { body: session() }, { body: [] });
    const nomi = client(fetch);
    const events: AuthEvent[] = [];
    nomi.auth.onAuthStateChange((event) => events.push(event));

    await nomi.auth.signInWithCode('a@b.test');
    const verified = await nomi.auth.verifyCode('a@b.test', '123456');
    await nomi.from('task').select();

    expect(verified.data?.user.id).toBe('user-1');
    expect(calls[0]!.url.pathname).toBe('/auth/otp');
    expect(calls[0]!.body).toEqual({ email: 'a@b.test', create_user: true });
    expect(calls[1]!.body).toEqual({ email: 'a@b.test', code: '123456' });
    expect(calls[2]!.headers.authorization).toBe('Bearer access-1');
    expect(events).toContain('SIGNED_IN');
  });

  it('keeps the session in the given store across clients', async () => {
    const storage = new Store();
    const first = mockFetch({ body: session() });
    await client(first.fetch, { auth: { storage } }).auth.verifyCode('a@b.test', '1');

    const second = mockFetch({ body: [] });
    await client(second.fetch, { auth: { storage } }).from('task').select();
    expect(second.calls[0]!.headers.authorization).toBe('Bearer access-1');
  });
});

describe('refresh', () => {
  it('refreshes an expiring token before using it, once for concurrent calls', async () => {
    const storage = new Store();
    const seed = mockFetch({ body: session({ expires_at: Math.floor(Date.now() / 1000) + 5 }) });
    await client(seed.fetch, { auth: { storage } }).auth.verifyCode('a@b.test', '1');

    const { fetch, calls } = mockFetch({ body: session({ access_token: 'access-2', refresh_token: 'refresh-2' }) }, { body: [] });
    const nomi = client(fetch, { auth: { storage } });
    await Promise.all([nomi.from('task').select(), nomi.from('task').select()]);

    const refreshes = calls.filter((call) => call.url.pathname === '/auth/token');
    expect(refreshes).toHaveLength(1);
    expect(refreshes[0]!.url.searchParams.get('grant_type')).toBe('refresh_token');
    expect(refreshes[0]!.body).toEqual({ refresh_token: 'refresh-1' });
    expect(refreshes[0]!.headers.authorization).toBe(`Bearer ${ANON_KEY}`);
    expect(calls.filter((call) => call.url.pathname === '/rest/task').map((call) => call.headers.authorization)).toEqual([
      'Bearer access-2',
      'Bearer access-2'
    ]);
  });

  it('signs out when the refresh token was already used', async () => {
    const storage = new Store();
    const seed = mockFetch({ body: session({ expires_at: 0 }) });
    await client(seed.fetch, { auth: { storage } }).auth.verifyCode('a@b.test', '1');

    const { fetch } = mockFetch({ status: 401, body: { message: 'refresh token reused', code: 'refresh_token_reused' } });
    const nomi = client(fetch, { auth: { storage } });
    const events: AuthEvent[] = [];
    nomi.auth.onAuthStateChange((event) => events.push(event));
    const result = await nomi.auth.getSession();

    expect(result.error?.code).toBe('refresh_token_reused');
    expect(events).toContain('SIGNED_OUT');
    expect(storage.items.size).toBe(0);
  });
});

describe('OAuth', () => {
  it('builds the authorize URL with PKCE and finishes with the verifier', async () => {
    const storage = new Store();
    const { fetch, calls } = mockFetch({ body: session() });
    const nomi = client(fetch, { auth: { storage } });

    const { data } = await nomi.auth.signInWithOAuth('github', { redirectTo: 'https://app.test/done', scopes: ['read:user'] });
    const url = new URL(data!.url);
    expect(url.pathname).toBe('/auth/authorize/github');
    expect(url.searchParams.get('redirect_to')).toBe('https://app.test/done');
    expect(url.searchParams.get('scopes')).toBe('read:user');
    expect(url.searchParams.get('code_challenge_method')).toBe('S256');
    const verifier = [...storage.items.values()][0]!;

    await nomi.auth.exchangeCodeForSession('one-time');
    expect(calls[0]!.url.searchParams.get('grant_type')).toBe('pkce');
    expect(calls[0]!.body).toEqual({ code: 'one-time', code_verifier: verifier });
    expect((await nomi.auth.getSession()).data?.session?.access_token).toBe('access-1');
  });

  it('refuses to exchange a code without a sign-in in progress', async () => {
    const { fetch, calls } = mockFetch({ body: session() });
    const result = await client(fetch).auth.exchangeCodeForSession('x');
    expect(result.error?.code).toBe('pkce_verifier_missing');
    expect(calls).toHaveLength(0);
  });

  it('makes an S256 challenge of the verifier', async () => {
    const { verifier, challenge } = await createPkcePair();
    const digest = new Uint8Array(await crypto.subtle.digest('SHA-256', new TextEncoder().encode(verifier)));
    expect(challenge).toBe(Buffer.from(digest).toString('base64url'));
    expect(verifier.length).toBeGreaterThanOrEqual(43);
  });
});

describe('admin', () => {
  it('uses the key, not a user session', async () => {
    const { fetch, calls } = mockFetch({ body: { id: 'u2' } });
    const nomi = client(fetch);
    await nomi.auth.admin.createUser({ email: 'new@b.test', email_verified: true });
    expect(calls[0]!.url.pathname).toBe('/auth/admin/users');
    expect(calls[0]!.headers.authorization).toBe(`Bearer ${ANON_KEY}`);
  });
});
