import { NomiError } from './errors.js';
import { buildUrl, request, type HttpContext } from './http.js';
import type { Result } from './types.js';

/** Refresh the access token this long before it expires, so a request never goes out with a dead one. */
export const REFRESH_MARGIN_SECONDS = 60;
const MS_PER_SECOND = 1000;
/** Bytes of randomness in a PKCE code verifier (RFC 7636 asks for 32–96). */
const PKCE_VERIFIER_BYTES = 32;
const HTTP_OK = 200;
const HTTP_BAD_REQUEST = 400;
const HTTP_UNAUTHORIZED = 401;

export type Provider = 'google' | 'github' | 'apple' | (string & {});

export interface Identity {
  id: string;
  provider: Provider;
  provider_user_id: string;
  email: string | null;
  created_at: string;
}

export interface User {
  id: string;
  email: string | null;
  email_verified: boolean;
  created_at: string;
  last_sign_in_at?: string | null;
  identities?: Identity[];
  user_metadata?: Record<string, unknown>;
}

export interface Session {
  access_token: string;
  refresh_token: string;
  token_type: 'bearer';
  /** Unix time, in seconds, when the access token stops working. */
  expires_at: number;
  user: User;
}

export type AuthEvent = 'INITIAL_SESSION' | 'SIGNED_IN' | 'SIGNED_OUT' | 'TOKEN_REFRESHED' | 'USER_UPDATED';
export type AuthListener = (event: AuthEvent, session: Session | null) => void;

/** Where the session is kept. `localStorage` fits; so does anything with the same three methods. */
export interface SessionStore {
  getItem(key: string): string | null | Promise<string | null>;
  setItem(key: string, value: string): void | Promise<void>;
  removeItem(key: string): void | Promise<void>;
}

export interface AuthOptions {
  /** Keep the session across page loads. Defaults to true in a browser, false elsewhere. */
  persistSession?: boolean;
  storage?: SessionStore;
  /** Refresh the access token in the background before it expires. Defaults to true. */
  autoRefreshToken?: boolean;
  /** Finish an OAuth sign-in when the page loads with `?code=` in the URL. Defaults to true in a browser. */
  detectSessionInUrl?: boolean;
}

export interface OAuthOptions {
  /** Where Relay sends the browser after sign-in. Must be one of the project's `redirect_urls`. */
  redirectTo?: string;
  scopes?: string[];
  /** Return the URL instead of navigating to it, e.g. to open it in a popup. */
  skipBrowserRedirect?: boolean;
}

export interface AdminUserAttributes {
  email?: string;
  email_verified?: boolean;
  user_metadata?: Record<string, unknown>;
}

class MemoryStore implements SessionStore {
  private readonly items = new Map<string, string>();
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

function browserStorage(): SessionStore | null {
  try {
    return typeof globalThis.localStorage === 'undefined' ? null : globalThis.localStorage;
  } catch {
    return null;
  }
}

const inBrowser = () => typeof window !== 'undefined' && typeof window.location !== 'undefined';
const nowSeconds = () => Math.floor(Date.now() / MS_PER_SECOND);

function base64Url(bytes: Uint8Array): string {
  let binary = '';
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return btoa(binary).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}

export async function createPkcePair(): Promise<{ verifier: string; challenge: string }> {
  const verifier = base64Url(crypto.getRandomValues(new Uint8Array(PKCE_VERIFIER_BYTES)));
  const digest = await crypto.subtle.digest('SHA-256', new TextEncoder().encode(verifier));
  return { verifier, challenge: base64Url(new Uint8Array(digest)) };
}

/** Sign-in, sessions and the signed-in user. Reached as `nomi.auth`. */
export class AuthClient {
  readonly admin: AuthAdmin;
  private session: Session | null = null;
  private readonly listeners = new Set<AuthListener>();
  private readonly storage: SessionStore;
  private readonly storageKey: string;
  private readonly verifierKey: string;
  private readonly autoRefresh: boolean;
  private refreshTimer: ReturnType<typeof setTimeout> | null = null;
  private refreshing: Promise<Result<Session>> | null = null;
  private readonly ready: Promise<void>;

