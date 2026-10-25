import { describe, expect, it } from 'vitest';
import { ANON_KEY, client, mockFetch } from './helpers.js';

describe('from().select()', () => {
  it('builds a PostgREST read with filters, order and range', async () => {
    const { fetch, calls } = mockFetch({ body: [{ id: 1 }], headers: { 'content-range': '0-0/12' } });
    const nomi = client(fetch);

    const result = await nomi
      .from('task')
      .select('id, title', { count: 'exact' })
      .eq('done', false)
      .in('tag', ['a', 'b c'])
      .order('created_at', { ascending: false })
      .order('id')
      .range(10, 19);

    expect(result).toEqual({ data: [{ id: 1 }], error: null, status: 200, count: 12 });
    const call = calls[0]!;
    expect(call.url.pathname).toBe('/rest/task');
    expect(call.method).toBe('GET');
    expect(call.url.searchParams.get('select')).toBe('id,title');
    expect(call.url.searchParams.get('done')).toBe('eq.false');
    expect(call.url.searchParams.get('tag')).toBe('in.(a,"b c")');
    expect(call.url.searchParams.get('order')).toBe('created_at.desc,id.asc');
    expect(call.url.searchParams.get('offset')).toBe('10');
    expect(call.url.searchParams.get('limit')).toBe('10');
    expect(call.headers.prefer).toBe('count=exact');
    expect(call.headers.apikey).toBe(ANON_KEY);
    expect(call.headers.authorization).toBe(`Bearer ${ANON_KEY}`);
    expect(call.headers['x-nomi-client']).toMatch(/^js\//);
  });

  it('asks for one object with single()', async () => {
    const { fetch, calls } = mockFetch({ body: { id: 7 } });
    const { data } = await client(fetch).from('task').select().eq('id', 7).single();
    expect(data).toEqual({ id: 7 });
    expect(calls[0]!.headers.accept).toBe('application/vnd.pgrst.object+json');
  });

  it('maybeSingle() gives null for no rows and an error for many', async () => {
    const none = mockFetch({ body: [] });
    expect((await client(none.fetch).from('task').select().maybeSingle()).data).toBeNull();

    const many = mockFetch({ body: [{ id: 1 }, { id: 2 }] });
    const result = await client(many.fetch).from('task').select().maybeSingle();
    expect(result.error?.code).toBe('multiple_rows');
  });

  it('returns PostgREST errors without throwing', async () => {
    const { fetch } = mockFetch({ status: 400, body: { message: 'column task.nope does not exist', code: '42703' } });
    const result = await client(fetch).from('task').select('nope');
    expect(result.data).toBeNull();
    expect(result.error?.message).toBe('column task.nope does not exist');
    expect(result.error?.code).toBe('42703');
    expect(result.status).toBe(400);
  });

  it('reports a network failure with status 0', async () => {
    const fetch = (async () => {
      throw new TypeError('fetch failed');
    }) as unknown as typeof globalThis.fetch;
    const result = await client(fetch).from('task').select();
    expect(result.error?.code).toBe('network_error');
    expect(result.status).toBe(0);
  });
});

describe('writes', () => {
  it('insert sends JSON and returns nothing unless select() is chained', async () => {
    const { fetch, calls } = mockFetch({ status: 201 });
    const result = await client(fetch).from('task').insert({ title: 'a' });
    expect(result.data).toBeNull();
    expect(calls[0]!.method).toBe('POST');
    expect(calls[0]!.body).toEqual({ title: 'a' });
    expect(calls[0]!.headers.prefer).toBe('return=minimal');
  });

  it('insert().select() returns the new rows', async () => {
    const { fetch, calls } = mockFetch({ status: 201, body: [{ id: 1, title: 'a' }] });
    const { data } = await client(fetch).from('task').insert({ title: 'a' }).select();
    expect(data).toEqual([{ id: 1, title: 'a' }]);
    expect(calls[0]!.headers.prefer).toBe('return=representation');
  });

  it('upsert merges on conflict', async () => {
    const { fetch, calls } = mockFetch({ status: 201 });
    await client(fetch).from('task').upsert([{ id: 1 }], { onConflict: 'id' });
    expect(calls[0]!.headers.prefer).toBe('resolution=merge-duplicates,return=minimal');
    expect(calls[0]!.url.searchParams.get('on_conflict')).toBe('id');
  });

  it('update and delete carry their filters', async () => {
    const { fetch, calls } = mockFetch({ status: 204 });
    const nomi = client(fetch);
    await nomi.from('task').update({ done: true }).eq('id', 3);
    await nomi.from('task').delete().eq('id', 3);
    expect(calls.map((call) => call.method)).toEqual(['PATCH', 'DELETE']);
    expect(calls.every((call) => call.url.searchParams.get('id') === 'eq.3')).toBe(true);
  });

  it('rpc posts the arguments', async () => {
    const { fetch, calls } = mockFetch({ body: 42 });
    const { data } = await client(fetch).rpc('count_open', { owner: 'u' });
    expect(data).toBe(42);
    expect(calls[0]!.url.pathname).toBe('/rest/rpc/count_open');
    expect(calls[0]!.body).toEqual({ owner: 'u' });
  });
});
