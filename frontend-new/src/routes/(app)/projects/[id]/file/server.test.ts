import { describe, expect, it, vi } from 'vitest';

const projectFile = vi.fn();
vi.mock('$lib/server/workspace', () => ({ projectFile: (...args: unknown[]) => projectFile(...args) }));

const { GET } = await import('./+server');

const get = (path: string) => GET({ url: new URL(`http://localhost/projects/p1/file?path=${encodeURIComponent(path)}`), params: { id: 'p1' } } as never) as Promise<Response>;

describe('a project file for the code view', () => {
	it('comes with its lines highlighted for the code view', async () => {
		projectFile.mockResolvedValue({ path: 'src/lib/total.ts', content: 'const price = 249000;', binary: false });
		const body = await (await get('src/lib/total.ts')).json();
		expect(body.content).toBe('const price = 249000;');
		const tokens = body.lines[0] as { text: string; kind?: string }[];
		expect(tokens.map((token) => token.text).join('')).toBe('const price = 249000;');
		expect(tokens.find((token) => token.text === 'const')?.kind).toBe('keyword');
		expect(tokens.find((token) => token.text === '249000')?.kind).toBe('number');
	});

	it('sends a binary file without lines', async () => {
		projectFile.mockResolvedValue({ path: 'logo.png', binary: true });
		expect((await (await get('logo.png')).json()).lines).toBeUndefined();
	});

	it('refuses a path that leaves the project', async () => {
		expect((await get('../secrets')).status).toBe(400);
	});
});
