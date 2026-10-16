// Brings existing code into a project: a zip unpacked here in the browser and sent in small
// batches (the web app only takes small requests), or a public GitHub repository fetched by
// the server.

import { unzipSync } from 'fflate';
import { projectApi } from '$lib/projectApi';

export interface ImportResult {
	imported: number;
	skipped: number;
	stack: string;
}

/** Folders and files that never belong in a project (installed packages, build output, secrets). */
export function excluded(path: string): boolean {
	const parts = path.split('/');
	const dirs = ['node_modules', '.git', '.svelte-kit', 'build', 'dist', '.data', '.astro', '.vercel', '.output', '__MACOSX'];
	if (dirs.includes(parts[0]) || parts.includes('node_modules') || parts.includes('.git') || parts.includes('__MACOSX')) return true;
	const name = parts[parts.length - 1];
	return (
		name === '.DS_Store' ||
		((name === '.env' || name.startsWith('.env.')) && name !== '.env.example') ||
		/\.(db|sqlite|db-journal)$/.test(name)
	);
}

const MAX_FILE_BYTES = 1024 * 1024;
const BATCH_BYTES = 300 * 1024;

/** The text files of a zip, with the one folder they all sit in (GitHub's `repo-main/`) removed. */
export function readZip(bytes: Uint8Array): { files: { path: string; content: string }[]; skipped: number } {
	const entries = Object.entries(unzipSync(bytes)).filter(([path]) => !path.endsWith('/'));
	const first = entries[0]?.[0] ?? '';
	const top = first.includes('/') ? `${first.split('/')[0]}/` : null;
	const strip = top && entries.every(([path]) => path.startsWith(top)) ? top.length : 0;
	const decoder = new TextDecoder('utf-8', { fatal: true });
	const files: { path: string; content: string }[] = [];
	let skipped = 0;
	for (const [rawPath, data] of entries) {
		const path = rawPath.slice(strip);
		if (!path || excluded(path) || data.length > MAX_FILE_BYTES) {
			skipped += 1;
			continue;
		}
		try {
			const content = decoder.decode(data);
			if (content.includes('\0')) throw new Error('binary');
			files.push({ path, content });
		} catch {
			skipped += 1;
		}
	}
	return { files, skipped };
}

/** Sends `files` in batches; `onProgress` gets how many were sent. */
export async function uploadFiles(
	projectId: string,
	files: { path: string; content: string }[],
	onProgress?: (sent: number) => void,
): Promise<ImportResult> {
	let imported = 0;
	let skipped = 0;
	let stack = '';
	let index = 0;
	do {
		const batch: typeof files = [];
		let size = 0;
		while (index < files.length && batch.length < 400 && (batch.length === 0 || size + files[index].content.length < BATCH_BYTES)) {
			size += files[index].content.length + files[index].path.length;
			batch.push(files[index]);
			index += 1;
		}
		const finish = index >= files.length;
		const result = await projectApi<ImportResult>(projectId, 'import/files', { method: 'POST', body: { files: batch, finish } });
		imported += result.imported;
		skipped += result.skipped;
		if (finish) stack = result.stack;
		onProgress?.(index);
	} while (index < files.length);
	return { imported, skipped, stack };
}

export async function importZip(projectId: string, file: File, onProgress?: (sent: number, total: number) => void): Promise<ImportResult> {
	const { files, skipped } = readZip(new Uint8Array(await file.arrayBuffer()));
	if (files.length === 0) throw new Error('empty');
	const result = await uploadFiles(projectId, files, (sent) => onProgress?.(sent, files.length));
	return { ...result, skipped: result.skipped + skipped };
}

export function importGithub(projectId: string, repo: string, ref?: string): Promise<ImportResult> {
	return projectApi<ImportResult>(projectId, 'import/github', { method: 'POST', body: { repo, ref: ref || null } });
}
