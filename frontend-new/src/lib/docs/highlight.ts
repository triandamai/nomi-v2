import type { CodeKind, CodeLine } from '$lib/ds/chat/code';

// Just enough highlighting for the docs' Svelte and TypeScript examples, in the nm-code-* tokens
// the chat's code blocks use: components as types, attributes as functions, strings, numbers,
// comments, keywords inside <script> and {expressions}. Not a parser: keep examples plain.

const KEYWORDS = /^(?:import|from|export|const|let|var|function|return|if|else|for|of|in|as|type|interface|new|typeof|await|async|true|false|null|undefined|class|satisfies)\b/;
const RUNES = /^\$(?:props|state|derived|effect|bindable|inspect|host)\b/;

export function highlight(code: string, language: 'svelte' | 'ts' = 'svelte'): CodeLine[] {
	let inScript = language === 'ts';
	let openingScript = false;
	let depth = 0;
	return code.split('\n').map((source) => {
		const line: CodeLine = [];
		const push = (text: string, kind?: CodeKind) => {
			const last = line.at(-1);
			if (!kind && last && !last.kind) last.text += text;
			else line.push(kind ? { text, kind } : { text });
		};
		let rest = source;
		while (rest) {
			const code = inScript || depth > 0;
			const atStart = !line.some((token) => token.text.trim());
			let match: RegExpMatchArray | null;
			if ((match = rest.match(/^<!--.*?(?:-->|$)/))) push(match[0], 'comment');
			else if ((code || atStart) && (match = rest.match(/^\/\/.*/))) push(match[0], 'comment');
			else if ((match = rest.match(/^(?:"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'|`(?:\\.|[^`\\])*`)/))) push(match[0], 'string');
			else if ((match = rest.match(/^\{[#:/@][a-z]+/))) {
				push(match[0], 'keyword');
				depth++;
			} else if ((match = rest.match(/^<\/([A-Za-z][\w.:-]*)>/))) {
				push(match[0], /^[A-Z]/.test(match[1]) ? 'type' : 'punct');
				if (match[1] === 'script') inScript = false;
			} else if (!code && (match = rest.match(/^<([A-Za-z][\w.:-]*)/))) {
				push(match[0], /^[A-Z]/.test(match[1]) ? 'type' : 'punct');
				openingScript = match[1] === 'script';
			} else if (!code && (match = rest.match(/^\/?>/))) {
				push(match[0], 'punct');
				inScript = openingScript && match[0] === '>';
				openingScript = false;
			}
			else if ((match = rest.match(/^[\w:-]+(?==)/)) && !code) push(match[0], 'fn');
			else if ((match = rest.match(/^=>?/))) push(match[0], 'punct');
			else if ((match = rest.match(/^[{}]/))) {
				push(match[0], 'punct');
				if (!inScript) depth = Math.max(0, depth + (match[0] === '{' ? 1 : -1));
			} else if ((match = rest.match(/^[;()[\]]/))) push(match[0], 'punct');
			else if (code && (match = rest.match(/^\d+(?:\.\d+)?\b/))) push(match[0], 'number');
			else if (code && (match = rest.match(RUNES))) push(match[0], 'fn');
			else if (code && (match = rest.match(KEYWORDS))) push(match[0], 'keyword');
			else if ((match = rest.match(/^(?:[A-Za-z_$][\w$]*|\s+|.)/))) push(match[0]);
			rest = rest.slice(match![0].length);
		}
		return line;
	});
}
