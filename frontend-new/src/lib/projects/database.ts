// Database (Flow / Build — Workspace tools U16wU, step 11): a project's tables, read only. The
// statement is built on the server from a table the backend listed, never typed in the browser.

const quote = (name: string) => `"${name.replace(/"/g, '""')}"`;

/** The first `limit` rows of a table the backend listed. */
export function readTable(table: { schema: string; name: string }, limit: number): string {
	return `SELECT * FROM ${quote(table.schema)}.${quote(table.name)} LIMIT ${Math.max(1, Math.floor(limit))}`;
}

/** A value as the grid shows it: nothing as a dash, structures as their JSON. */
export function cellText(value: unknown): string {
	if (value === null || value === undefined) return '—';
	if (typeof value === 'object') return JSON.stringify(value);
	return String(value);
}
