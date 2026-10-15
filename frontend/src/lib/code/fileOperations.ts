import { unzipSync, strFromU8 } from 'fflate';

/**
 * Strips leading and trailing slashes and collapses multiple slashes.
 */
export function normalizePath(path: string): string {
	return path
		.trim()
		.replace(/^\/+/, '')
		.replace(/\/+$/, '')
		.replace(/\/+/g, '/');
}

/**
 * Gets the parent folder path of a file or folder, or empty string if at root.
 * e.g. "src/lib/db.ts" -> "src/lib", "index.html" -> ""
 */
export function parentFolder(path: string): string {
	const clean = normalizePath(path);
	const lastSlash = clean.lastIndexOf('/');
	return lastSlash === -1 ? '' : clean.slice(0, lastSlash);
}

/**
 * Gets the basename of a file or folder.
 * e.g. "src/lib/db.ts" -> "db.ts", "src/components" -> "components"
 */
export function baseName(path: string): string {
	const clean = normalizePath(path);
	const lastSlash = clean.lastIndexOf('/');
	return lastSlash === -1 ? clean : clean.slice(lastSlash + 1);
}

/**
 * Joins a folder path and a subpath or file name.
 */
export function joinPath(folder: string, subpath: string): string {
	const cleanFolder = normalizePath(folder);
	const cleanSub = normalizePath(subpath);
	if (!cleanFolder) return cleanSub;
	if (!cleanSub) return cleanFolder;
	return `${cleanFolder}/${cleanSub}`;
}

/**
 * Checks if targetPath is the same as parentPath or is a descendant of parentPath.
 */
export function isSubpathOf(targetPath: string, parentPath: string): boolean {
	const cleanTarget = normalizePath(targetPath);
	const cleanParent = normalizePath(parentPath);
	if (!cleanParent) return true; // root contains everything
	return cleanTarget === cleanParent || cleanTarget.startsWith(`${cleanParent}/`);
}

/**
 * Calculates new path when moving a file to targetFolder.
 * e.g. ("src/utils.ts", "lib") -> "lib/utils.ts"
 *      ("src/utils.ts", "") -> "utils.ts"
 */
export function relocateFilePath(sourcePath: string, targetFolder: string): string {
	const fileBase = baseName(sourcePath);
	return joinPath(targetFolder, fileBase);
}

/**
 * Calculates new path for a file inside a folder when that folder is moved to targetFolder.
 * e.g. folder "src/components" moved to "src/ui", file is "src/components/Button.svelte":
 *      relocateFolderFilePath("src/components", "src/ui", "src/components/Button.svelte")
 *      -> "src/ui/components/Button.svelte"
 */
export function relocateFolderFilePath(
	sourceFolder: string,
	targetFolder: string,
	filePath: string
): string {
	const cleanSource = normalizePath(sourceFolder);
	const cleanFile = normalizePath(filePath);
	const folderName = baseName(cleanSource);
	const subpath = cleanFile.startsWith(`${cleanSource}/`)
		? cleanFile.slice(cleanSource.length + 1)
		: cleanFile;
	const newFolder = joinPath(targetFolder, folderName);
	return joinPath(newFolder, subpath);
}

/**
 * Checks if a file is a zip archive by name or mime type.
 */
export function isZipFile(file: { name: string; type?: string }): boolean {
	const name = file.name.toLowerCase();
	return (
		name.endsWith('.zip') ||
		file.type === 'application/zip' ||
		file.type === 'application/x-zip-compressed'
	);
}

/**
 * Checks if a zip archive entry should be ignored (directories, mac metadata, git, etc.).
 */
export function shouldIgnoreZipEntry(entryName: string): boolean {
	if (entryName.endsWith('/')) return true;
	const parts = entryName.split('/');
	for (const part of parts) {
		if (
			part === '__MACOSX' ||
			part === '.DS_Store' ||
			part === '.git' ||
			part === 'node_modules' ||
			part === '..'
		) {
			return true;
		}
	}
	return false;
}

/**
 * Extracts all files from a zip archive buffer, preserving internal directory structure,
 * and prepending targetFolder if provided.
 */
export function extractZipArchive(
	buffer: ArrayBuffer | Uint8Array,
	targetFolder = ''
): { path: string; content: string }[] {
	const data = buffer instanceof Uint8Array ? buffer : new Uint8Array(buffer);
	const unzipped = unzipSync(data);
	const results: { path: string; content: string }[] = [];

	for (const [rawName, bytes] of Object.entries(unzipped)) {
		if (shouldIgnoreZipEntry(rawName)) continue;
		const cleanName = normalizePath(rawName);
		if (!cleanName) continue;

		let content: string;
		try {
			content = strFromU8(bytes);
		} catch {
			content = new TextDecoder('utf-8', { fatal: false }).decode(bytes);
		}

		const fullPath = joinPath(targetFolder, cleanName);
		results.push({ path: fullPath, content });
	}

	return results;
}
