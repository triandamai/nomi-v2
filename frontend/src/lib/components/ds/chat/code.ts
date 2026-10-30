// Chat responses → Code and Diff. Highlighting happens before the component (the syntax colours
// are the nm-code-* tokens); a block without tokens shows its lines plain.

export type CodeKind = 'keyword' | 'string' | 'fn' | 'number' | 'type' | 'comment' | 'punct';
export type CodeToken = { text: string; kind?: CodeKind };
export type CodeLine = CodeToken[];

export const plainLines = (code: string): CodeLine[] => code.split('\n').map((text) => (text ? [{ text }] : []));

export type DiffLine = { kind: 'context' | 'add' | 'del'; old?: number; new?: number; text: string };
export type DiffHunk = { header: string; lines: DiffLine[] };

export function diffStats(hunks: DiffHunk[]): { added: number; removed: number } {
	const lines = hunks.flatMap((hunk) => hunk.lines);
	return { added: lines.filter((line) => line.kind === 'add').length, removed: lines.filter((line) => line.kind === 'del').length };
}