  constructor(
    private readonly http: HttpContext,
    options: AuthOptions = {}
  ) {
    const host = new URL(http.baseUrl).host;
    this.storageKey = `nomi.session.${host}`;
    this.verifierKey = `nomi.pkce.${host}`;
    const persist = options.persistSession ?? browserStorage() !== null;
    this.storage = options.storage ?? (persist ? (browserStorage() ?? new MemoryStore()) : new MemoryStore());
    this.autoRefresh = options.autoRefreshToken ?? true;
    this.admin = new AuthAdmin(http);
    this.ready = this.initialize(options.detectSessionInUrl ?? inBrowser());
  }

  /** Emails a six-digit sign-in code. Finish with `verifyCode`. */
  async signInWithCode(email: string, options: { createUser?: boolean } = {}): Promise<Result<null>> {
    return request<null>(this.http, '/auth/otp', {
      method: 'POST',
      body: { email, create_user: options.createUser ?? true },
      token: this.http.key
    });
  }

  async verifyCode(email: string, code: string): Promise<Result<Session>> {
    const result = await request<Session>(this.http, '/auth/verify', {
      method: 'POST',
      body: { email, code },
      token: this.http.key
    });
    if (result.data) await this.setSession(result.data, 'SIGNED_IN');
    return result;
  }

  /**
   * Starts sign-in with Google, GitHub, Apple or a project's OpenID Connect provider. In a browser
   * this navigates away; the page at `redirectTo` finishes it (automatically, unless
   * `detectSessionInUrl` is off, then call `exchangeCodeForSession`).
   */
  async signInWithOAuth(provider: Provider, options: OAuthOptions = {}): Promise<Result<{ url: string; provider: Provider }>> {
    const challenge = await this.newChallenge();
    const url = buildUrl(this.http.baseUrl, `/auth/authorize/${encodeURIComponent(provider)}`, {
      redirect_to: options.redirectTo ?? (inBrowser() ? window.location.href : undefined),
      scopes: options.scopes?.join(' '),
      code_challenge: challenge,
      code_challenge_method: 'S256',
      apikey: this.http.key
    });
    if (inBrowser() && !options.skipBrowserRedirect) window.location.assign(url);
    return { data: { url, provider }, error: null, status: HTTP_OK };
  }

  /** Trades the one-time `code` from an OAuth redirect for a session. */
  async exchangeCodeForSession(code: string): Promise<Result<Session>> {
    const verifier = await this.storage.getItem(this.verifierKey);
    if (!verifier) {
      return {
        data: null,
        error: new NomiError('No sign-in is in progress in this browser', HTTP_BAD_REQUEST, 'pkce_verifier_missing'),
        status: HTTP_BAD_REQUEST
      };
    }
    const result = await request<Session>(this.http, '/auth/token', {
      method: 'POST',
      query: { grant_type: 'pkce' },
      body: { code, code_verifier: verifier },
      token: this.http.key
    });
    await this.storage.removeItem(this.verifierKey);
    if (result.data) await this.setSession(result.data, 'SIGNED_IN');
    return result;
  }

  /** The current session, refreshed first if it's about to expire. */
  async getSession(): Promise<Result<{ session: Session | null }>> {
    await this.ready;
    if (this.session && this.session.expires_at - REFRESH_MARGIN_SECONDS <= nowSeconds()) {
      const refreshed = await this.refreshSession();
      if (refreshed.error) return { data: null, error: refreshed.error, status: refreshed.status };
    }
    return { data: { session: this.session }, error: null, status: HTTP_OK };
  }

  /** Asks Relay for the signed-in user, so the answer reflects removals and changes made elsewhere. */
  async getUser(): Promise<Result<{ user: User }>> {
    const { data, error, status } = await this.getSession();
    if (error) return { data: null, error, status };
    if (!data.session) return { data: null, error: new NomiError('Not signed in', HTTP_UNAUTHORIZED, 'not_signed_in'), status: HTTP_UNAUTHORIZED };
    const result = await request<User>(this.http, '/auth/user');
    if (result.error) return result;
    return { data: { user: result.data }, error: null, status: result.status };
  }

