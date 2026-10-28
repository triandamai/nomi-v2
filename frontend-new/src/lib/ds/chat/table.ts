// Chat responses → Table: a cell is its text, or its text with a tone when the number is good or
// bad news (spending up is bad). Copy gives tab-separated rows (pastes into a sheet), download a
// CSV, both with the header and the total row.

export type Cell = string | { value: string; tone?: 'good' | 'bad' };

export const cellText = (cell: Cell) => (typeof cell === 'string' ? cell : cell.value);

const allRows = (columns: string[], rows: Cell[][], total?: Cell[]) => [columns, ...rows, ...(total ? [total] : [])].map((row) => row.map(cellText));

export function tableTsv(columns: string[], rows: Cell[][], total?: Cell[]): string {
	return allRows(columns, rows, total)
		.map((row) => row.map((text) => text.replace(/[\t\n]/g, ' ')).join('\t'))
		.join('\n');
}

const csvField = (text: string) => (/[",\n\r]/.test(text) ? `"${text.replaceAll('"', '""')}"` : text);

export function tableCsv(columns: string[], rows: Cell[][], total?: Cell[]): string {
	return allRows(columns, rows, total)
		.map((row) => row.map(csvField).join(','))
		.join('\n');
}
