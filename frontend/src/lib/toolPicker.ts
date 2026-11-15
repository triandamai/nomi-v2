// The tool catalog's search and selection rules, shared by the ToolPicker, the "@" picker in a
// soul editor and the admin's agent cards. Pure functions; the components own the state.
import { m } from '$lib/paraglide/messages';
import type { CatalogTool } from './types';

/** The catalog endpoint's reply (GET /api/tools/catalog). */
export interface ToolCatalog {
	tools: CatalogTool[];
}

/** How a tool matched a query; lower ranks come first. */
const RANK_NAME_PREFIX = 0;
const RANK_NAME_CONTAINS = 1;
const RANK_KEYWORD = 2;
const RANK_DESCRIPTION = 3;

function rank(tool: CatalogTool, query: string, description: string): number | null {
	const name = tool.name.toLowerCase();
	if (name.startsWith(query)) return RANK_NAME_PREFIX;
	if (name.includes(query)) return RANK_NAME_CONTAINS;
	if (tool.keywords.some((keyword) => keyword.toLowerCase().includes(query))) return RANK_KEYWORD;
	if (description.toLowerCase().includes(query) || tool.description.toLowerCase().includes(query)) return RANK_DESCRIPTION;
	return null;
}

/** The tools matching `query` by name, keyword or description (the one shown, `describe`), best
 * matches first and the catalog's order within a rank. An empty query keeps them all. Spaces in
 * the query match the underscores in names ("list files" finds list_files). */
export function filterTools(options: CatalogTool[], query: string, describe: (tool: CatalogTool) => string = (tool) => tool.description): CatalogTool[] {
	const q = query.trim().toLowerCase().replace(/\s+/g, '_');
	if (!q) return options;
	const ranked: { tool: CatalogTool; rank: number; index: number }[] = [];
	options.forEach((tool, index) => {
		const matched = rank(tool, q, describe(tool)) ?? rank(tool, q.replace(/_/g, ' '), describe(tool));
		if (matched !== null) ranked.push({ tool, rank: matched, index });
	});
	ranked.sort((a, b) => a.rank - b.rank || a.index - b.index);
	return ranked.map((entry) => entry.tool);
}

/** `selected` with `name` added (once). */
export function addTool(selected: string[], name: string): string[] {
	return selected.includes(name) ? selected : [...selected, name];
}

/** `selected` without `name`. */
export function removeTool(selected: string[], name: string): string[] {
	return selected.filter((tool) => tool !== name);
}

/** `selected` without its last tool (Backspace in an empty search). */
export function removeLastTool(selected: string[]): string[] {
	return selected.slice(0, -1);
}

/** The tool's description in the person's language when a `tooldesc_<name>` message exists, else
 * the catalog's own. */
export function toolDescription(tool: CatalogTool): string {
	const translate = (m as unknown as Record<string, (() => string) | undefined>)[`tooldesc_${tool.name}`];
	return translate?.() ?? tool.description;
}

/** The catalog from the endpoint's reply; anything else is an empty catalog. */
export function catalogFrom(body: unknown): CatalogTool[] {
	const tools = (body as Partial<ToolCatalog> | null)?.tools;
	return Array.isArray(tools) ? tools : [];
}
