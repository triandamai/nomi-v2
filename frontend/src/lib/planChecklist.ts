// Reads a plan's markdown for the chat's draft bubble: its task-list items, or failing that its
// opening lines.

export interface ChecklistItem {
	text: string;
	status: 'done' | 'in_progress' | 'pending';
}

const TASK = /^\s*[-*+]\s+\[( |x|X)\]\s+(.+?)\s*$/;

/** `- [x] Pick dates` style items. The first unfinished one is the current step. */
export function planChecklist(markdown: string): ChecklistItem[] {
	const items: ChecklistItem[] = [];
	let currentFound = false;
	for (const line of markdown.split('\n')) {
		const match = TASK.exec(line);
		if (!match) continue;
		const done = match[1] !== ' ';
		let status: ChecklistItem['status'] = done ? 'done' : 'pending';
		if (!done && !currentFound) {
			status = 'in_progress';
			currentFound = true;
		}
		items.push({ text: stripInline(match[2]), status });
	}
	return items;
}

function stripInline(text: string): string {
	return text
		.replace(/\[([^\]]+)\]\([^)]*\)/g, '$1')
		.replace(/[*_`~]/g, '')
		.trim();
}

/** The plan's first lines of prose (headings skipped), for a plan with no checklist. */
export function planExcerpt(markdown: string, maxChars = 220): string {
	const prose = markdown
		.split('\n')
		.map((line) => line.trim())
		.filter((line) => line && !line.startsWith('#') && !line.startsWith('|') && !/^[-*_]{3,}$/.test(line))
		.map((line) => stripInline(line.replace(/^[-*+]\s+|^\d+\.\s+/, '')))
		.join(' ');
	return prose.length > maxChars ? `${prose.slice(0, maxChars - 1).trimEnd()}…` : prose;
}
