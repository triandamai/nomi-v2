import type { Cookies } from '@sveltejs/kit';

// For the sign-in pages' tests: a request event with a cookie jar and a backend whose answers the
// test decides, recording every call the page makes.

export type BackendCall = { url: string; method: string; body: unknown; headers: Record<string, string> };
type Answer = Response | 'unreachable';

/** A backend that answers each call with `answer(call)` and remembers the calls. */
export function fakeBackend(answer: (call: BackendCall) => Answer | Promise<Answer>) {
	const calls: BackendCall[] = [];
	const fetchFn = async (input: RequestInfo | URL, init?: RequestInit): Promise<Response> => {
		const headers = Object.fromEntries(new Headers(init?.headers).entries());
		const call = { url: String(input), method: init?.method ?? 'GET', body: init?.body ? JSON.parse(String(init.body)) : undefined, headers };
		calls.push(call);
		const reply = await answer(call);
		if (reply === 'unreachable') throw new TypeError('fetch failed');
		return reply;
	};
	return { calls, fetch: fetchFn as typeof fetch };
}

/** A JSON answer. */
export const json = (body: unknown, status = 200) => Response.json(body, { status });

export type FakeEvent = ReturnType<typeof fakeEvent>;

export function fakeEvent(options: {
	url?: string;
	form?: Record<string, string>;
	cookies?: Record<string, string>;
	locals?: Partial<App.Locals>;
	params?: Record<string, string>;
	fetch?: typeof fetch;
}) {
	const jar = new Map(Object.entries(options.cookies ?? {}));
	const set = new Map<string, Record<string, unknown>>();
	const cookies = {
		get: (name: string) => jar.get(name),
		getAll: () => [...jar].map(([name, value]) => ({ name, value })),
		set: (name: string, value: string, opts: Record<string, unknown>) => {
			jar.set(name, value);
			set.set(name, opts);
		},
		delete: (name: string) => void jar.delete(name),
		serialize: () => '',
	} as unknown as Cookies;
	const url = new URL(options.url ?? 'http://localhost/');
	let request: Request;
	if (options.form) {
		const body = new FormData();
		for (const [key, value] of Object.entries(options.form)) body.set(key, value);
		request = new Request(url, { method: 'POST', body });
	} else {
		request = new Request(url);
	}
	const fetchFn = options.fetch ?? (async () => new Response(null, { status: 599 }));
	return {
		url,
		request,
		cookies,
		/** What the page left in the cookie jar, and how it set each. */
		jar,
		cookieOptions: set,
		locals: { ...options.locals } as App.Locals,
		params: options.params ?? {},
		fetch: fetchFn,
	};
}

/** What a load or action threw (SvelteKit's redirect), for `expect(...).toMatchObject`. */
export async function thrown(run: () => unknown): Promise<unknown> {
	try {
		await run();
	} catch (error) {
		return error;
	}
	throw new Error('Expected it to throw (a redirect), but it returned');
}
