import type { NomiError } from './errors.js';

/** One table in the generated `Database` type: what a row looks like, and what insert and update accept. */
export interface TableDef {
  Row: object;
  Insert: object;
  Update: object;
}

/** One SQL function exposed at `/rest/rpc/<name>`. */
export interface RpcDef {
  Args: object;
  Returns: unknown;
}

/** The shape of `src/lib/nomi.types.ts`, which Nomi writes after each migration. */
export interface DatabaseDef {
  tables: Record<string, TableDef>;
  functions: Record<string, RpcDef>;
}

/** Used when an app doesn't pass its generated types: any table, any column. */
export interface AnyDatabase {
  tables: Record<string, { Row: Record<string, any>; Insert: Record<string, any>; Update: Record<string, any> }>;
  functions: Record<string, { Args: Record<string, any>; Returns: any }>;
}

/** Every call resolves to this instead of throwing: check `error` first, then use `data`. */
export type Result<T> =
  | { data: T; error: null; status: number; count?: number | null }
  | { data: null; error: NomiError; status: number; count?: null };

export type Fetch = typeof fetch;
