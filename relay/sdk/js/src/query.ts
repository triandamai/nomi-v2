import { NomiError } from './errors.js';
import { readBody, send, type HttpContext } from './http.js';
import type { Result, TableDef } from './types.js';

type Column<Row> = keyof Row & string;
type Method = 'GET' | 'POST' | 'PATCH' | 'DELETE';
type Cardinality = 'many' | 'one' | 'maybe';

/** PostgREST answers this media type with a single object instead of an array. */
const SINGLE_OBJECT_MEDIA_TYPE = 'application/vnd.pgrst.object+json';

/** Characters that make a value inside `in.(...)` or `or=(...)` need double quotes. */
const RESERVED_IN_LIST = /[,()"\\\s:]/;

export type FilterOperator =
  | 'eq'
  | 'neq'
  | 'gt'
  | 'gte'
  | 'lt'
  | 'lte'
  | 'like'
  | 'ilike'
  | 'is'
  | 'in'
  | 'cs'
  | 'cd'
  | 'fts'
  | 'plfts'
  | 'wfts';

export interface OrderOptions {
  ascending?: boolean;
  nullsFirst?: boolean;
}

export interface CountOptions {
  /** Ask for the total number of matching rows; it comes back in `count`. */
  count?: 'exact' | 'planned' | 'estimated';
}

function formatValue(value: unknown): string {
  if (value === null) return 'null';
  if (value instanceof Date) return value.toISOString();
  if (typeof value === 'object') return JSON.stringify(value);
  return String(value);
}

function formatListItem(value: unknown): string {
  const text = formatValue(value);
  return RESERVED_IN_LIST.test(text) ? `"${text.replace(/\\/g, '\\\\').replace(/"/g, '\\"')}"` : text;
}

/**
 * Builds one PostgREST request. Chain filters and modifiers, then `await` it:
 * `await nomi.from('task').select('*').eq('done', false).order('created_at')`.
 */
export class QueryBuilder<T extends TableDef, Out = T['Row'][]> implements PromiseLike<Result<Out>> {
  private method: Method = 'GET';
  private readonly params = new URLSearchParams();
  private readonly orders: string[] = [];
  private readonly prefer: string[] = [];
  private readonly headers: Record<string, string> = {};
  private body: unknown;
  private cardinality: Cardinality = 'many';
  private returnsRows = true;

  constructor(
    private readonly http: HttpContext,
    private readonly path: string
  ) {}

  /** Read rows, or (after insert/update/upsert/delete) ask for the changed rows back. */
  select(columns = '*', options: CountOptions = {}): QueryBuilder<T, T['Row'][]> {
    this.params.set('select', columns.replace(/\s+/g, ''));
    if (options.count) this.prefer.push(`count=${options.count}`);
    if (this.method !== 'GET') this.returnsRows = true;
    return this as unknown as QueryBuilder<T, T['Row'][]>;
  }

  insert(values: T['Insert'] | T['Insert'][], options: CountOptions = {}): QueryBuilder<T, null> {
    return this.write('POST', values, options);
  }

  /** Insert, or update the row that already has the same key (or `onConflict` columns). */
  upsert(
    values: T['Insert'] | T['Insert'][],
    options: CountOptions & { onConflict?: string; ignoreDuplicates?: boolean } = {}
  ): QueryBuilder<T, null> {
    this.prefer.push(`resolution=${options.ignoreDuplicates ? 'ignore' : 'merge'}-duplicates`);
    if (options.onConflict) this.params.set('on_conflict', options.onConflict);
    return this.write('POST', values, options);
  }

  /** Update every row the filters match. Always add a filter: an unfiltered update is refused by the gateway. */
  update(values: T['Update'], options: CountOptions = {}): QueryBuilder<T, null> {
    return this.write('PATCH', values, options);
  }

  /** Delete every row the filters match. Always add a filter: an unfiltered delete is refused by the gateway. */
  delete(options: CountOptions = {}): QueryBuilder<T, null> {
    return this.write('DELETE', undefined, options);
  }

  eq<K extends Column<T['Row']>>(column: K, value: T['Row'][K]): this {
    return this.filter(column, 'eq', value);
  }
  neq<K extends Column<T['Row']>>(column: K, value: T['Row'][K]): this {
    return this.filter(column, 'neq', value);
  }
  gt<K extends Column<T['Row']>>(column: K, value: T['Row'][K]): this {
    return this.filter(column, 'gt', value);
  }
  gte<K extends Column<T['Row']>>(column: K, value: T['Row'][K]): this {
    return this.filter(column, 'gte', value);
  }
  lt<K extends Column<T['Row']>>(column: K, value: T['Row'][K]): this {
    return this.filter(column, 'lt', value);
  }
  lte<K extends Column<T['Row']>>(column: K, value: T['Row'][K]): this {
    return this.filter(column, 'lte', value);
  }
  /** SQL `like`; use `*` or `%` as the wildcard. */
  like(column: Column<T['Row']>, pattern: string): this {
    return this.filter(column, 'like', pattern);
  }
  ilike(column: Column<T['Row']>, pattern: string): this {
    return this.filter(column, 'ilike', pattern);
  }
  is(column: Column<T['Row']>, value: null | boolean): this {
    return this.filter(column, 'is', value);
  }
  in<K extends Column<T['Row']>>(column: K, values: readonly T['Row'][K][]): this {
    this.params.append(column, `in.(${values.map(formatListItem).join(',')})`);
    return this;
  }
  /** Array or JSON column contains every given value. */
  contains(column: Column<T['Row']>, value: unknown): this {
    const text = Array.isArray(value) ? `{${value.map(formatListItem).join(',')}}` : formatValue(value);
    this.params.append(column, `cs.${text}`);
    return this;
  }
  /** Full-text search on a `tsvector` column. */
  textSearch(column: Column<T['Row']>, query: string, options: { type?: 'plain' | 'phrase' | 'websearch'; config?: string } = {}): this {
    const op = { plain: 'plfts', phrase: 'phfts', websearch: 'wfts' }[options.type ?? 'plain'] ?? 'fts';
    this.params.append(column, `${op}${options.config ? `(${options.config})` : ''}.${query}`);
    return this;
  }
  not(column: Column<T['Row']>, operator: FilterOperator, value: unknown): this {
    this.params.append(column, `not.${operator}.${formatValue(value)}`);
    return this;
  }
  /** Raw PostgREST `or`, e.g. `.or('done.eq.true,priority.gt.3')`. */
  or(filters: string): this {
    this.params.append('or', `(${filters})`);
    return this;
  }
  /** Any PostgREST filter by name, for operators without their own method. */
  filter(column: string, operator: FilterOperator | `not.${FilterOperator}`, value: unknown): this {
    this.params.append(column, `${operator}.${formatValue(value)}`);
    return this;
  }
  /** Match every column in `query` exactly. */
  match(query: Partial<T['Row']>): this {
    for (const [column, value] of Object.entries(query)) this.filter(column, 'eq', value);
    return this;
  }

  order(column: Column<T['Row']>, options: OrderOptions = {}): this {
    let term = `${column}.${options.ascending === false ? 'desc' : 'asc'}`;
    if (options.nullsFirst !== undefined) term += options.nullsFirst ? '.nullsfirst' : '.nullslast';
    this.orders.push(term);
    return this;
  }
  limit(count: number): this {
    this.params.set('limit', String(count));
    return this;
  }
  /** Rows `from` to `to`, both included, counting from 0. */
  range(from: number, to: number): this {
    this.params.set('offset', String(from));
    this.params.set('limit', String(to - from + 1));
    return this;
  }

  /** Exactly one row, or an error. */
  single(): QueryBuilder<T, T['Row']> {
    this.cardinality = 'one';
    this.headers.accept = SINGLE_OBJECT_MEDIA_TYPE;
    return this as unknown as QueryBuilder<T, T['Row']>;
  }
  /** One row or `null`; more than one is an error. */
  maybeSingle(): QueryBuilder<T, T['Row'] | null> {
    this.cardinality = 'maybe';
    return this as unknown as QueryBuilder<T, T['Row'] | null>;
  }

  then<A = Result<Out>, B = never>(
    onfulfilled?: ((value: Result<Out>) => A | PromiseLike<A>) | null,
    onrejected?: ((reason: unknown) => B | PromiseLike<B>) | null
  ): PromiseLike<A | B> {
    return this.execute().then(onfulfilled, onrejected);
  }

  private write(method: Method, body: unknown, options: CountOptions): QueryBuilder<T, null> {
    this.method = method;
    this.body = body;
    this.returnsRows = false;
    if (options.count) this.prefer.push(`count=${options.count}`);
    return this as unknown as QueryBuilder<T, null>;
  }

  private async execute(): Promise<Result<Out>> {
    const query = new URLSearchParams(this.params);
    if (this.orders.length) query.set('order', this.orders.join(','));
    const prefer = [...this.prefer];
    if (this.method !== 'GET') prefer.push(this.returnsRows ? 'return=representation' : 'return=minimal');
    const headers = { ...this.headers };
    if (prefer.length) headers.prefer = prefer.join(',');

    const { response, error } = await send(this.http, this.path, {
      method: this.method,
      query,
      headers,
      body: this.body
    });
    if (error || !response) return { data: null, error: error!, status: response?.status ?? 0, count: null };

    const count = countFrom(response.headers.get('content-range'));
    let data = (this.returnsRows ? await readBody(response) : null) as unknown;
    if (this.cardinality === 'maybe' && Array.isArray(data)) {
      if (data.length > 1) {
        return {
          data: null,
          error: new NomiError(`Expected at most one row, got ${data.length}`, response.status, 'multiple_rows'),
          status: response.status,
          count: null
        };
      }
      data = data[0] ?? null;
    }
    return { data: data as Out, error: null, status: response.status, count };
  }
}

/** `Content-Range: 0-9/42` → 42. `*` (unknown) or a missing header → null. */
function countFrom(header: string | null): number | null {
  const total = header?.split('/')[1];
  if (!total || total === '*') return null;
  const parsed = Number(total);
  return Number.isFinite(parsed) ? parsed : null;
}
