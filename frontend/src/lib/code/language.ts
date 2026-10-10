import type { Extension } from '@codemirror/state';
import { css } from '@codemirror/lang-css';
import { html } from '@codemirror/lang-html';
import { javascript } from '@codemirror/lang-javascript';

/** CodeMirror language support for a file, by extension (none for plain text). */
export function languageFor(path: string): Extension[] {
	const ext = path.split('.').pop()?.toLowerCase();
	if (ext === 'html' || ext === 'htm' || ext === 'svelte' || ext === 'vue' || ext === 'astro') return [html()];
	if (ext === 'css') return [css()];
	if (ext === 'ts' || ext === 'mts' || ext === 'cts') return [javascript({ typescript: true })];
	if (ext === 'tsx') return [javascript({ typescript: true, jsx: true })];
	if (ext === 'jsx') return [javascript({ jsx: true })];
	if (ext === 'js' || ext === 'mjs' || ext === 'cjs' || ext === 'json') return [javascript()];
	return [];
}

/** The language name VS Code shows in its status bar. */
export function languageName(path: string): string {
	const ext = path.split('.').pop()?.toLowerCase() ?? '';
	const names: Record<string, string> = {
		ts: 'TypeScript', tsx: 'TypeScript JSX', js: 'JavaScript', mjs: 'JavaScript', cjs: 'JavaScript', jsx: 'JavaScript JSX',
		svelte: 'Svelte', vue: 'Vue', astro: 'Astro', html: 'HTML', css: 'CSS', json: 'JSON', md: 'Markdown', sql: 'SQL'
	};
	return names[ext] ?? 'Plain Text';
}
