import { NomiClient } from '../client.js';
import type { AnyDatabase, DatabaseDef } from '../types.js';

/** The gateway checks the caller's token, then passes its claims to the functions service in this header. */
export const CLAIMS_HEADER = 'x-nomi-claims';

const FUNCTION_MARK = Symbol.for('nomi.function');

/** The token claims the gateway checked: `sub` is the user's ID. */
export interface Claims {
  sub?: string;
  role: 'anon' | 'authenticated' | 'service';
  email?: string;
  exp?: number;
  [claim: string]: unknown;
}

export interface FunctionUser {
  id: string;
  email: string | null;
  role: Claims['role'];
}

export interface FunctionContext<DB extends DatabaseDef = AnyDatabase> {
  /** The caller, or `null` for an anonymous call (only possible with `"auth": "public"`). */
  user: FunctionUser | null;
  claims: Claims;
  /** Acts as the caller, so row-level security still applies. Use this by default. */
  nomi: NomiClient<DB>;
  /** Acts with the service key and bypasses row-level security. Only for steps that truly need it. */
  admin: NomiClient<DB>;
  /** The project's secrets, by name. */
  env: Record<string, string | undefined>;
}

export type FunctionHandler<DB extends DatabaseDef = AnyDatabase> = (
  req: Request,
  ctx: FunctionContext<DB>
) => Response | Promise<Response>;

export type NomiFunction<DB extends DatabaseDef = AnyDatabase> = FunctionHandler<DB> & { readonly [FUNCTION_MARK]: true };

/**
 * ```ts
 * // nomi/functions/place-order.ts
 * import { defineFunction } from 'nomi-relay/functions';
 * export default defineFunction(async (req, ctx) => Response.json({ hello: ctx.user?.id }));
 * ```
 */
export function defineFunction<DB extends DatabaseDef = AnyDatabase>(handler: FunctionHandler<DB>): NomiFunction<DB> {
  return Object.assign(handler, { [FUNCTION_MARK]: true as const });
}

export function isNomiFunction(value: unknown): value is NomiFunction {
  return typeof value === 'function' && (value as Partial<NomiFunction>)[FUNCTION_MARK] === true;
}

/** Reads the environment on Deno or Node. */
export function readEnv(): Record<string, string | undefined> {
  const deno = (globalThis as { Deno?: { env: { toObject(): Record<string, string> } } }).Deno;
  if (deno) return deno.env.toObject();
  const proc = (globalThis as { process?: { env: Record<string, string | undefined> } }).process;
  return proc ? { ...proc.env } : {};
}

function decodeClaims(header: string | null): Claims {
  if (!header) return { role: 'anon' };
  try {
    const json = atob(header.replace(/-/g, '+').replace(/_/g, '/'));
    const bytes = Uint8Array.from(json, (char) => char.charCodeAt(0));
    return JSON.parse(new TextDecoder().decode(bytes)) as Claims;
  } catch {
    return { role: 'anon' };
  }
}

/**
 * Builds the `ctx` a handler gets, from the request the gateway forwarded. The functions router
 * calls this; it's exported for `nomi-relay functions serve` and for tests.
 */
export function createFunctionContext<DB extends DatabaseDef = AnyDatabase>(
  req: Request,
  env: Record<string, string | undefined> = readEnv()
): FunctionContext<DB> {
  const url = env.NOMI_URL;
  const anonKey = env.NOMI_ANON_KEY;
  const serviceKey = env.NOMI_SERVICE_KEY;
  if (!url || !anonKey || !serviceKey) {
    throw new Error('nomi-relay: NOMI_URL, NOMI_ANON_KEY and NOMI_SERVICE_KEY must be set for functions');
  }
  const claims = decodeClaims(req.headers.get(CLAIMS_HEADER));
  const bearer = req.headers.get('authorization')?.replace(/^Bearer\s+/i, '');
  const user = claims.sub ? { id: claims.sub, email: claims.email ?? null, role: claims.role } : null;
  return {
    user,
    claims,
    nomi: new NomiClient<DB>(url, anonKey, user && bearer ? { accessToken: bearer } : { auth: { persistSession: false, autoRefreshToken: false } }),
    admin: new NomiClient<DB>(url, serviceKey, { accessToken: serviceKey }),
    env
  };
}
