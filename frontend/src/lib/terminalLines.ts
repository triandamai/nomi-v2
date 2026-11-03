// The preview's log as the design system's terminal lines (Build → Terminal): the commands Koda
// ran, what they printed, and errors and warnings coloured as such.
export type TerminalLine = { kind: 'command' | 'out' | 'ok' | 'fail' | 'error' | 'warn' | 'blank'; text?: string };

export function terminalLines(log: string, max = 400): TerminalLine[] {
	const lines = log.replace(/\r\n?/g, '\n').split('\n');
	return lines.slice(Math.max(0, lines.length - max)).map((raw): TerminalLine => {
		const text = raw.replace(/\x1b\[[0-9;]*m/g, '');
		if (!text.trim()) return { kind: 'blank' };
		if (/^\s*[$>❯]\s/.test(text)) return { kind: 'command', text: text.replace(/^\s*[$>❯]\s/, '$ ') };
		if (/\b(npm ERR!|error|failed|exception)\b/i.test(text)) return { kind: 'error', text };
		if (/\bwarn(ing)?\b/i.test(text)) return { kind: 'warn', text };
		if (/^\s*(✓|✔)|\b(ready in|compiled successfully|done in)\b/i.test(text)) return { kind: 'ok', text };
		return { kind: 'out', text };
	});
}
