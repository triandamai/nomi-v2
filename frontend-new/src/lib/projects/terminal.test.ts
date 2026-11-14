import { describe, expect, it } from 'vitest';
import { appendOutput, capLog, terminalLines } from './terminal';

describe('the runtime’s log', () => {
	it('drops the blank screenful Vite prints before clearing the screen', () => {
		const vite = `${'\n'.repeat(40)}\u001b[1;1H\u001b[0J  VITE v8 ready\n`;
		expect(appendOutput('200 GET /\n', vite)).toBe('200 GET /\n  VITE v8 ready\n');
		expect(appendOutput('', `\n\n\n\u001b[2J\u001b[3J\u001b[Hready\n`)).toBe('ready\n');
	});

	it('collapses long runs of blank lines but keeps single gaps', () => {
		expect(appendOutput('a\n', '\n\n\n\nb\n')).toBe('a\n\nb\n');
		expect(appendOutput('a\n', '\nb\n')).toBe('a\n\nb\n');
	});

	it('lets a carriage return redraw the line, across chunks and within one', () => {
		let log = 'start\n';
		log = appendOutput(log, 'transforming (67) utils.js\r');
		log = appendOutput(log, 'transforming (145) client.js\r');
		log = appendOutput(log, '✓ built in 473ms\n');
		expect(log).toBe('start\n✓ built in 473ms\n');
		expect(appendOutput('', '⠋ installing\r⠙ installing\rdone\r\n')).toBe('done\n');
	});

	it('strips colour codes', () => {
		expect(appendOutput('', '\u001b[32m✓\u001b[39m built in 1s\n')).toBe('✓ built in 1s\n');
	});

	it('keeps the end of a long log, from a whole line', () => {
		expect(capLog('one\ntwo\nthree\n', 9)).toBe('three\n');
		expect(capLog('short', 100)).toBe('short');
	});
});

describe('the terminal’s lines', () => {
	it('tells commands, output, warnings, errors and success apart', () => {
		const lines = terminalLines('$ npm install\nadded 12 packages\nnpm warn deprecated x\n\n> vite dev\n✓ ready in 300ms\nError: Cannot find module');
		expect(lines.map((line) => line.kind)).toEqual(['command', 'out', 'warn', 'blank', 'command', 'ok', 'error']);
		expect(lines[4].text).toBe('$ vite dev');
	});

	it('drops colour codes and keeps the last lines only', () => {
		expect(terminalLines('\x1b[32mok\x1b[0m')[0].text).toBe('ok');
		expect(terminalLines('a\nb\nc', 2).map((line) => line.text)).toEqual(['b', 'c']);
	});

	it('has no lines for an empty log', () => {
		expect(terminalLines('')).toEqual([]);
		expect(terminalLines('ready\n')).toEqual([{ kind: 'out', text: 'ready' }]);
	});
});