  async updateUser(attributes: { email?: string; user_metadata?: Record<string, unknown> }): Promise<Result<{ user: User }>> {
    const result = await request<User>(this.http, '/auth/user', { method: 'PATCH', body: attributes });
    if (result.error) return result;
    if (this.session) await this.setSession({ ...this.session, user: result.data }, 'USER_UPDATED');
    return { data: { user: result.data }, error: null, status: result.status };
  }

  /** Gets a new access token. A refresh token works once; each refresh hands back a new one. */
  refreshSession(): Promise<Result<Session>> {
    this.refreshing ??= this.doRefresh().finally(() => {
      this.refreshing = null;
    });
    return this.refreshing;
  }

  async signOut(): Promise<Result<null>> {
    await this.ready;
    const token = this.session?.access_token;
    await this.clearSession();
    if (!token) return { data: null, error: null, status: HTTP_OK };
    const result = await request<null>(this.http, '/auth/logout', { method: 'POST', token });
    // The session is gone locally either way; an already-revoked token isn't worth reporting.
    if (result.error?.status === HTTP_UNAUTHORIZED) return { data: null, error: null, status: HTTP_OK };
    return result;
  }

  /** Adds another sign-in method to the signed-in user. Navigates away like `signInWithOAuth`. */
  async linkIdentity(provider: Provider, options: OAuthOptions = {}): Promise<Result<{ url: string; provider: Provider }>> {
    const challenge = await this.newChallenge();
    const result = await request<{ url: string }>(this.http, '/auth/identities', {
      method: 'POST',
      body: {
        provider,
        redirect_to: options.redirectTo ?? (inBrowser() ? window.location.href : undefined),
        scopes: options.scopes?.join(' '),
        code_challenge: challenge,
        code_challenge_method: 'S256'
      }
    });
    if (result.error) return result;
    if (inBrowser() && !options.skipBrowserRedirect) window.location.assign(result.data.url);
    return { data: { url: result.data.url, provider }, error: null, status: result.status };
  }

  /** Removes a sign-in method. Relay refuses to remove a user's last one. */
  async unlinkIdentity(identityId: string): Promise<Result<null>> {
    const result = await request<null>(this.http, `/auth/identities/${encodeURIComponent(identityId)}`, { method: 'DELETE' });
    if (!result.error && this.session) {
      const identities = this.session.user.identities?.filter((identity) => identity.id !== identityId);
      await this.setSession({ ...this.session, user: { ...this.session.user, identities } }, 'USER_UPDATED');
    }
    return result;
  }

  /** Calls `listener` on every sign-in, sign-out and refresh, starting with the current session. */
  onAuthStateChange(listener: AuthListener): { unsubscribe: () => void } {
    this.listeners.add(listener);
    void this.ready.then(() => {
      if (this.listeners.has(listener)) listener('INITIAL_SESSION', this.session);
    });
    return { unsubscribe: () => this.listeners.delete(listener) };
  }

  /** The bearer token for a request: the user's access token, or null when nobody is signed in. */
  async accessToken(): Promise<string | null> {
    const { data } = await this.getSession();
    return data?.session?.access_token ?? null;
  }

  /** Stops the background refresh. Call it when throwing a client away in tests or servers. */
  dispose(): void {
    if (this.refreshTimer) clearTimeout(this.refreshTimer);
    this.refreshTimer = null;
    this.listeners.clear();
  }

  private async initialize(detectInUrl: boolean): Promise<void> {
    const stored = await this.storage.getItem(this.storageKey);
    if (stored) {
      try {
        this.session = JSON.parse(stored) as Session;
        this.scheduleRefresh();
      } catch {
        await this.storage.removeItem(this.storageKey);
      }
    }
    if (detectInUrl && inBrowser()) await this.finishRedirect();
  }

