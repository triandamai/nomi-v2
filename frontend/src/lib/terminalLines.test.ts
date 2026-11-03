import { describe, expect, it } from 'vitest';
import { terminalLines } from './terminalLines';

describe('terminalLines', () => {
	it('tells commands, output, warnings, errors and success apart', () => {
		const lines = terminalLines('$ npm install\nadded 12 packages\nnpm warn deprecated x\n\n> vite dev\n✓ ready in 300ms\nError: Cannot find module');
		expect(lines.map((line) => line.kind)).toEqual(['command', 'out', 'warn', 'blank', 'command', 'ok', 'error']);
		expect(lines[4].text).toBe('$ vite dev');
	});
	it('drops colour codes', () => {
		expect(terminalLines('\x1b[32mok\x1b[0m')[0].text).toBe('ok');
	});
});
