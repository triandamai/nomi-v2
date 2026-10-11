// Helpers for running a project in the browser (see projectRuntime.svelte.ts): turning the
// project's files into a WebContainer tree, working out which files changed, and tidying
// terminal output for the log.

import type { FileSystemTree } from '@webcontainer/api';

export interface ManifestEntry {
	path: string;
	size_bytes: number;
	updated_at: string;
}

/** The nested tree `WebContainer.mount` takes, from flat `path → content`. */
export function toFileTree(files: { path: string; content: string }[]): FileSystemTree {
	const root: FileSystemTree = {};
	for (const { path, content } of files) {
		const parts = path.split('/');
		let dir = root;
		for (const part of parts.slice(0, -1)) {
			const node = dir[part];
			if (!node || !('directory' in node)) dir[part] = { directory: {} };
			dir = (dir[part] as { directory: FileSystemTree }).directory;
		}
		dir[parts[parts.length - 1]] = { file: { contents: content } };
	}
	return root;
}

/** Which files to fetch (new or changed since) and which to delete, against what's mounted. */
export function diffManifest(known: Map<string, string>, manifest: ManifestEntry[]): { changed: string[]; removed: string[] } {
	const current = new Set(manifest.map((f) => f.path));
	return {
		changed: manifest.filter((f) => known.get(f.path) !== f.updated_at).map((f) => f.path),
		removed: [...known.keys()].filter((path) => !current.has(path)),
	};
}

/** The folder a file is in ('' at the top). */
export function parentDir(path: string): string {
	const i = path.lastIndexOf('/');
	return i < 0 ? '' : path.slice(0, i);
}

// eslint-disable-next-line no-control-regex
const ANSI = /\u001b\[[0-9;?]*[ -/]*[@-~]|\u001b\][^\u0007]*\u0007|\u001b[()][A-Z0-9]/g;

/** Terminal output as plain text: no colour codes, carriage-return redraws collapsed. */
export function cleanOutput(chunk: string): string {
	return chunk
		.replace(ANSI, '')
		.split('\n')
		.map((line) => {
			const parts = line.split('\r').filter((p) => p.length > 0);
			return parts.length ? parts[parts.length - 1] : '';
		})
		.join('\n');
}

// Clearing the screen: a cursor jump to the top (ESC[H, ESC[row;colH), erase-display (ESC[J,
// ESC[2J...) or a full reset (ESC c). Vite clears by printing a screenful of blank lines and then
// jumping back to the top, so the blank lines have to go along with the jump.
// eslint-disable-next-line no-control-regex
const CLEAR = /\u001b\[(?:\d*;\d*)?[Hf]|\u001b\[[0-3]?J|\u001bc/;

/** `log` with the blank lines at its end removed (keeps its final newline). */
function dropTrailingBlankLines(log: string): string {
	const trimmed = log.replace(/(?:\n[ \t]*)+$/, '');
	return trimmed.length > 0 ? `${trimmed}\n` : '';
}

/**
 * Adds a chunk of terminal output to the log. A screen clear isn't replayed (the history stays);
 * the blank padding before it is dropped, and long runs of blank lines collapse to one.
 */
export function appendOutput(log: string, chunk: string): string {
	const segments = chunk.split(CLEAR);
	let next = log + cleanOutput(segments[0]);
	for (const segment of segments.slice(1)) {
		next = dropTrailingBlankLines(next) + cleanOutput(segment);
	}
	return next.replace(/\n(?:[ \t]*\n){2,}/g, '\n\n');
}

/** Keeps the last `max` characters of a growing log, starting on a whole line. */
export function capLog(log: string, max = 200_000): string {
	if (log.length <= max) return log;
	const cut = log.slice(log.length - max);
	const newline = cut.indexOf('\n');
	return newline >= 0 ? cut.slice(newline + 1) : cut;
}

/** The npm scripts in a package.json, or none when it isn't valid. */
export function scriptsOf(packageJson: string | undefined): Record<string, string> {
	if (!packageJson) return {};
	try {
		const scripts = (JSON.parse(packageJson) as { scripts?: Record<string, string> }).scripts;
		return scripts && typeof scripts === 'object' ? scripts : {};
	} catch {
		return {};
	}
}
