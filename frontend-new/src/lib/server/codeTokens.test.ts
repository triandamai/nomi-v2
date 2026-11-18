import { describe, expect, it } from 'vitest';
import { highlightLines, kindOf, languageOf, MAX_HIGHLIGHT_CHARS } from './codeTokens';

const text = (lines: { text: string }[][]) => lines.map((line) => line.map((token) => token.text).join(''));

describe('the code view’s highlighting', () => {
	it('knows a file’s language from its name', () => {
		expect(languageOf('src/routes/+page.svelte')).toBe('svelte');
		expect(languageOf('src/lib/api.ts')).toBe('typescript');
		expect(languageOf('package.json')).toBe('json');
		expect(languageOf('Dockerfile')).toBe('dockerfile');
		expect(languageOf('notes.txt')).toBeNull();
		expect(languageOf('README')).toBeNull();
	});

	it('names what a scope is in the code view’s seven colours', () => {
		expect(kindOf(['source.ts', 'storage.type.ts'])).toBe('keyword');
		expect(kindOf(['source.ts', 'keyword.control.flow.ts'])).toBe('keyword');
		expect(kindOf(['source.ts', 'string.quoted.single.ts'])).toBe('string');
		expect(kindOf(['source.ts', 'meta.function-call.ts', 'entity.name.function.ts'])).toBe('fn');
		expect(kindOf(['source.ts', 'constant.numeric.decimal.ts'])).toBe('number');
		expect(kindOf(['source.ts', 'entity.name.type.interface.ts'])).toBe('type');
		expect(kindOf(['source.ts', 'comment.line.double-slash.ts'])).toBe('comment');
		expect(kindOf(['source.ts', 'punctuation.terminator.statement.ts'])).toBe('punct');
		expect(kindOf(['source.ts', 'variable.other.readwrite.ts'])).toBeUndefined();
	});

	it('highlights a file and keeps every character and line', async () => {
		const code = "// the price\nconst price: number = 249000;\nexport function total(n: number) {\n\treturn n * price;\n}\n";
		const lines = await highlightLines(code, 'src/lib/total.ts');
		expect(text(lines)).toEqual(code.split('\n'));
		const kinds = lines.flat().map((token) => (token as { kind?: string }).kind);
		expect(kinds).toContain('comment');
		expect(kinds).toContain('keyword');
		expect(kinds).toContain('number');
		expect(kinds).toContain('fn');
	});

	it('shows a file it can’t highlight as plain lines', async () => {
		expect(await highlightLines('just words\nmore words', 'notes.txt')).toEqual([[{ text: 'just words' }], [{ text: 'more words' }]]);
		const big = 'x'.repeat(MAX_HIGHLIGHT_CHARS + 1);
		expect(await highlightLines(big, 'big.ts')).toEqual([[{ text: big }]]);
	});
});
