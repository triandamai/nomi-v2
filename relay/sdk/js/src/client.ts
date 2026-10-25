import { AuthClient, type AuthOptions } from './auth.js';
import { FunctionsClient } from './functions.js';
import { request, type HttpContext } from './http.js';
import { QueryBuilder } from './query.js';
import { RealtimeClient, type RealtimeOptions } from './realtime.js';
import { StorageClient } from './storage.js';
import type { AnyDatabase, DatabaseDef, Fetch, Result } from './types.js';

export interface ClientOptions {
  auth?: AuthOptions;
  realtime?: RealtimeOptions;
  /** Headers sent with every request. */
  headers?: Record<string, string>;
  fetch?: Fetch;
  /**
   * Act as this user instead of managing a session: server code and functions pass the caller's
   * access token here, so row-level security applies to them.
   */
  accessToken?: string;
}

type Tables<DB extends DatabaseDef> = DB['tables'];
type Rpcs<DB extends DatabaseDef> = DB['functions'];

/** Everything an app calls. Make one with `createClient`. */
export class NomiClient<DB extends DatabaseDef = AnyDatabase> {
  readonly auth: AuthClient;
  readonly storage: StorageClient;
  readonly functions: FunctionsClient;
  readonly realtime: RealtimeClient;
  private readonly http: HttpContext;

  constructor(url: string, key: string, options: ClientOptions = {}) {
    if (!url) throw new Error('nomi-relay: createClient needs the project URL (is PUBLIC_NOMI_URL set?)');
    if (!key) throw new Error('nomi-relay: createClient needs a key (is PUBLIC_NOMI_KEY set?)');
    const baseUrl = new URL(url).toString().replace(/\/+$/, '');
    const fixedToken = options.accessToken;

    this.http = {
      baseUrl,
      key,
      fetch: options.fetch ?? ((...args) => globalThis.fetch(...args)),
      headers: options.headers ?? {},
      token: async () => fixedToken ?? (await this.auth.accessToken()) ?? key
    };
    this.auth = new AuthClient(this.http, {
      ...options.auth,
      // A client acting as a fixed user has no session of its own to keep or refresh.
      ...(fixedToken ? { persistSession: false, autoRefreshToken: false, detectSessionInUrl: false } : {})
    });
    this.storage = new StorageClient(this.http);
    this.functions = new FunctionsClient(this.http);
    this.realtime = new RealtimeClient(baseUrl, key, this.http.token, options.realtime);
    this.auth.onAuthStateChange((event) => {
      if (event !== 'INITIAL_SESSION') void this.realtime.updateToken();
    });
  }

  /** Starts a query on a table (or view): `nomi.from('task').select('*')`. */
  from<T extends keyof Tables<DB> & string>(table: T): QueryBuilder<Tables<DB>[T]> {
    return new QueryBuilder<Tables<DB>[T]>(this.http, `/rest/${encodeURIComponent(table)}`);
  }

  /** Calls a SQL function from the migrations, exposed at `/rest/rpc/<name>`. */
  rpc<F extends keyof Rpcs<DB> & string>(name: F, args?: Rpcs<DB>[F]['Args']): Promise<Result<Rpcs<DB>[F]['Returns']>> {
    return request(this.http, `/rest/rpc/${encodeURIComponent(name)}`, { method: 'POST', body: args ?? {} });
  }

  /** Stops background work (token refresh, realtime). For tests and short-lived server clients. */
  dispose(): void {
    this.auth.dispose();
    this.realtime.disconnect();
  }
}

/**
 * ```ts
 * import { createClient } from 'nomi-relay';
 * import type { Database } from '$lib/nomi.types';
 * export const nomi = createClient<Database>(import.meta.env.PUBLIC_NOMI_URL, import.meta.env.PUBLIC_NOMI_KEY);
 * ```
 */
export function createClient<DB extends DatabaseDef = AnyDatabase>(url: string, key: string, options?: ClientOptions): NomiClient<DB> {
  return new NomiClient<DB>(url, key, options);
}
