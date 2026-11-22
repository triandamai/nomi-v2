import type { FileSystemTree } from '@webcontainer/api';

// Running a project on the person's own device (WebContainer, in this browser tab) when the
// backend's sandbox is off or can't be used: the pure parts. The runtime itself is device.svelte.ts.
// Ported from the old app's browser runtime (frontend/src/lib/projectRuntime.ts), which the backend
// still serves: the same snapshot, manifest, check-in and command endpoints.

/** The query that asks the workspace to run the project on this device (a full page load, so the server can isolate it). */
export const DEVICE_PARAM = 'run';
export const DEVICE_VALUE = 'device';

/** The workspace's own address with the device run asked for. */
export function deviceHref(projectId: string): string {
	return `/projects/${encodeURIComponent(projectId)}?${DEVICE_PARAM}=${DEVICE_VALUE}`;
}

/** Whether this request is a workspace page asked to run on the device (it gets the isolation headers). */
export function wantsDevice(url: URL): boolean {
	return /^\/projects\/[^/]+\/?$/.test(url.pathname) && url.searchParams.get(DEVICE_PARAM) === DEVICE_VALUE;
}

/** The backend paths the device runtime may reach through the workspace (nothing else is proxied). */
export const DEVICE_PATHS = /^(snapshot|manifest|files\/.+|runtime|runtime\/check|runtime\/runs\/[\w-]+|relay\/preview)$/;

/** The backend path for a device call, each part encoded; null for anything off the list or climbing out of it (`..`). */
export function devicePath(rest: string): string | null {
	const parts = rest.split('/');
	if (!DEVICE_PATHS.test(rest) || parts.some((part) => part === '.' || part === '..' || part === '')) return null;
	return parts.map((part) => encodeURIComponent(part)).join('/');
}

export interface ManifestEntry {
	path: string;
	size_bytes: number;
	updated_at: string;
}

/** A RelayDB project's dev backend, as Nomi hands it to the preview (the service key for admins only). */
export interface RelayPreviewAccess {
	url: string;
	anon_key: string;
	service_key?: string;
}

/** A file's contents as written into the container: text, or a binary file's bytes from base64. */
export function payloadContents(file: { content: string; encoding?: string }): string | Uint8Array {
	if (file.encoding !== 'base64') return file.content;
	const binary = atob(file.content);
	const bytes = new Uint8Array(binary.length);
	for (let index = 0; index < binary.length; index++) bytes[index] = binary.charCodeAt(index);
	return bytes;
}

/** The nested tree `WebContainer.mount` takes, from flat paths. */
export function toFileTree(files: { path: string; content: string; encoding?: string }[]): FileSystemTree {
	const root: FileSystemTree = {};
	for (const file of files) {
		const parts = file.path.split('/');
		let dir = root;
		for (const part of parts.slice(0, -1)) {
			const node = dir[part];
			if (!node || !('directory' in node)) dir[part] = { directory: {} };
			dir = (dir[part] as { directory: FileSystemTree }).directory;
		}
		dir[parts[parts.length - 1]] = { file: { contents: payloadContents(file) } };
	}
	return root;
}

/** The folder a file is in ('' at the top). */
export function parentDir(path: string): string {
	const index = path.lastIndexOf('/');
	return index < 0 ? '' : path.slice(0, index);
}

/** Which files to fetch (new or changed) and which to delete, against what's mounted. */
export function diffManifest(known: Map<string, string>, manifest: ManifestEntry[]): { changed: string[]; removed: string[] } {
	const current = new Set(manifest.map((file) => file.path));
	return {
		changed: manifest.filter((file) => known.get(file.path) !== file.updated_at).map((file) => file.path),
		removed: [...known.keys()].filter((path) => !current.has(path)),
	};
}

/** The preview's environment with its dev backend in the variables the app reads. */
export function previewEnv(base: Record<string, string>, access: RelayPreviewAccess | null): Record<string, string> {
	if (!access) return base;
	const env: Record<string, string> = { ...base, PUBLIC_NOMI_URL: access.url, PUBLIC_NOMI_KEY: access.anon_key, NOMI_URL: access.url, NOMI_ANON_KEY: access.anon_key };
	if (access.service_key) env.NOMI_SERVICE_KEY = access.service_key;
	return env;
}

/** How often a quiet process is looked at again. */
const EXIT_POLL_MS = 5000;

export interface ExitWatch {
	lastOutputAt: () => number;
	/** Whether the work looks finished (node_modules is there), so a quiet process can be let go. */
	settled: () => Promise<boolean>;
	quietMs: number;
	timeoutMs: number;
	pollMs?: number;
	now?: () => number;
}

/**
 * Waits for a process to exit, or gives up on it: quiet for `quietMs` once its work looks finished
 * is 'hung' (some installs never exit in a WebContainer), past `timeoutMs` is 'timeout'.
 */
export function watchExit(exit: Promise<number>, watch: ExitWatch): Promise<number | 'hung' | 'timeout'> {
	const now = watch.now ?? Date.now;
	const started = now();
	const pollMs = watch.pollMs ?? EXIT_POLL_MS;
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
			if (!done) timer = setTimeout(poll, pollMs);
		};
		timer = setTimeout(poll, pollMs);
	});
}
