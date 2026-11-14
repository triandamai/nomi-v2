import { createCipheriv, createDecipheriv, createHash, randomBytes } from 'node:crypto';
import { env } from '$env/dynamic/private';

// A project's static preview runs sandboxed, in an origin of its own, so the files Koda (or a
// teammate) wrote never run as this app. A sandboxed page's requests carry no sign-in cookie, so
// the frame's address carries a grant instead: the project and the person's token, sealed with a
// key only this server has (AES-GCM), for an hour at most. It opens that project's preview files
// and nothing else; the token inside is never readable from the address.

const MAX_AGE_MS = 60 * 60 * 1000;
const IV_BYTES = 12;
const TAG_BYTES = 16;

let processKey: Buffer | undefined;
/** `PREVIEW_KEY` (needed when several servers share the traffic), else a key for this process. */
function key(): Buffer {
	if (env.PREVIEW_KEY) return createHash('sha256').update(env.PREVIEW_KEY).digest();
	processKey ??= randomBytes(32);
	return processKey;
}

/** When the token itself runs out, read from its `exp`; null when it doesn't say. */
function tokenExpiry(token: string): number | null {
	try {
		const exp = (JSON.parse(Buffer.from(token.split('.')[1] ?? '', 'base64url').toString()) as { exp?: unknown }).exp;
		return typeof exp === 'number' ? exp * 1000 : null;
	} catch {
		return null;
	}
}

export function sealPreview(projectId: string, accessToken: string, now = Date.now()): string {
	const until = Math.min(now + MAX_AGE_MS, tokenExpiry(accessToken) ?? Infinity);
	const iv = randomBytes(IV_BYTES);
	const cipher = createCipheriv('aes-256-gcm', key(), iv);
	const sealed = Buffer.concat([cipher.update(JSON.stringify({ p: projectId, a: accessToken, e: until })), cipher.final()]);
	return Buffer.concat([iv, cipher.getAuthTag(), sealed]).toString('base64url');
}

/** The token a grant holds, when it's for this project and still good; null for anything else. */
export function openPreview(grant: string, projectId: string, now = Date.now()): string | null {
	try {
		const bytes = Buffer.from(grant, 'base64url');
		if (bytes.length <= IV_BYTES + TAG_BYTES) return null;
		const decipher = createDecipheriv('aes-256-gcm', key(), bytes.subarray(0, IV_BYTES));
		decipher.setAuthTag(bytes.subarray(IV_BYTES, IV_BYTES + TAG_BYTES));
		const opened = JSON.parse(Buffer.concat([decipher.update(bytes.subarray(IV_BYTES + TAG_BYTES)), decipher.final()]).toString()) as { p: string; a: string; e: number };
		return opened.p === projectId && now < opened.e ? opened.a : null;
	} catch {
		return null;
	}
}
