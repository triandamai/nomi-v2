// Editing one row of a project's table in the Database sheet: the cell's new text read back
// into a value of the cell's kind, and the row's primary key. The backend gets column names and
// values, never SQL.

/** The text a cell is edited as: NULL stays empty, objects as JSON. */
export function cellText(value: unknown): string {
	if (value === null || value === undefined) return '';
	return typeof value === 'object' ? JSON.stringify(value) : String(value);
}

/** The edited text as a value like the original: numbers stay numbers, true/false booleans,
 * JSON objects and arrays parsed; an emptied cell that was NULL stays NULL. Anything else is
 * sent as text and the database reads it into the column's type (or says why it can't). */
export function parseCell(text: string, original: unknown): unknown {
	if (text === '' && (original === null || original === undefined)) return null;
	if (typeof original === 'number') {
		const n = Number(text);
		return text.trim() !== '' && Number.isFinite(n) ? n : text;
	}
	if (typeof original === 'boolean') {
		if (text === 'true') return true;
		if (text === 'false') return false;
		return text;
	}
	if (original !== null && typeof original === 'object') {
		try {
			return JSON.parse(text);
		} catch {
			return text;
		}
	}
	return text;
}

/** The row's primary key as `{column: value}`, or null when a key column isn't in the result. */
export function rowKey(columns: string[], row: unknown[], primaryKey: string[]): Record<string, unknown> | null {
	if (primaryKey.length === 0) return null;
	const key: Record<string, unknown> = {};
	for (const column of primaryKey) {
		const at = columns.indexOf(column);
		if (at < 0) return null;
		key[column] = row[at];
	}
	return key;
}
