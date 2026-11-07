// Helpers for running a project in the browser (see projectRuntime.svelte.ts): turning the
// project's files into a WebContainer tree, working out which files changed, and tidying
// terminal output for the log.

import type { FileSystemTree } from '@webcontainer/api';
import { payloadContents } from '$lib/code/fileBytes';

export interface ManifestEntry {
	path: string;
	size_bytes: number;
	updated_at: string;
}

/** The nested tree `WebContainer.mount` takes, from flat `path → content` (binary files in base64). */
export function toFileTree(files: { path: string; content: string; encoding?: string }[]): FileSystemTree {
	const root: FileSystemTree = {};
	for (const file of files) {
		const { path } = file;
		const content = payloadContents(file);
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

/** A Relay project's dev backend, as Nomi gives it to the preview (the service key for admins only). */
export interface RelayPreviewAccess {
	url: string;
	anon_key: string;
	service_key?: string;
}

/** The preview's environment with its dev backend in the variables the app reads (see the relay guide). */
export function previewEnv(base: Record<string, string>, access: RelayPreviewAccess | null): Record<string, string> {
	if (!access) return base;
	const env: Record<string, string> = {
		...base,
		PUBLIC_NOMI_URL: access.url,
		PUBLIC_NOMI_KEY: access.anon_key,
		NOMI_URL: access.url,
		NOMI_ANON_KEY: access.anon_key,
	};
	if (access.service_key) env.NOMI_SERVICE_KEY = access.service_key;
	return env;
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

/**
 * Appends a terminal text segment to the existing log, correctly resolving:
 * - ANSI escapes (stripped)
 * - CRLF \r\n (converted to \n)
 * - Lone \r (returns cursor to column 0 so subsequent output replaces the current line)
 */
function appendSegment(log: string, segment: string): string {
	const normalized = segment.replace(ANSI, '').replace(/\r\n/g, '\n');
	let combined = log;
	for (let i = 0; i < normalized.length; i++) {
		const ch = normalized[i];
		if (ch === '\r') {
			if (!combined.endsWith('\r')) combined += '\r';
		} else if (ch === '\n') {
			if (combined.endsWith('\r')) combined = combined.slice(0, -1);
			combined += '\n';
		} else {
			if (combined.endsWith('\r')) {
				combined = combined.slice(0, -1);
				const lastNl = combined.lastIndexOf('\n');
				combined = lastNl >= 0 ? combined.slice(0, lastNl + 1) : '';
			}
			combined += ch;
		}
	}
	return combined;
}

/** Terminal output as plain text: no colour codes, carriage-return redraws collapsed. */
export function cleanOutput(chunk: string): string {
	return appendSegment('', chunk).replace(/\r+$/, '');
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
	let next = appendSegment(log, segments[0]);
	for (const segment of segments.slice(1)) {
		next = dropTrailingBlankLines(next);
		next = appendSegment(next, segment);
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

export interface ExitWatch {
	/** When the process last printed something (ms, like Date.now()). */
	lastOutputAt: () => number;
	/** Whether the work looks finished (say, node_modules is there), so a quiet process can be let go. */
	settled: () => Promise<boolean>;
	/** How long the process may sit quiet once its work looks finished. */
	quietMs: number;
	/** The longest it may run at all. */
	timeoutMs: number;
	pollMs?: number;
	now?: () => number;
}

/**
 * Waits for a process to exit, or gives up on it. Some installs never exit in a WebContainer even
 * though their work is done: Rolldown fetches its WebAssembly binding during `prepare`
 * (`svelte-kit sync`), and that binding's worker threads keep the script alive. So a process that
 * has gone quiet for `quietMs` after its work looks finished counts as 'hung' (the caller kills
 * it and carries on), and one that runs past `timeoutMs` as 'timeout'.
 */
export function watchExit(exit: Promise<number>, watch: ExitWatch): Promise<number | 'hung' | 'timeout'> {
	const now = watch.now ?? Date.now;
	const started = now();
	return new Promise((resolve) => {
		let done = false;
		let timer: ReturnType<typeof setTimeout> | undefined;
		const finish = (result: number | 'hung' | 'timeout') => {
			if (done) return;
			done = true;
			clearTimeout(timer);
			resolve(result);
		};
		void exit.then(finish, () => finish(1));
		const poll = async () => {
			if (done) return;
			const at = now();
			if (at - started >= watch.timeoutMs) return finish('timeout');
			if (at - watch.lastOutputAt() >= watch.quietMs && (await watch.settled().catch(() => false))) return finish('hung');
			if (!done) timer = setTimeout(poll, watch.pollMs ?? 5000);
		};
		timer = setTimeout(poll, watch.pollMs ?? 5000);
	});
}