  private async finishRedirect(): Promise<void> {
    const url = new URL(window.location.href);
    const code = url.searchParams.get('code');
    if (!code || !(await this.storage.getItem(this.verifierKey))) return;
    url.searchParams.delete('code');
    window.history.replaceState(window.history.state, '', url.toString());
    await this.exchangeCodeForSession(code);
  }

  private async newChallenge(): Promise<string> {
    const { verifier, challenge } = await createPkcePair();
    await this.storage.setItem(this.verifierKey, verifier);
    return challenge;
  }

  private async doRefresh(): Promise<Result<Session>> {
    const refreshToken = this.session?.refresh_token;
    if (!refreshToken) {
      return { data: null, error: new NomiError('Not signed in', HTTP_UNAUTHORIZED, 'not_signed_in'), status: HTTP_UNAUTHORIZED };
    }
    const result = await request<Session>(this.http, '/auth/token', {
      method: 'POST',
      query: { grant_type: 'refresh_token' },
      body: { refresh_token: refreshToken },
      token: this.http.key
    });
    if (result.data) {
      await this.setSession(result.data, 'TOKEN_REFRESHED');
    } else if (result.error.status === HTTP_UNAUTHORIZED || result.error.status === HTTP_BAD_REQUEST) {
      // The refresh token was revoked or already used: this session is over.
      await this.clearSession();
    }
    return result;
  }

  private async setSession(session: Session, event: AuthEvent): Promise<void> {
    this.session = session;
    await this.storage.setItem(this.storageKey, JSON.stringify(session));
    this.scheduleRefresh();
    this.emit(event);
  }

  private async clearSession(): Promise<void> {
    const hadSession = this.session !== null;
    this.session = null;
    if (this.refreshTimer) clearTimeout(this.refreshTimer);
    this.refreshTimer = null;
    await this.storage.removeItem(this.storageKey);
    if (hadSession) this.emit('SIGNED_OUT');
  }

  private scheduleRefresh(): void {
    if (this.refreshTimer) clearTimeout(this.refreshTimer);
    this.refreshTimer = null;
    if (!this.autoRefresh || !this.session) return;
    const delay = Math.max(0, (this.session.expires_at - REFRESH_MARGIN_SECONDS - nowSeconds()) * MS_PER_SECOND);
    this.refreshTimer = setTimeout(() => void this.refreshSession(), delay);
    // Don't keep a Node process alive just to refresh a token.
    (this.refreshTimer as { unref?: () => void }).unref?.();
  }

  private emit(event: AuthEvent): void {
    for (const listener of this.listeners) listener(event, this.session);
  }
}

/** User management for server code and functions. Needs a client made with the service key. */
export class AuthAdmin {
  constructor(private readonly http: HttpContext) {}

  listUsers(options: { page?: number; perPage?: number } = {}): Promise<Result<{ users: User[]; total: number }>> {
    return request(this.http, '/auth/admin/users', {
      query: { page: options.page?.toString(), per_page: options.perPage?.toString() },
      token: this.http.key
    });
  }
  getUserById(id: string): Promise<Result<User>> {
    return request(this.http, `/auth/admin/users/${encodeURIComponent(id)}`, { token: this.http.key });
  }
  createUser(attributes: AdminUserAttributes & { email: string }): Promise<Result<User>> {
    return request(this.http, '/auth/admin/users', { method: 'POST', body: attributes, token: this.http.key });
  }
  updateUserById(id: string, attributes: AdminUserAttributes): Promise<Result<User>> {
    return request(this.http, `/auth/admin/users/${encodeURIComponent(id)}`, {
      method: 'PATCH',
      body: attributes,
      token: this.http.key
    });
  }
  /** Deletes the user and ends all their sessions. Rows that reference them follow your `on delete` rules. */
  deleteUser(id: string): Promise<Result<null>> {
    return request(this.http, `/auth/admin/users/${encodeURIComponent(id)}`, { method: 'DELETE', token: this.http.key });
  }
}
