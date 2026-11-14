// The runtime's output (Code, terminal & phone preview uR6dq, steps 3–4): a growing log kept as
// a terminal would show it, and its lines in the design system's Terminal's kinds.

export type TerminalLine = { kind: 'command' | 'out' | 'ok' | 'fail' | 'error' | 'warn' | 'blank'; text?: string };

// eslint-disable-next-line no-control-regex
const ANSI = /\u001b\[[0-9;?]*[ -/]*[@-~]|\u001b\][^\u0007]*\u0007|\u001b[()][A-Z0-9]/g;
// Clearing the screen: a jump to the top, erase-display or a reset. Vite clears by printing a
// screenful of blank lines first, so those go along with it.
// eslint-disable-next-line no-control-regex
const CLEAR = /\u001b\[(?:\d*;\d*)?[Hf]|\u001b\[[0-3]?J|\u001bc/;

/** A segment added to the log: colour codes dropped, a lone `\r` redrawing the line it's on. */
function appendSegment(log: string, segment: string): string {
	const normalized = segment.replace(ANSI, '').replace(/\r\n/g, '\n');
	let combined = log;
	for (const ch of normalized) {
		if (ch === '\r') {
			if (!combined.endsWith('\r')) combined += '\r';
		} else if (ch === '\n') {
			if (combined.endsWith('\r')) combined = combined.slice(0, -1);
			combined += '\n';
		} else {
			if (combined.endsWith('\r')) {
				combined = combined.slice(0, -1);
				const newline = combined.lastIndexOf('\n');
				combined = newline >= 0 ? combined.slice(0, newline + 1) : '';
			}
			combined += ch;
		}
	}
	return combined;
}

function dropTrailingBlankLines(log: string): string {
	const trimmed = log.replace(/(?:\n[ \t]*)+$/, '');
	return trimmed ? `${trimmed}\n` : '';
}

/** Adds a chunk of output to the log. A screen clear keeps the history; long blank runs become one. */
export function appendOutput(log: string, chunk: string): string {
	const segments = chunk.split(CLEAR);
	let next = appendSegment(log, segments[0]);
	for (const segment of segments.slice(1)) next = appendSegment(dropTrailingBlankLines(next), segment);
	return next.replace(/\n(?:[ \t]*\n){2,}/g, '\n\n');
}

/** The last `max` characters of a growing log, from a whole line. */
export function capLog(log: string, max = 200_000): string {
	if (log.length <= max) return log;
	const cut = log.slice(log.length - max);
	const newline = cut.indexOf('\n');
	return newline >= 0 ? cut.slice(newline + 1) : cut;
}

/** The log's last `max` lines as the Terminal shows them: commands, output, warnings, errors, success. */
export function terminalLines(log: string, max = 400): TerminalLine[] {
	const lines = log.replace(/\r\n?/g, '\n').replace(/\n$/, '').split('\n');
	if (lines.length === 1 && !lines[0]) return [];
	return lines.slice(Math.max(0, lines.length - max)).map((raw): TerminalLine => {
		const text = raw.replace(ANSI, '');
		if (!text.trim()) return { kind: 'blank' };
		if (/^\s*[$>❯]\s/.test(text)) return { kind: 'command', text: text.replace(/^\s*[$>❯]\s/, '$ ') };
		if (/\b(npm ERR!|error|failed|exception)\b/i.test(text)) return { kind: 'error', text };
		if (/\bwarn(ing)?\b/i.test(text)) return { kind: 'warn', text };
		if (/^\s*(✓|✔)|\b(ready in|compiled successfully|done in)\b/i.test(text)) return { kind: 'ok', text };
		return { kind: 'out', text };
	});
}
