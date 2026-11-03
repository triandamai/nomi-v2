import { describe, expect, it } from 'vitest';
import { highlight } from './codeTokens';

const text = (lines: ReturnType<typeof highlight>) => lines.map((line) => line.map((token) => token.text).join('')).join('\n');
const kinds = (lines: ReturnType<typeof highlight>) =>
	lines
		.flat()
		.filter((token) => token.kind)
		.map((token) => [token.kind, token.text.trim()]);

describe('highlight', () => {
	it('never changes the code, so copying gives it back exactly', () => {
		const code = 'export function weeklyBudget(total: number, weeks = 4) {\n  // one less GoFood\n  return `${total}` + "x" /* note */\n}\n';
		expect(text(highlight(code, 'typescript'))).toBe(code);
	});

	it('colours keywords, calls, numbers, strings and comments', () => {
		const found = kinds(highlight("const base = Math.round(total / 4) // per week\nlet s = 'hi'", 'ts'));
		expect(found).toContainEqual(['keyword', 'const']);
		expect(found).toContainEqual(['fn', 'round']);
		expect(found).toContainEqual(['number', '4']);
		expect(found).toContainEqual(['comment', '// per week']);
		expect(found).toContainEqual(['string', "'hi'"]);
	});

	it('reads comments the way the language writes them', () => {
		expect(kinds(highlight('x = 1  # note', 'python'))).toContainEqual(['comment', '# note']);
		expect(kinds(highlight('SELECT 1 -- note', 'sql'))).toContainEqual(['comment', '-- note']);
		expect(kinds(highlight('a /* one\ntwo */ b', 'js')).filter(([kind]) => kind === 'comment')).toEqual([
			['comment', '/* one'],
			['comment', 'two */'],
		]);
	});
});
